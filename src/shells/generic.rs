use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Result, Write, stdin};
use std::path::Path;

pub fn setup_alias(setup_command: String, config_path: &Path) -> Result<()> {
    let mut config_file = match OpenOptions::new().read(true).append(true).open(config_path) {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                println!(
                    "Can't find config file({}), do you want to create it? (Y/n)",
                    config_path.display()
                );
                let mut input = String::new();
                stdin()
                    .read_line(&mut input)
                    .expect("Error getting user input");
                if input.trim().eq_ignore_ascii_case("y") || input.trim().is_empty() {
                    File::create(config_path).expect("Failed to create config file")
                } else {
                    return Err(ErrorKind::NotFound.into());
                }
            }
            _ => {
                return Err(error);
            }
        },
    };

    let mut config_content = String::new();

    config_file.read_to_string(&mut config_content)?;
    if config_content.contains(&setup_command) {
        return Err(ErrorKind::AlreadyExists.into());
    }

    writeln!(config_file, "{setup_command}")
}
