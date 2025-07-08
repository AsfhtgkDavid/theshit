mod python;
mod rust;
mod structs;

use crate::fix::rust::NativeRule;
use crate::fix::structs::CommandOutput;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, read};
use crossterm::style::Stylize;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{fs, io};
use structs::RawModeGuard;

pub fn fix_command(command: String, expand_command: String) -> io::Result<String> {
    let command_output = match get_command_output(expand_command) {
        Ok(output) => output,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => CommandOutput::new(
                "command not found".to_string(),
                "command not found".to_string(),
            ),
            ErrorKind::PermissionDenied => CommandOutput::new(
                "permission denied".to_string(),
                "permission denied".to_string(),
            ),
            _ => {
                eprintln!("{}: {}", "Error executing command".red(), e);
                return Err(e);
            }
        },
    };
    let command_struct = structs::Command::new(command, command_output);
    let active_rules_dir = dirs::config_dir()
        .ok_or(ErrorKind::NotFound)?
        .join("theshit/fix_rules/active");
    let mut fixed_commands: Vec<String> = vec![];
    let mut python_rules: Vec<PathBuf> = vec![];
    for rule in fs::read_dir(active_rules_dir)? {
        let rule = rule?;
        let path = rule.path();
        if path
            .file_name()
            .unwrap_or_else(|| panic!("Can't get get file name for {}", path.display()))
            .to_string_lossy()
            == "__pycache__"
        {
            continue;
        }
        match path.extension() {
            Some(extension) => match extension.to_string_lossy().as_ref() {
                "native" => {
                    let native_rule_name = match path.file_stem() {
                        Some(name) => name,
                        None => {
                            eprintln!("{}{}", "Failed to get stem for: ".yellow(), path.display());
                            continue;
                        }
                    };
                    let native_rule =
                        NativeRule::from_str(native_rule_name.to_string_lossy().as_ref());
                    match native_rule {
                        Ok(rule) => {
                            if let Some(fixed) = rule.fix_native(&command_struct) {
                                fixed_commands.push(fixed)
                            }
                        }
                        Err(_) => {
                            eprintln!(
                                "{}{}{}",
                                "Native rule '".yellow(),
                                native_rule_name.to_string_lossy(),
                                "' isn't supported".yellow()
                            );
                            continue;
                        }
                    }
                }
                "py" => python_rules.push(path),
                _ => {
                    eprintln!(
                        "{}{}{}",
                        "Rule type '".yellow(),
                        path.display(),
                        "' isn't supported".yellow()
                    )
                }
            },
            None => {
                eprintln!("{}{}", "Can't get extension for ".yellow(), path.display())
            }
        }
    }
    if !python_rules.is_empty() {
        match python::process_python_rules(&command_struct, python_rules) {
            Ok(commands) => fixed_commands.extend(commands),
            Err(e) => eprintln!("{}: {}", "Python rules processing failed".red(), e),
        }
    }
    Ok(choose_fixed_command(fixed_commands))
}

fn get_command_output(expand_command: String) -> io::Result<CommandOutput> {
    let split_command = shell_words::split(&expand_command)
        .map_err(|e| io::Error::other(format!("Failed to parse command: {e}")))?;
    let output = Command::new(&split_command[0])
        .args(&split_command[1..])
        .env("LANG", "C") // Set locale to C to avoid issues with rules that depend on locale
        .env("LC_ALL", "C")
        .output()?;
    Ok(CommandOutput::from(output))
}

fn choose_fixed_command(mut fixed_commands: Vec<String>) -> String {
    if fixed_commands.is_empty() {
        eprintln!(
            "{}: {}",
            "No fixed commands found".yellow(),
            "Exiting...".red()
        );
        std::process::exit(1);
    }

    let mut current_command = fixed_commands.first().unwrap();
    let mut current_index = 0;

    eprintln!();
    let _raw_mode_guard = RawModeGuard::new();
    let mut err = io::stderr();
    err.write_all(
        format!(
            "{} [{}/{}/{}/{}]",
            current_command,
            "enter".green(),
            "↑".cyan(),
            "↓".cyan(),
            "Ctrl+C".red()
        )
        .as_bytes(),
    )
    .expect("Failed to write to stderr");
    loop {
        match read() {
            Ok(event) => {
                if let Event::Key(KeyEvent {
                    code, modifiers, ..
                }) = event
                {
                    match (code, modifiers) {
                        (KeyCode::Up, _) => {
                            if fixed_commands.len() > 1 {
                                if current_index > 0 {
                                    current_index -= 1;
                                } else {
                                    current_index = fixed_commands.len() - 1;
                                }
                                current_command = fixed_commands.get(current_index).unwrap();
                                err.write_all(
                                    format!(
                                        "{} [{}/{}/{}/{}]",
                                        current_command,
                                        "enter".green(),
                                        "↑".cyan(),
                                        "↓".cyan(),
                                        "Ctrl+C".red()
                                    )
                                    .as_bytes(),
                                )
                                .expect("Failed to write to stderr");
                            }
                        }
                        (KeyCode::Down, _) => {
                            if fixed_commands.len() > 1 {
                                if current_index < fixed_commands.len() - 1 {
                                    current_index += 1;
                                } else {
                                    current_index = 0;
                                }
                                current_command = fixed_commands.get(current_index).unwrap();
                                err.write_all(
                                    format!(
                                        "{} [{}/{}/{}/{}]",
                                        current_command,
                                        "enter".green(),
                                        "↑".cyan(),
                                        "↓".cyan(),
                                        "Ctrl+C".red()
                                    )
                                    .as_bytes(),
                                )
                                .expect("Failed to write to stderr");
                            }
                        }
                        (KeyCode::Enter, _) => {
                            drop(_raw_mode_guard);
                            eprintln!();
                            eprintln!("{}: {}", "Selected command: ".green(), &current_command);
                            return fixed_commands.remove(current_index);
                        }
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            drop(_raw_mode_guard);
                            eprintln!();
                            eprintln!("{}: {}", "Exiting...".yellow(), "User interrupted".red());
                            std::process::exit(1);
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => {
                eprintln!("{}: {}", "Error reading input".red(), "Exiting...".yellow());
                drop(_raw_mode_guard);
                std::process::exit(1);
            }
        }
    }
}
