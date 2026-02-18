use crate::fix::structs::Command;

static PATTERNS: &[&str] = &["you cannot perform this operation as root"];
pub fn is_match(command: &Command) -> bool {
    if !command.parts().is_empty() && command.parts()[0] != "sudo" {
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

pub fn fix(command: &Command) -> Result<String, crate::errors::TheShitError> {
    Ok(command.parts()[1..].join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::structs::{Command, CommandOutput};

    #[test]
    fn test_is_match_true() {
        let command = Command::new(
            "sudo some_command".to_string(),
            CommandOutput::new(
                "some output".to_string(),
                "you cannot perform this operation as root".to_string(),
            ),
        );
        assert!(is_match(&command));
    }

    #[test]
    fn test_is_match_without_sudo() {
        let command = Command::new(
            "some_command".to_string(),
            CommandOutput::new("some output".to_string(), String::new()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_is_match_without_error() {
        let command = Command::new(
            "sudo some_command".to_string(),
            CommandOutput::new("some output".to_string(), "No error".to_string()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_fix() {
        let command = Command::new(
            "sudo some_command".to_string(),
            CommandOutput::new(String::new(), String::new()),
        );
        let fixed_command = fix(&command);
        assert_eq!(fixed_command, "some_command");
    }
}
