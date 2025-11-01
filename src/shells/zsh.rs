use crate::shells::generic;
use std::collections::HashMap;
use std::env;
use std::io::{ErrorKind, Result};
use std::path::Path;

pub fn get_shell_function(name: &str, path: &Path) -> String {
    format!(
        "
{name}() {{
    export SH_SHELL=zsh;
    SH_PREV_CMD=\"$(fc -ln -1)\";
    export SH_PREV_CMD;
    SH_SHELL_ALIASES=$(alias);
    export SH_SHELL_ALIASES;

    SH_CMD=$(
      {} fix $@
    ) && eval \"$SH_CMD\";

    unset SH_SHELL_ALIASES;
    unset SH_PREV_CMD;
    unset SH_SHELL;
}}
    ",
        path.display()
    )
    .trim()
    .to_string()
}

pub fn setup_alias(name: &str, program_path: &Path) -> Result<()> {
    let config_path = dirs::home_dir().ok_or(ErrorKind::NotFound)?.join(".zshrc");
    generic::setup_alias(
        format!("eval $( {} alias {})", program_path.display(), name),
        config_path.as_path(),
    )
}

pub fn get_aliases() -> HashMap<String, String> {
    let raw_aliases = env::var("SH_SHELL_ALIASES").unwrap_or(String::from(""));
    let split_raw_aliases = raw_aliases.split('\n');
    let mut aliases: HashMap<String, String> = HashMap::new();
    for raw_alias in split_raw_aliases {
        if !raw_alias.contains('=') || raw_alias.is_empty() {
            continue;
        }
        if let Some((name, mut value)) = raw_alias.split_once('=') {
            let value_bytes = value.as_bytes();
            if (value_bytes[0] == b'"' && value_bytes[value.len() - 1] == b'"')
                || (value_bytes[0] == b'\'' && value_bytes[value.len() - 1] == b'\'')
            {
                value = &value[1..value.len() - 1];
            }
            aliases.insert(name.to_string(), value.to_string());
        }
    }
    aliases
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_shell_function_contains_name() {
        let path = PathBuf::from("/usr/bin/theshit");
        let result = get_shell_function("shit", &path);
        assert!(result.contains("shit()"));
    }

    #[test]
    fn test_get_shell_function_contains_path() {
        let path = PathBuf::from("/usr/bin/theshit");
        let result = get_shell_function("shit", &path);
        assert!(result.contains("/usr/bin/theshit"));
    }

    #[test]
    fn test_get_shell_function_exports_shell_type() {
        let path = PathBuf::from("/usr/bin/theshit");
        let result = get_shell_function("shit", &path);
        assert!(result.contains("export SH_SHELL=zsh"));
    }

    #[test]
    fn test_get_aliases_empty() {
        let aliases = get_aliases();
        assert!(aliases.is_empty());
    }

    #[test]
    fn test_get_aliases_with_double_quotes() {
        unsafe {
            env::set_var("SH_SHELL_ALIASES", "grep=\"grep --color=auto\"");
        }
        let aliases = get_aliases();
        assert_eq!(aliases.get("grep"), Some(&"grep --color=auto".to_string()));
        unsafe {
            env::remove_var("SH_SHELL_ALIASES");
        }
    }

    #[test]
    fn test_get_aliases_with_single_quotes() {
        unsafe {
            env::set_var("SH_SHELL_ALIASES", "cls='clear'");
        }
        let aliases = get_aliases();
        assert_eq!(aliases.get("cls"), Some(&"clear".to_string()));
        unsafe {
            env::remove_var("SH_SHELL_ALIASES");
        }
    }
}
