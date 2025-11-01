use crate::shells::generic;
use std::collections::HashMap;
use std::env;
use std::io::ErrorKind;
use std::path::Path;

pub fn get_shell_function(name: &str, path: &Path) -> String {
    format!(
        "
{name}() {{
    export SH_SHELL=bash;
    export SH_PREV_CMD=\"$(fc -ln -1)\";
    export SH_SHELL_ALIASES=\"$(alias)\";
    
    local SH_CMD;
    SH_CMD=$(
      command {} fix \"$@\"
    ) && eval \"$SH_CMD\";

    unset SH_SHELL_ALIASES;
    unset SH_PREV_CMD;
    unset SH_SHELL;
}}
    ",
        path.display()
    )
}

pub fn setup_alias(name: &str, program_path: &Path) -> std::io::Result<()> {
    let config_path = dirs::home_dir().ok_or(ErrorKind::NotFound)?.join(".bashrc");
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
        if let (Some(name), Some(mut value)) =
            (raw_alias.split('=').next(), raw_alias.split('=').nth(1))
        {
            let value_bytes = value.as_bytes();
            if (value_bytes[0] == b'"' && value_bytes[value.len() - 1] == b'"')
                || (value_bytes[0] == b'\'' && value_bytes[value.len() - 1] == b'\'')
            {
                value = &value[1..value.len() - 1];
            }
            aliases.insert(
                name.replacen("alias ", "", 1).to_string(),
                value.to_string(),
            );
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
        assert!(result.contains("export SH_SHELL=bash"));
    }

    #[test]
    fn test_get_aliases_empty() {
        let aliases = get_aliases();
        assert!(aliases.is_empty());
    }

    #[test]
    fn test_get_aliases_with_env() {
        unsafe {
            env::set_var("SH_SHELL_ALIASES", "alias ll='ls -l'\nalias la='ls -la'");
        }
        let aliases = get_aliases();
        assert_eq!(aliases.get("ll"), Some(&"ls -l".to_string()));
        assert_eq!(aliases.get("la"), Some(&"ls -la".to_string()));
        unsafe {
            env::remove_var("SH_SHELL_ALIASES");
        }
    }

    #[test]
    fn test_get_aliases_with_double_quotes() {
        unsafe {
            env::set_var("SH_SHELL_ALIASES", "alias grep=\"grep --color=auto\"");
        }
        let aliases = get_aliases();
        assert_eq!(
            aliases.get("grep"),
            Some(&"\"grep --color=auto".to_string())
        );
        unsafe {
            env::remove_var("SH_SHELL_ALIASES");
        }
    }

    #[test]
    fn test_get_aliases_with_single_quotes() {
        unsafe {
            env::set_var("SH_SHELL_ALIASES", "alias cls='clear'");
        }
        let aliases = get_aliases();
        assert_eq!(aliases.get("cls"), Some(&"clear".to_string()));
        unsafe {
            env::remove_var("SH_SHELL_ALIASES");
        }
    }
}
