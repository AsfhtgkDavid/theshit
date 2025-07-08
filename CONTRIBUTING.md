# Contributing to The Shit

Thank you for your interest in contributing to theshit! This document provides guidelines and information for
contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Contributing Guidelines](#contributing-guidelines)
    - [Bug Reports](#bug-reports)
    - [Feature Requests](#feature-requests)
    - [Code Contributions](#code-contributions)
- [Writing Native Rules](#writing-native-rules)
- [Adding New Shells](#adding-new-shells)
- [Testing](#testing)
- [Code Style](#code-style)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

This project follows a standard code of conduct. Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your changes
4. Make your changes
5. Test your changes
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Python 3.8 or later (for Python rule development)
- Git

### Building

```bash
git clone https://github.com/yourusername/theshit.git
cd theshit
cargo build
```

## Project Structure

```
theshit/
├── src/
│   ├── cli.rs              # Command-line interface
│   ├── fix/
│   │   ├── rust/           # Native Rust rules
│   │   │   ├── sudo.rs
│   │   │   ├── to_cd.rs
│   │   │   └── ...
│   │   ├── python.rs       # Python rule processor
│   │   ├── rust.rs         # Native rule processor
│   │   └── structs.rs      # Core data structures
│   ├── shells/
│   │   ├── bash.rs         # Bash shell support
│   │   ├── zsh.rs          # Zsh shell support
│   │   ├── enums.rs        # Shell enumeration
│   │   └── helpers.rs      # Shell detection utilities
│   ├── main.rs             # Main entry point
│   └── misc.rs             # Utility functions
├── assets/
│   ├── active/             # Default active rules
│   └── additional/         # Additional rule examples
├── Cargo.toml
└── README.md
```

## Contributing Guidelines

### Bug Reports

When reporting bugs, please include:

1. **Environment details:**
    - Operating system and version
    - Shell and version
    - Rust version
    - theshit version

2. **Reproduction steps:**
    - Original command that failed
    - Error message received
    - Expected vs actual behavior of theshit

3. **Additional context:**
    - Shell configuration
    - Custom rules (if any)
    - Error logs

### Feature Requests

When requesting features:

1. **Clear description** of the feature
2. **Use case** - why is this feature needed?
3. **Proposed implementation** (if you have ideas)
4. **Examples** of how it would work

### Code Contributions

1. **Follow the existing code style**
2. **Write tests** for new functionality
3. **Update documentation** as needed
4. **Ensure all tests pass**
5. **Keep commits focused** and atomic

## Writing Native Rules

Native rules are written in Rust and provide the best performance. Here's how to add a new native rule:

### Step 1: Create the Rule Module

Create a new file in `src/fix/rust/` (e.g., `new_rule.rs`):

```rust
use crate::fix::structs::Command;

pub fn is_match(command: &Command) -> bool {
    // Your matching logic here
    // Return true if this rule should apply to the command

    // Example: Check if command contains specific text
    command.output().stderr().contains("specific error pattern")
        && command.parts()[0] == "your_command"
}

pub fn fix(command: &Command) -> String {
    // Your fix logic here
    // Return the corrected command string

    // Example: Simple replacement
    command.command().replace("wrong_text", "correct_text")
}
```

### Step 2: Write the Rule tests

Create a test module in the same file to ensure your rule works correctly:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::structs::{Command, CommandOutput};

    #[test]
    fn test_rule_matches() {
        let output = CommandOutput::new("".to_string(), "specific error pattern".to_string());
        let command = Command::new("your_command arg1 arg2".to_string(), output);
        assert!(is_match(&command));
    }

    #[test]
    fn test_rule_fix() {
        let output = CommandOutput::new("".to_string(), "specific error pattern".to_string());
        let command = Command::new("your_command wrong_text".to_string(), output);
        assert_eq!(fix(&command), "your_command correct_text");
    }
}
```

### Step 3: Add to the Enum

In `src/fix/rust.rs`, add your rule to the `NativeRule` enum:

```rust
#[derive(EnumString, Debug)]
pub enum NativeRule {
    // ... existing rules ...
    #[strum(serialize = "new_rule")]
    NewRule,
}
```

### Step 4: Add to the Match Implementation

In the same file, add your rule to the `fix_native` method:

```rust
impl NativeRule {
    pub fn fix_native(self, command: &Command) -> Option<String> {
        match self {
            // ... existing rules ...
            NativeRule::NewRule => Self::match_and_fix(new_rule::is_match, new_rule::fix, command),
        }
    }
}
```

### Step 5: Import the Module

Add the module import at the top of `src/fix/rust.rs`:

```rust
mod new_rule;
```

### Step 6: Create the Asset File

Create a corresponding file in `assets/active/new_rule.native` with a description:

```
Brief description of what this rule does. Example: "Fixes typos in git commands by suggesting similar commands."
```

### Example: Complete Rule Implementation

Here's a complete example for fixing `git` typos:

```rust
// src/fix/rust/sudo.rs
use crate::fix::structs::Command;

static PATTERNS: &[&str] = &[
    "permission denied",
    //    ...
];
pub fn is_match(command: &Command) -> bool {
    if !command.parts().is_empty()
        && !command.parts().contains(&"&&".to_string())
        && command.parts()[0] == "sudo"
    {
        return false;
    }

    for pattern in PATTERNS {
        if command.output().stdout().to_lowercase().contains(pattern)
            || command.output().stderr().to_lowercase().contains(pattern)
        {
            return true;
        }
    }
    false
}

pub fn fix(command: &Command) -> String {
    if command.command().contains("&&") {
        format!("sudo sh -c '{}'", command.command().replace("sudo", ""))
    } else if command.command().contains('>') {
        format!("sudo sh -c \"{}\"", command.command().replace("\"", "\\\""))
    } else {
        format!("sudo {}", command.command())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::structs::{Command, CommandOutput};

    #[test]
    fn test_is_match_true() {
        let command = Command::new(
            "some_command".to_string(),
            CommandOutput::new(
                "some output".to_string(),
                "error: permission denied".to_string(),
            ),
        );
        assert!(is_match(&command));
    }

    #[test]
    fn test_fix_simple_command() {
        let command = Command::new(
            "some_command".to_string(),
            CommandOutput::new(
                "some output".to_string(),
                "error: permission denied".to_string(),
            ),
        );
        assert_eq!(fix(&command), "sudo some_command");
    }
}

```

## Adding New Shells

To add support for a new shell:

### Step 1: Create Shell Module

Create a new file in `src/shells/` (e.g., `fish.rs`):

```rust
use crate::shells::generic;
use std::collections::HashMap;
use std::env;
use std::io::{ErrorKind, Result};
use std::path::Path;

pub fn get_shell_function(name: &str) -> String {
    format!(
        "
function {}
    set -x SH_SHELL fish
    set -x SH_PREV_CMD (history | head -1)
    
    set SH_CMD (
        command ./target/debug/theshit fix $argv
    )
    and eval \"$SH_CMD\"
    
    set -e SH_PREV_CMD
    set -e SH_SHELL
end
        ",
        name
    )
}

pub fn setup_alias(name: &str, program_path: &Path) -> Result<()> {
    let config_path = dirs::home_dir()
        .ok_or(ErrorKind::NotFound)?
        .join(".config/fish/config.fish");
    generic::setup_alias(
        format!("eval ({} alias {})", program_path.display(), name),
        config_path.as_path(),
    )
}

pub fn get_aliases() -> HashMap<String, String> {
    // Implementation to parse fish aliases
    // This will depend on how fish stores aliases
    HashMap::new()
}
```

### Step 2: Add to Shell Enum

In `src/shells/enums.rs`, add the new shell:

```rust
#[derive(EnumString, Debug)]
pub enum Shell {
    #[strum(serialize = "bash")]
    Bash,
    #[strum(serialize = "zsh")]
    Zsh,
    #[strum(serialize = "fish")]
    Fish,
}
```

### Step 3: Implement Shell Methods

Add the implementation in the same file:

```rust
impl Shell {
    pub fn get_shell_function(&self, name: &str) -> String {
        match self {
            Shell::Bash => bash::get_shell_function(name),
            Shell::Zsh => zsh::get_shell_function(name),
            Shell::Fish => fish::get_shell_function(name),
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
```

### Step 5: Update Shell Detection

In `src/shells/helpers.rs`, ensure the shell detection logic can identify your shell:

```rust
fn get_current_shell_by_process() -> Option<Shell> {
    // The existing logic should work if the shell executable name matches
    // the enum serialize string
}
```

## Code Style

- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common issues
- Follow Rust naming conventions
- Add documentation comments for public APIs
- Keep functions focused and small

## Commit Messages

Use clear, descriptive commit messages, following the conventional commits format:

```
feat: add support for fish shell
fix: handle edge case in sudo rule
docs: update README with new examples
test: add unit tests for git_typo rule
```

Prefix types:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `test:` - Test additions/changes
- `refactor:` - Code refactoring
- `perf:` - Performance improvements

## Pull Request Process

1. **Update documentation** if needed
2. **Add tests** for new functionality
3. **Ensure all tests pass**: `cargo test`
4. **Check formatting**: `cargo fmt`
5. **Check linting**: `cargo clippy`
6. **Write clear PR description**:
    - What changes were made
    - Why they were made
    - How to test them
7. **Link related issues**
8. **Be responsive** to code review feedback

## Development Tips

1. **Use debug builds** during development for better error messages
2. **Test with multiple shells** if making shell-related changes
3. **Use `SH_SHELL` environment variable** to override shell detection
4. **Test edge cases** like empty commands, special characters, etc.
5. **Profile performance** for rules that might be slow

## Questions?

If you have questions about contributing:

1. Check existing issues and discussions
2. Open a new issue with the "question" label
3. Reach out to maintainers

Thank you for contributing to theshit!