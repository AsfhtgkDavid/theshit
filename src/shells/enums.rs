use super::{bash, fish, zsh};
use std::collections::HashMap;
use std::io::Result;
use std::path::Path;
use strum::EnumString;

#[derive(EnumString, Debug)]
pub enum Shell {
    #[strum(serialize = "bash")]
    Bash,

    #[strum(serialize = "zsh")]
    Zsh,

    #[strum(serialize = "fish")]
    Fish,
}

impl Shell {
    pub fn get_shell_function(&self, name: &str, path: &Path) -> String {
        match self {
            Shell::Bash => bash::get_shell_function(name, path),
            Shell::Zsh => zsh::get_shell_function(name, path),
            Shell::Fish => fish::get_shell_function(name, path),
        }
    }
    pub fn setup_alias(&self, name: &str, path: &Path) -> Result<()> {
        match self {
            Shell::Bash => bash::setup_alias(name, path),
            Shell::Zsh => zsh::setup_alias(name, path),
            Shell::Fish => fish::setup_alias(name, path),
        }
    }
    pub fn get_aliases(&self) -> HashMap<String, String> {
        match self {
            Shell::Bash => bash::get_aliases(),
            Shell::Zsh => zsh::get_aliases(),
            Shell::Fish => fish::get_aliases(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn test_shell_from_str_bash() {
        let shell = Shell::from_str("bash");
        assert!(shell.is_ok());
        assert!(matches!(shell.unwrap(), Shell::Bash));
    }

    #[test]
    fn test_shell_from_str_zsh() {
        let shell = Shell::from_str("zsh");
        assert!(shell.is_ok());
        assert!(matches!(shell.unwrap(), Shell::Zsh));
    }

    #[test]
    fn test_shell_from_str_fish() {
        let shell = Shell::from_str("fish");
        assert!(shell.is_ok());
        assert!(matches!(shell.unwrap(), Shell::Fish));
    }

    #[test]
    fn test_shell_from_str_invalid() {
        let shell = Shell::from_str("invalid");
        assert!(shell.is_err());
    }

    #[test]
    fn test_get_shell_function_bash() {
        let shell = Shell::Bash;
        let path = PathBuf::from("/usr/bin/theshit");
        let result = shell.get_shell_function("shit", &path);
        assert!(result.contains("shit()"));
        assert!(result.contains("SH_SHELL=bash"));
    }

    #[test]
    fn test_get_shell_function_zsh() {
        let shell = Shell::Zsh;
        let path = PathBuf::from("/usr/bin/theshit");
        let result = shell.get_shell_function("shit", &path);
        assert!(result.contains("shit()"));
        assert!(result.contains("SH_SHELL=zsh"));
    }

    #[test]
    fn test_get_shell_function_fish() {
        let shell = Shell::Fish;
        let path = PathBuf::from("/usr/bin/theshit");
        let result = shell.get_shell_function("shit", &path);
        assert!(result.contains("function shit"));
        assert!(result.contains("SH_SHELL fish"));
    }
}
