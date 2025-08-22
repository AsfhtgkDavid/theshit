mod python;
mod rust;
mod structs;

use crate::fix::rust::NativeRule;
use crate::fix::structs::CommandOutput;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};
use crossterm::style::Stylize;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::mpsc;
use std::time::Duration;
use std::{fs, io, thread};
use structs::RawModeGuard;

pub fn fix_command(command: String, expand_command: String) -> io::Result<String> {
    let command_output = match get_command_output(expand_command) {
        Ok(output) => output,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => CommandOutput::new(
                "command not found".to_string(),
                "command not found".to_string(),
            ),
            ErrorKind::PermissionDenied => CommandOutput::new(
                "permission denied".to_string(),
                "permission denied".to_string(),
            ),
            _ => {
                eprintln!("{}: {}", "Error executing command".red(), e);
                return Err(e);
            }
        },
    };
    let command_struct = structs::Command::new(command, command_output);
    let active_rules_dir = dirs::config_dir()
        .ok_or(ErrorKind::NotFound)?
        .join("theshit/fix_rules/active");
    let mut fixed_commands: Vec<String> = vec![];
    let mut python_rules: Vec<PathBuf> = vec![];
    for rule in fs::read_dir(active_rules_dir)? {
        let rule = rule?;
        let path = rule.path();
        if path
            .file_name()
            .unwrap_or_else(|| panic!("Can't get get file name for {}", path.display()))
            .to_string_lossy()
            == "__pycache__"
        {
            continue;
        }
        match path.extension() {
            Some(extension) => match extension.to_string_lossy().as_ref() {
                "native" => {
                    let native_rule_name = match path.file_stem() {
                        Some(name) => name,
                        None => {
                            eprintln!("{}{}", "Failed to get stem for: ".yellow(), path.display());
                            continue;
                        }
                    };
                    let native_rule =
                        NativeRule::from_str(native_rule_name.to_string_lossy().as_ref());
                    match native_rule {
                        Ok(rule) => {
                            if let Some(fixed) = rule.fix_native(&command_struct) {
                                fixed_commands.push(fixed)
                            }
                        }
                        Err(_) => {
                            eprintln!(
                                "{}{}{}",
                                "Native rule '".yellow(),
                                native_rule_name.to_string_lossy(),
                                "' isn't supported".yellow()
                            );
                            continue;
                        }
                    }
                }
                "py" => python_rules.push(path),
                _ => {
                    eprintln!(
                        "{}{}{}",
                        "Rule type '".yellow(),
                        path.display(),
                        "' isn't supported".yellow()
                    )
                }
            },
            None => {
                eprintln!("{}{}", "Can't get extension for ".yellow(), path.display())
            }
        }
    }
    if !python_rules.is_empty() {
        match python::process_python_rules(&command_struct, python_rules) {
            Ok(commands) => fixed_commands.extend(commands),
            Err(e) => eprintln!("{}: {}", "Python rules processing failed".red(), e),
        }
    }
    Ok(choose_fixed_command(fixed_commands))
}

fn get_command_timeout(command_name: &str) -> Duration {
    // Get the base command name without path
    let base_command = command_name
        .split('/')
        .last()
        .unwrap_or(command_name);
    
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

fn get_command_output(expand_command: String) -> io::Result<CommandOutput> {
    let split_command = shell_words::split(&expand_command)
        .map_err(|e| io::Error::other(format!("Failed to parse command: {e}")))?;
    
    if split_command.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Empty command provided"
        ));
    }
    
    let timeout = get_command_timeout(&split_command[0]);
    
    let child = Command::new(&split_command[0])
        .args(&split_command[1..])
        .env("LANG", "C") // Set locale to C to avoid issues with rules that depend on locale
        .env("LC_ALL", "C")
        .spawn()?;
    
    // Create a channel to communicate between threads
    let (sender, receiver) = mpsc::channel();
    
    // Spawn a thread to wait for the child process
    let _handle = thread::spawn(move || {
        let result = child.wait_with_output();
        let _ = sender.send(result);
    });
    
    // Wait for either the command to complete or timeout
    match receiver.recv_timeout(timeout) {
        Ok(Ok(output)) => Ok(CommandOutput::from(output)),
        Ok(Err(e)) => Err(e),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Command timed out, we need to kill it
            // Unfortunately, we moved `child` into the thread, so we can't kill it directly
            // We'll let the thread continue and return a timeout error
            Err(io::Error::new(
                ErrorKind::TimedOut,
                format!("Command timed out after {:?}", timeout)
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(io::Error::new(
                ErrorKind::Other,
                "Command thread disconnected unexpectedly"
            ))
        }
    }
}

