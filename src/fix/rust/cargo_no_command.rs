use crate::fix::structs::Command;
use crate::misc;
use regex::Regex;

pub fn is_match(command: &Command) -> bool {
    command.output().stderr().contains("no such command")
        && command
            .output()
            .stderr()
            .contains("a command with a similar name exists:")
        && command.parts()[0] == "cargo"
}

pub fn fix(command: &Command) -> String {
    let broken = &command.parts()[1];
    let fix = Regex::new(r"a command with a similar name exists: `([^`]*)`")
        .unwrap()
        .captures(command.output().stderr())
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap();
    misc::replace_argument(command.command(), broken, fix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::structs::{Command, CommandOutput};

    #[test]
    fn test_is_match_true() {
        let command = Command::new(
            "cargo no_command".to_string(),
            CommandOutput::new(
                String::new(),
                "error: no such command `no_command`\n\
                     a command with a similar name exists: `new`"
                    .to_string(),
            ),
        );
        assert!(is_match(&command));
    }

    #[test]
    fn test_is_match_without_error() {
        let command = Command::new(
            "cargo build".to_string(),
            CommandOutput::new(String::new(), "Building project...".to_string()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_is_match_without_similar_command() {
        let command = Command::new(
            "cargo no_command".to_string(),
            CommandOutput::new(
                String::new(),
                "error: no such command `no_command`".to_string(),
            ),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_is_match_without_cargo() {
        let command = Command::new(
            "no_command".to_string(),
            CommandOutput::new(
                String::new(),
                "error: no such command `no_command`".to_string(),
            ),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_fix() {
        let command = Command::new(
            "cargo no_command".to_string(),
            CommandOutput::new(
                String::new(),
                "error: no such command `no_command`\n\
                     a command with a similar name exists: `new`"
                    .to_string(),
            ),
        );
        assert_eq!(fix(&command), "cargo new");
    }
}
