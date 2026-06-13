use crate::error::{AppError, AppResult};
use crate::fix::structs::CommandOutput;
use crate::misc::split_command;
use crossterm::style::Stylize;
use libc::{geteuid, getuid, gid_t, uid_t};
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::Duration;

pub fn get_command_output(expand_command: String) -> AppResult<CommandOutput> {
    let split_command = split_command(&expand_command);

    if split_command.is_empty() {
        return Err(AppError::Other("Command is empty".to_string()));
    }

    let timeout = get_command_timeout(&split_command[0]);

    let mut command = Command::new(&split_command[0]);
    command
        .args(&split_command[1..])
        .env("LANG", "C") // Set locale to C to avoid issues with rules that depend on locale
        .env("LC_ALL", "C");

    let permission_issue = PermissionIssue::detect();
    permission_issue.fix(&mut command)?;

    let (sender, receiver) = std::sync::mpsc::channel();

    let _handle = std::thread::spawn(move || {
        let output = command.output();
        let _ = sender.send(output);
    });

    match receiver.recv_timeout(timeout) {
        Ok(Ok(output)) => Ok(CommandOutput::from(output)),
        Ok(Err(e)) => Err(AppError::Io(e)),
        Err(_) => Err(AppError::Other("Command execution timed out".to_string())),
    }
}

enum PermissionIssue {
    EuidNotEqualRuid, // SUID/SGID execution detected
    SudoEnvironment,  // SUDO_UID or SUDO_GID is set
    DoasEnvironment,  // Detected running in a Doas environment
    Normal,           // No permission issues detected
}

impl PermissionIssue {
    fn detect() -> Self {
        if unsafe { geteuid() != getuid() } {
            return PermissionIssue::EuidNotEqualRuid;
        }
        if std::env::var("SUDO_UID").is_ok() || std::env::var("SUDO_GID").is_ok() {
            return PermissionIssue::SudoEnvironment;
        }
        if std::env::var("DOAS_USER").is_ok() {
            return PermissionIssue::DoasEnvironment;
        }
        PermissionIssue::Normal
    }

