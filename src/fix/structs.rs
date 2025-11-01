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

    #[test]
    fn test_command_output_new() {
        let output = CommandOutput::new("test stdout".to_string(), "test stderr".to_string());
        assert_eq!(output.stdout(), "test stdout");
        assert_eq!(output.stderr(), "test stderr");
    }

    #[test]
    fn test_command_output_from_output() {
        let process_output = Output {
            status: std::process::ExitStatus::default(),
            stdout: b"test stdout".to_vec(),
            stderr: b"test stderr".to_vec(),
        };
        let output = CommandOutput::from(process_output);
        assert_eq!(output.stdout(), "test stdout");
        assert_eq!(output.stderr(), "test stderr");
    }

    #[test]
    fn test_command_new() {
        let cmd_output = CommandOutput::new("stdout".to_string(), "stderr".to_string());
        let command = Command::new("echo hello world".to_string(), cmd_output);
        assert_eq!(command.command(), "echo hello world");
        assert_eq!(command.parts(), &["echo", "hello", "world"]);
        assert_eq!(command.output().stdout(), "stdout");
        assert_eq!(command.output().stderr(), "stderr");
    }

    #[test]
    fn test_command_with_quoted_args() {
        let cmd_output = CommandOutput::new("".to_string(), "".to_string());
        let command = Command::new("echo 'hello world'".to_string(), cmd_output);
        assert_eq!(command.command(), "echo 'hello world'");
        assert_eq!(command.parts(), &["echo", "hello world"]);
    }

    #[test]
    fn test_command_empty() {
        let cmd_output = CommandOutput::new("".to_string(), "".to_string());
        let command = Command::new("".to_string(), cmd_output);
        assert_eq!(command.command(), "");
        assert!(command.parts().is_empty());
    }
}
