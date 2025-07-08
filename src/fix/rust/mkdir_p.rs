use crate::fix::structs::Command;
use regex::Regex;

pub fn is_match(command: &Command) -> bool {
    command.parts().contains(&"mkdir".to_string())
        && !command.parts().contains(&"-p".to_string())
        && (command
            .output()
            .stdout()
            .contains("No such file or directory")
            || command
                .output()
                .stderr()
                .contains("No such file or directory"))
}

pub fn fix(command: &Command) -> String {
    Regex::new(r"\bmkdir (.*)")
        .unwrap()
        .replace(command.command(), "mkdir -p $1")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::structs::{Command, CommandOutput};

    #[test]
    fn test_is_match_true() {
        let command = Command::new(
            "mkdir some_directory".to_string(),
            CommandOutput::new(String::new(), "No such file or directory".to_string()),
        );
        assert!(is_match(&command));
    }

    #[test]
    fn test_is_match_with_flag_p() {
        let command = Command::new(
            "mkdir -p some_directory".to_string(),
            CommandOutput::new(String::new(), "No such file or directory".to_string()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_is_match_without_error() {
        let command = Command::new(
            "mkdir some_directory".to_string(),
            CommandOutput::new(String::new(), "Directory created successfully".to_string()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_is_match_without_mkdir() {
        let command = Command::new(
            "ls -l".to_string(),
            CommandOutput::new(String::new(), "Listing files".to_string()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_fix() {
        let command = Command::new(
            "mkdir some_directory".to_string(),
            CommandOutput::new(String::new(), "No such file or directory".to_string()),
        );
        assert_eq!(fix(&command), "mkdir -p some_directory");
    }
}