    fn fix(self, command: &mut Command) -> AppResult<()> {
        let current_euid = unsafe { geteuid() };
        let current_uid = unsafe { getuid() };
        let current_egid = unsafe { libc::getegid() };
        let current_gid = unsafe { libc::getgid() };
        let change_supplementary_groups = matches!(
            self,
            PermissionIssue::SudoEnvironment | PermissionIssue::DoasEnvironment
        );

        let target = match self {
            PermissionIssue::EuidNotEqualRuid => (current_uid, current_gid),
            PermissionIssue::SudoEnvironment => {
                let uid = std::env::var("SUDO_UID").map_err(|e| {
                    AppError::Security(format!(
                        "{} Detected sudo environment, but cannot get SUDO_UID: {}",
                        "SECURITY ERROR:".red().bold(),
                        e
                    ))
                })?;
                let gid = std::env::var("SUDO_GID").map_err(|e| {
                    AppError::Security(format!(
                        "{} Detected sudo environment, but cannot get SUDO_GID: {}",
                        "SECURITY ERROR:".red().bold(),
                        e
                    ))
                })?;
                (
                    uid.parse::<uid_t>().map_err(|e| {
                        AppError::Security(format!(
                            "{} Invalid SUDO_UID value: {}",
                            "SECURITY ERROR:".red().bold(),
                            e
                        ))
                    })?,
                    gid.parse::<gid_t>().map_err(|e| {
                        AppError::Security(format!(
                            "{} Invalid SUDO_GID value: {}",
                            "SECURITY ERROR:".red().bold(),
                            e
                        ))
                    })?,
                )
            }
            PermissionIssue::DoasEnvironment => {
                let doas_user = std::env::var("DOAS_USER").map_err(|e| {
                    AppError::Security(format!(
                        "{} Detected doas environment, but cannot get DOAS_USER: {}",
                        "SECURITY ERROR:".red().bold(),
                        e
                    ))
                })?;
                get_ids_by_username(doas_user)?
            }
            PermissionIssue::Normal => return Ok(()),
        };

        if target == (current_euid, current_egid) {
            return Ok(());
        }

        let user_context = get_user_context_by_uid(target.0)?;

        if change_supplementary_groups && current_euid != 0 {
            return Err(AppError::Security(format!(
                "{} Cannot change supplementary groups when not running as root.",
                "SECURITY ERROR:".red().bold()
            )));
        }

        command.env("USER", &user_context.username);
        command.env("HOME", &user_context.home_dir);

        command.env_remove("SUDO_UID");
        command.env_remove("SUDO_GID");
        command.env_remove("SUDO_USER");
        command.env_remove("SUDO_COMMAND");
        command.env_remove("DOAS_USER");

        let username_cstring = std::ffi::CString::new(user_context.username).unwrap();

        unsafe {
            command.pre_exec(move || {
                if current_euid == 0
                    && change_supplementary_groups
                    && libc::initgroups(username_cstring.as_ptr(), target.1) != 0
                {
                    return Err(io::Error::last_os_error());
                }
                if libc::setgid(target.1) != 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::setuid(target.0) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        Ok(())
    }
}
fn get_ids_by_username(username: String) -> AppResult<(uid_t, gid_t)> {
    let user_cstr = std::ffi::CString::new(username.clone()).map_err(|e| {
        AppError::Security(format!(
            "{} Username contains null bytes: {}",
            "SECURITY ERROR:".red().bold(),
            e
        ))
    })?;
    let passwd = unsafe { libc::getpwnam(user_cstr.as_ptr()) };
    if passwd.is_null() {
        return Err(AppError::Security(format!(
            "{} Cannot find user info for '{}'",
            "SECURITY ERROR:".red().bold(),
            username
        )));
    }
    unsafe { Ok(((*passwd).pw_uid, (*passwd).pw_gid)) }
}

struct UserContext {
    username: String,
    home_dir: String,
}

fn get_user_context_by_uid(uid: uid_t) -> AppResult<UserContext> {
    let passwd = unsafe { libc::getpwuid(uid) };
    if passwd.is_null() {
        return Err(AppError::Security(format!(
            "{} Cannot find user info for UID '{}'",
            "SECURITY ERROR:".red().bold(),
            uid
        )));
    }
    unsafe {
        let username = std::ffi::CStr::from_ptr((*passwd).pw_name)
            .to_string_lossy()
            .into_owned();

        let home_dir = std::ffi::CStr::from_ptr((*passwd).pw_dir)
            .to_string_lossy()
            .into_owned();

        Ok(UserContext { username, home_dir })
    }
}

fn get_command_timeout(command_name: &str) -> Duration {
    // Get the base command name without path
    let base_command = command_name.split('/').next_back().unwrap_or(command_name);

    match base_command {
        // Slow commands that may take longer
        "gradle" | "gradlew" => Duration::from_secs(10),
        "mvn" | "maven" => Duration::from_secs(10),
        "npm" | "yarn" | "pnpm" => Duration::from_secs(10),
        "cargo" => Duration::from_secs(10),
        "docker" | "podman" => Duration::from_secs(10),
        "kubectl" | "helm" => Duration::from_secs(10),
        "terraform" | "tf" => Duration::from_secs(10),
        "ansible" | "ansible-playbook" => Duration::from_secs(10),

        // Medium-speed commands
        "git" => Duration::from_secs(5),
        "make" => Duration::from_secs(5),
        "pip" | "pip3" => Duration::from_secs(5),
        "composer" => Duration::from_secs(5),
        "bundle" => Duration::from_secs(5),

        // Fast commands - default timeout
        _ => Duration::from_secs(1),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command_timeout_fast_commands() {
        assert_eq!(get_command_timeout("ls"), Duration::from_secs(1));
        assert_eq!(get_command_timeout("echo"), Duration::from_secs(1));
        assert_eq!(get_command_timeout("cat"), Duration::from_secs(1));
        assert_eq!(get_command_timeout("/bin/ls"), Duration::from_secs(1));
    }

    #[test]
    fn test_get_command_timeout_slow_commands() {
        assert_eq!(get_command_timeout("gradle"), Duration::from_secs(10));
        assert_eq!(get_command_timeout("gradlew"), Duration::from_secs(10));
        assert_eq!(get_command_timeout("mvn"), Duration::from_secs(10));
        assert_eq!(get_command_timeout("npm"), Duration::from_secs(10));
        assert_eq!(get_command_timeout("cargo"), Duration::from_secs(10));
        assert_eq!(get_command_timeout("docker"), Duration::from_secs(10));
        assert_eq!(
            get_command_timeout("/usr/local/bin/gradle"),
            Duration::from_secs(10)
        );
    }

    #[test]
    fn test_get_command_timeout_medium_commands() {
        assert_eq!(get_command_timeout("git"), Duration::from_secs(5));
        assert_eq!(get_command_timeout("make"), Duration::from_secs(5));
        assert_eq!(get_command_timeout("pip"), Duration::from_secs(5));
        assert_eq!(get_command_timeout("/usr/bin/git"), Duration::from_secs(5));
    }

    #[test]
    fn test_get_command_output_empty_command() {
        let result = get_command_output("".to_string());
        assert!(result.is_err());
        let err = result.err().expect("Expected error but got success");
        assert!(err.to_string().contains("Command is empty"));
    }

    #[test]
    fn test_get_command_output_nonexistent_command() {
        let result = get_command_output("nonexistent_command_12345".to_string());
        assert!(result.is_err());
        let err = result.err().expect("Expected error but got success");
        assert!(err.to_string().contains("No such file or directory"));
    }
}
