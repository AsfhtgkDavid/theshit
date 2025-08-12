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