fn choose_fixed_command(mut fixed_commands: Vec<String>) -> String {
    if fixed_commands.is_empty() {
        eprintln!(
            "{}: {}",
            "No fixed commands found".yellow(),
            "Exiting...".red()
        );
        std::process::exit(1);
    }

    let mut current_command = fixed_commands.first().unwrap();
    let mut current_index = 0;

    eprintln!();
    let _raw_mode_guard = RawModeGuard::new();
    let mut err = io::stderr();
    err.write_all(
        format!(
            "{} [{}/{}/{}/{}]",
            current_command,
            "enter".green(),
            "↑".cyan(),
            "↓".cyan(),
            "Ctrl+C".red()
        )
        .as_bytes(),
    )
    .expect("Failed to write to stderr");
    loop {
        match read() {
            Ok(event) => {
                if let Event::Key(KeyEvent {
                    code, modifiers, ..
                }) = event
                {
                    match (code, modifiers) {
                        (KeyCode::Up, _) => {
                            if fixed_commands.len() > 1 {
                                if current_index > 0 {
                                    current_index -= 1;
                                } else {
                                    current_index = fixed_commands.len() - 1;
                                }
                                current_command = fixed_commands.get(current_index).unwrap();
                                err.write_all(
                                    format!(
                                        "{} [{}/{}/{}/{}]",
                                        current_command,
                                        "enter".green(),
                                        "↑".cyan(),
                                        "↓".cyan(),
                                        "Ctrl+C".red()
                                    )
                                    .as_bytes(),
                                )
                                .expect("Failed to write to stderr");
                            }
                        }
                        (KeyCode::Down, _) => {
                            if fixed_commands.len() > 1 {
                                if current_index < fixed_commands.len() - 1 {
                                    current_index += 1;
                                } else {
                                    current_index = 0;
                                }
                                current_command = fixed_commands.get(current_index).unwrap();
                                err.write_all(
                                    format!(
                                        "{} [{}/{}/{}/{}]",
                                        current_command,
                                        "enter".green(),
                                        "↑".cyan(),
                                        "↓".cyan(),
                                        "Ctrl+C".red()
                                    )
                                    .as_bytes(),
                                )
                                .expect("Failed to write to stderr");
                            }
                        }
                        (KeyCode::Enter, _) => {
                            drop(_raw_mode_guard);
                            eprintln!();
                            eprintln!("{}: {}", "Selected command: ".green(), &current_command);
                            return fixed_commands.remove(current_index);
                        }
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            drop(_raw_mode_guard);
                            eprintln!();
                            eprintln!("{}: {}", "Exiting...".yellow(), "User interrupted".red());
                            std::process::exit(1);
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => {
                eprintln!("{}: {}", "Error reading input".red(), "Exiting...".yellow());
                drop(_raw_mode_guard);
                std::process::exit(1);
            }
        }
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
        assert_eq!(get_command_timeout("/usr/local/bin/gradle"), Duration::from_secs(10));
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
        let err = result.err().unwrap();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }
    
    #[test]
    fn test_get_command_output_nonexistent_command() {
        let result = get_command_output("nonexistent_command_12345".to_string());
        assert!(result.is_err());
        // Note: The exact error type may vary between systems
        let err = result.err().unwrap();
        assert!(matches!(err.kind(), ErrorKind::NotFound));
    }
}
