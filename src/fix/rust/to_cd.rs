use crate::fix::structs::Command;
use crate::misc;

pub fn is_match(command: &Command) -> bool {
    if !command.parts().is_empty()
        && (command.parts()[0] == "cd"
            || command.parts()[0].len() > 3
            || command.parts()[0].len() < 2)
    {
        return false;
    }
    misc::string_similarity(&command.parts()[0], "cd") >= 0.5
}

pub fn fix(command: &Command) -> String {
    "cd ".to_string() + &command.parts()[1..].join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::structs::{Command, CommandOutput};

    #[test]
    fn test_is_match_true() {
        let command = Command::new(
            "cs /some/directory".to_string(),
            CommandOutput::new(String::new(), String::new()),
        );
        assert!(is_match(&command));
    }

    #[test]
    fn test_is_match_already_cd() {
        let command = Command::new(
            "cd /some/directory".to_string(),
            CommandOutput::new(String::new(), String::new()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_is_match_not_similar_cd() {
        let command = Command::new(
            "ls -l".to_string(),
            CommandOutput::new(String::new(), String::new()),
        );
        assert!(!is_match(&command));
    }

    #[test]
    fn test_fix() {
        let command = Command::new(
            "cs /some/directory".to_string(),
            CommandOutput::new(String::new(), String::new()),
        );
        let fixed_command = fix(&command);
        assert_eq!(fixed_command, "cd /some/directory");
    }
}
