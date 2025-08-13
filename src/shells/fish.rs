use crate::misc;
use crate::shells::generic;
use std::collections::HashMap;
use std::env;
use std::io::ErrorKind;
use std::path::Path;

pub fn get_shell_function(name: &str, path: &Path) -> String {
    format!(
        "
function {name} -d \"Correct your previous command\"
    set -lx SH_SHELL fish
    set -lx SH_PREV_CMD \"$history[1]\"
    set -lx SH_SHELL_ALIASES (alias)
    
    set -l SH_CMD;
    command {} fix $argv | read -l SH_CMD;


    if test -n \"$SH_CMD\"
        eval \"$SH_CMD\";
    end
    set -e SH_SHELL_ALIASES;
    set -e SH_PREV_CMD;
    set -e SH_SHELL;
end
    ",
        path.display()
    )
}

pub fn setup_alias(name: &str, program_path: &Path) -> std::io::Result<()> {
    let config_path = dirs::config_dir()
        .ok_or(ErrorKind::NotFound)?
        .join("fish/config.fish");
    generic::setup_alias(
        format!("{} alias {} | source", program_path.display(), name),
        config_path.as_path(),
    )
}

pub fn get_aliases() -> HashMap<String, String> {
    let raw_aliases = env::var("SH_SHELL_ALIASES").unwrap_or(String::from(""));
    let split_raw_aliases = raw_aliases.split('\n');
    let mut aliases: HashMap<String, String> = HashMap::new();
    for raw_alias in split_raw_aliases {
        if !raw_alias.contains("alias ") || raw_alias.is_empty() {
            continue;
        }
        let parts = misc::split_command(raw_alias);
        if parts.len() != 3 || parts[0] != "alias" {
            continue;
        }
        aliases.insert(parts[1].to_string(), parts[2].to_string());
    }
    aliases
}
