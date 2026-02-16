# The Shit [![Version][version-badge]][version-link] [![MIT License][license-badge]](LICENSE.md) [![codecov][codecov-badge]][codecov-link]

<p align="center">
  <img src="theshit-demo.gif" alt="theshit demo" />
</p>

A command-line utility that automatically fixes common shell command errors, inspired
by [thefuck](https://github.com/nvbn/thefuck). When you type a command incorrectly, just run `shit` and it will suggest
the correct command.

## Table of Contents

- [Installation](#installation)
    - [From crates.io](#from-cratesio)
    - [Build from source](#build-from-source)
- [Usage](#usage)
    - [Setup](#setup)
    - [Basic usage](#basic-usage)
- [Supported Shells](#supported-shells)
- [Built-in Rules](#built-in-rules)
- [Custom Rules](#custom-rules)
    - [Writing Python rules](#writing-python-rules)
    - [Disabling rules](#disabling-rules)
- [Configuration](#configuration)
- [Tricks and Tips](#tricks-and-tips)
- [Contributing](#contributing)
- [Bug Reports](#bug-reports)
- [License](#license)

## Installation

### From crates.io

```bash
cargo install theshit
```

### Build from source

```bash
git clone https://github.com/AsfhtgkDavid/theshit.git
cd theshit
cargo build --release --no-default-features
```

The binary will be available at `target/release/theshit`. You can install it to your system with
`cargo install --path .` or copy it to a directory in your `$PATH`.

## Usage

### Setup

First, set up the alias in your shell:

```bash
# This will add the necessary alias to your shell configuration and create the base rules
theshit setup
```

Or specify a custom alias name:

```bash
theshit setup myfix
```

After setup, restart your shell or source your configuration file.

### Basic usage

When a command fails, just type `shit` (or your custom alias):

```bash
$ sl
bash: sl: command not found
$ shit
ls [enter/↑/↓/Ctrl+C]
```

The tool will suggest corrections. Use:

- **Enter** to execute the selected command
- **↑/↓** to navigate between suggestions
- **Ctrl+C** to cancel

## Supported Shells

- **Bash**
- **Zsh**
- **Fish**

## Built-in Rules

### Native Rules (Rust)

| Rule               | Description                                                   | Example                                        |
|--------------------|---------------------------------------------------------------|------------------------------------------------|
| `sudo`             | Adds `sudo` to commands that failed with permission errors    | `mkdir /etc/config` → `sudo mkdir /etc/config` |
| `to_cd`            | Fixes typos in the `cd` command                               | `cs /home` → `cd /home`                        |
| `unsudo`           | Removes `sudo` from commands that shouldn't be run as root    | `sudo npm install` → `npm install`             |
| `mkdir_p`          | Adds `-p` flag to `mkdir` when parent directories don't exist | `mkdir a/b/c` → `mkdir -p a/b/c`               |
| `cargo_no_command` | Fixes cargo subcommand typos                                  | `cargo biuld` → `cargo build`                  |

### Permission Patterns

The `sudo` rule recognizes these error patterns:

- "permission denied"
- "eacces"
- "operation not permitted"
- "must be run as root"
- "authentication is required"
- And many more...

## Custom Rules

### Writing Python rules

Create a Python file in `~/.config/theshit/fix_rules/active/` with the following structure:

```python
def match(command: str, stdout: str, stderr: str) -> bool:
    """
    Determine if this rule should be applied to the failed command.
    
    Args:
        command: The original command that failed
        stdout: Standard output from the failed command
        stderr: Standard error from the failed command
    
    Returns:
        True if this rule should fix the command, False otherwise
    """
    return "your condition here"

def fix(command: str, stdout: str, stderr: str) -> str:
    """
    Generate the corrected command.
    
    Args:
        command: The original command that failed
        stdout: Standard output from the failed command
        stderr: Standard error from the failed command
    
    Returns:
        The corrected command string
    """
    return "your fixed command here"
```

#### Example: Git branch typo rule

```python
# ~/.config/theshit/fix_rules/active/git_branch_typo.py
import re

def match(command: str, stdout: str, stderr: str) -> bool:
    return (command.startswith("git") and 
            "did you mean" in stderr and
            "branch" in command)

def fix(command: str, stdout: str, stderr: str) -> str:
    # Extract suggested branch name from git error message
    match = re.search(r"did you mean '([^']+)'", stderr)
    if match:
        suggested_branch = match.group(1)
        return re.sub(r"branch \S+", f"branch {suggested_branch}", command)
    return command
```

### Disabling rules

To disable a rule temporarily, add `.bak` to its filename:

```bash
# Disable the sudo rule
mv ~/.config/theshit/fix_rules/active/sudo.native ~/.config/theshit/fix_rules/active/sudo.native.bak

# Disable a Python rule
mv ~/.config/theshit/fix_rules/active/my_rule.py ~/.config/theshit/fix_rules/active/my_rule.py.bak
```

To permanently disable a rule, move it to the additional rules directory:

```bash
mv ~/.config/theshit/fix_rules/active/sudo.native ~/.config/theshit/fix_rules/additional/
```

## Configuration

Configuration files are stored in `~/.config/theshit/`:

```
~/.config/theshit/
├── fix_rules/
│   ├── active/          # Rules that are currently enabled
│   │   ├── sudo.native
│   │   ├── to_cd.native
│   │   └── custom_rule.py
│   └── additional/      # Disabled rules
│       └── disabled_rule.py
```

## Tricks and Tips

### 1. Shell Integration

For the best experience, add this to your shell configuration:

**Bash (~/.bashrc):**

```bash
# Run theshit on double ESC
bind '"\e\e": "theshit\n"'
```

**Zsh (~/.zshrc):**

```zsh
# Run theshit on double ESC
bindkey -s '\e\e' 'theshit\n'
```

### 2. Alias Expansion

The tool automatically expands aliases before processing, so if you have:

```bash
alias ll='ls -la'
```

And you run `ll /nonexistent`, the tool will process `ls -la /nonexistent`.

### 3. Custom Alias Names

You can use any alias name:

```bash
theshit setup fix
theshit setup oops
theshit setup dammit
```

### 4. Environment Variables

The tool sets these environment variables during execution:

- `SH_SHELL`: Current shell (bash/zsh)
- `SH_PREV_CMD`: The previous command that failed
- `SH_SHELL_ALIASES`: Available shell aliases

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to this project.

For a quick overview of the project architecture, you can
use  [Code2tutorial](https://code2tutorial.com/tutorial/711fc304-35d9-4c8a-a743-c3ddf1c3d09e/index.md).

## Bug Reports

If you encounter a bug or have a feature request, please [open an issue](https://github.com/AsfhtgkDavid/theshit/issues)
with:

1. **Environment information:**
    - OS and version
    - Shell and version
    - Rust version (if building from source)

2. **Steps to reproduce:**
    - The original command that failed
    - The error message
    - Expected vs actual behavior

3. **Relevant logs:**
    - Include any error messages from `theshit`
    - Shell configuration if relevant

4. **For rule-related issues:**
    - Output of the failing command
    - Contents of `~/.config/theshit/fix_rules/active/` if using custom rules

### Common Issues

**Issue: Custom Python rules not working**

- Check that the rule file has both `match()` and `fix()` functions
- Ensure the file is in the `active` directory
- Verify Python syntax with `python -m py_compile your_rule.py`

**Issue: Alias not found after setup**

- Restart your shell or run `source ~/.bashrc` / `source ~/.zshrc`
- Check that the alias was added to the correct configuration file

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

[version-badge]: https://img.shields.io/crates/v/theshit

[version-link]: https://crates.io/crates/theshit

[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg

[codecov-badge]: https://codecov.io/github/asfhtgkdavid/theshit/graph/badge.svg?token=016D8DZWLG

[codecov-link]: https://codecov.io/github/asfhtgkdavid/theshit
