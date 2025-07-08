use crate::misc;
use crossterm::terminal;
use std::process::Output;

pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> Self {
        terminal::enable_raw_mode().expect("Failed to enable raw mode");
        RawModeGuard
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Failed to disable raw mode");
    }
}

pub struct CommandOutput {
    stdout: String,
    stderr: String,
}

impl CommandOutput {
    pub fn new(stdout: String, stderr: String) -> Self {
        CommandOutput { stdout, stderr }
    }

    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    pub fn stderr(&self) -> &str {
        &self.stderr
    }
}

impl From<Output> for CommandOutput {
    fn from(output: Output) -> Self {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        CommandOutput { stdout, stderr }
    }
}

pub struct Command {
    command: String,
    parts: Vec<String>,
    output: CommandOutput,
}

impl Command {
    pub fn new(command: String, output: CommandOutput) -> Self {
        let parts = misc::split_command(&command);
        Command {
            command,
            parts,
            output,
        }
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    pub fn output(&self) -> &CommandOutput {
        &self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::terminal;

    #[test]
    fn raw_mode_guard_enables_raw_mode_on_creation() {
        let _guard = RawModeGuard::new();
        assert!(terminal::is_raw_mode_enabled().unwrap());
    }

    #[test]
    fn raw_mode_guard_disables_raw_mode_on_drop() {
        {
            let _guard = RawModeGuard::new();
            assert!(terminal::is_raw_mode_enabled().unwrap());
        }
        assert!(!terminal::is_raw_mode_enabled().unwrap());
    }
}
