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
