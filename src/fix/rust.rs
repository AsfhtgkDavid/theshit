mod cargo_no_command;
mod mkdir_p;
mod sudo;
mod to_cd;
mod unsudo;

use super::structs::Command;
use strum::EnumString;

#[derive(EnumString, Debug)]
pub enum NativeRule {
    #[strum(serialize = "sudo")]
    Sudo,
    #[strum(serialize = "to_cd")]
    ToCd,
    #[strum(serialize = "unsudo")]
    Unsudo,
    #[strum(serialize = "mkdir_p")]
    MkdirP,
    #[strum(serialize = "cargo_no_command")]
    CargoNoCommand,
}

impl NativeRule {
    pub fn fix_native(self, command: &Command) -> Option<String> {
        match self {
            NativeRule::Sudo => Self::match_and_fix(sudo::is_match, sudo::fix, command),
            NativeRule::ToCd => Self::match_and_fix(to_cd::is_match, to_cd::fix, command),
            NativeRule::Unsudo => Self::match_and_fix(unsudo::is_match, unsudo::fix, command),
            NativeRule::MkdirP => Self::match_and_fix(mkdir_p::is_match, mkdir_p::fix, command),
            NativeRule::CargoNoCommand => {
                Self::match_and_fix(cargo_no_command::is_match, cargo_no_command::fix, command)
            }
        }
    }

    fn match_and_fix(
        match_function: fn(&Command) -> bool,
        fix_function: fn(&Command) -> String,
        command: &Command,
    ) -> Option<String> {
        if match_function(command) {
            Some(fix_function(command))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::structs::CommandOutput;
    use std::str::FromStr;

    #[test]
    fn test_native_rule_from_str_sudo() {
        let rule = NativeRule::from_str("sudo");
        assert!(rule.is_ok());
        assert!(matches!(rule.unwrap(), NativeRule::Sudo));
    }

    #[test]
    fn test_native_rule_from_str_to_cd() {
        let rule = NativeRule::from_str("to_cd");
        assert!(rule.is_ok());
        assert!(matches!(rule.unwrap(), NativeRule::ToCd));
    }

    #[test]
    fn test_native_rule_from_str_unsudo() {
        let rule = NativeRule::from_str("unsudo");
        assert!(rule.is_ok());
        assert!(matches!(rule.unwrap(), NativeRule::Unsudo));
    }

    #[test]
    fn test_native_rule_from_str_mkdir_p() {
        let rule = NativeRule::from_str("mkdir_p");
        assert!(rule.is_ok());
        assert!(matches!(rule.unwrap(), NativeRule::MkdirP));
    }

    #[test]
    fn test_native_rule_from_str_cargo_no_command() {
        let rule = NativeRule::from_str("cargo_no_command");
        assert!(rule.is_ok());
        assert!(matches!(rule.unwrap(), NativeRule::CargoNoCommand));
    }

    #[test]
    fn test_native_rule_from_str_invalid() {
        let rule = NativeRule::from_str("invalid_rule");
        assert!(rule.is_err());
    }

    #[test]
    fn test_fix_native_sudo() {
        let command = Command::new(
            "some_command".to_string(),
            CommandOutput::new("".to_string(), "permission denied".to_string()),
        );
        let rule = NativeRule::Sudo;
        let result = rule.fix_native(&command);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "sudo some_command");
    }

    #[test]
    fn test_fix_native_to_cd() {
        let command = Command::new(
            "cs /some/directory".to_string(),
            CommandOutput::new("".to_string(), "".to_string()),
        );
        let rule = NativeRule::ToCd;
        let result = rule.fix_native(&command);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "cd /some/directory");
    }

    #[test]
    fn test_fix_native_no_match() {
        let command = Command::new(
            "ls -l".to_string(),
            CommandOutput::new("".to_string(), "".to_string()),
        );
        let rule = NativeRule::Sudo;
        let result = rule.fix_native(&command);
        assert!(result.is_none());
    }
}
