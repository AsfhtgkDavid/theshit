use crate::fix::structs::Command;

static PATTERNS: &[&str] = &[
    "permission denied",
    "eacces",
    "pkg: insufficient privileges",
    "you cannot perform this operation unless you are root",
    "non-root users cannot",
    "operation not permitted",
    "not super-user",
    "superuser privilege",
    "root privilege",
    "this command has to be run under the root user.",
    "this operation requires root.",
    "requested operation requires superuser privilege",
    "must be run as root",
    "must run as root",
    "must be superuser",
    "must be root",
    "need to be root",
    "need root",
    "needs to be run as root",
    "only root can ",
    "you don't have access to the history db.",
    "authentication is required",
    "edspermissionerror",
    "you don't have write permissions",
    "use `sudo`",
    "sudorequirederror",
    "error: insufficient privileges",
    "updatedb: can not open a temporary file",
];
pub fn is_match(command: &Command) -> bool {
    if !command.parts().is_empty()
        && !command.parts().contains(&"&&".to_string())
        && command.parts()[0] == "sudo"
    {
        return false;
    }

    for pattern in PATTERNS {
        if command.output().stdout().to_lowercase().contains(pattern)
            || command.output().stderr().to_lowercase().contains(pattern)
        {
            return true;
        }
    }
    false
}

pub fn fix(command: &Command) -> String {
    if command.command().contains("&&") {
        format!("sudo sh -c '{}'", command.command().replace("sudo", ""))
    } else if command.command().contains('>') {
        format!("sudo sh -c \"{}\"", command.command().replace("\"", "\\\""))
    } else {
        format!("sudo {}", command.command())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::structs::{Command, CommandOutput};

    #[test]
    fn test_is_match_true() {
        let command = Command::new(
            "some_command".to_string(),
            CommandOutput::new(
                "some output".to_string(),
                "error: permission denied".to_string(),
            ),
        );
        assert!(is_match(&command));
    }

    #[test]
    fn test_is_match_with_sudo() {
        let command = Command::new(
            "sudo some_command".to_string(),
            CommandOutput::new("some output".to_string(), String::new()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_is_match_without_error() {
        let command = Command::new(
            "some_command".to_string(),
            CommandOutput::new("some output".to_string(), "No error".to_string()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_fix_simple_command() {
        let command = Command::new(
            "some_command".to_string(),
            CommandOutput::new(
                "some output".to_string(),
                "error: permission denied".to_string(),
            ),
        );
        assert_eq!(fix(&command), "sudo some_command");
    }
    #[test]
    fn test_fix_multiple_commands() {
        let command_with_and = Command::new(
            "some_command && another_command".to_string(),
            CommandOutput::new(
                "some output".to_string(),
                "error: permission denied".to_string(),
            ),
        );
        assert_eq!(
            fix(&command_with_and),
            "sudo sh -c 'some_command && another_command'"
        );
    }

    #[test]
    fn test_fix_command_with_redirection() {
        let command_with_redirection = Command::new(
            "some_command > output.txt".to_string(),
            CommandOutput::new(
                "some output".to_string(),
                "error: permission denied".to_string(),
            ),
        );
        assert_eq!(
            fix(&command_with_redirection),
            "sudo sh -c \"some_command > output.txt\""
        );
    }
}
