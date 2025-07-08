//! TheShit - A command-line utility to fix and enhance shell commands.
//!
//! See [README](https://github.com/AsfhtgkDavid/theshit) for more details.
mod cli;
mod fix;
mod misc;
mod shells;

use clap::Parser;
use cli::{Cli, Command};
use crossterm::style::Stylize;
use std::env;
use std::io::ErrorKind;
use std::str::FromStr;

fn main() {
    #[cfg(not(feature = "standard_panic"))]
    misc::set_panic_hook();

    let args = Cli::parse();

    let shell = args
        .shell
        .and_then(|shell| shells::Shell::from_str(&shell).ok())
        .or(shells::get_current_shell())
        .expect("Could not determine the current shell.");

    match args.command {
        Command::Alias { name } => {
            let program_path =
                env::current_exe().expect("Could not determine the current executable path.");
            let alias = shell.get_shell_function(&name, program_path.as_path());
            println!("{alias}");
        }
        Command::Fix => {
            let command =
                env::var("SH_PREV_CMD").expect("SH_PREV_CMD environment variable is not set.");
            let expand_command = misc::expand_aliases(&command, shell.get_aliases());
            let fixed_command = fix::fix_command(command, expand_command);
            match fixed_command {
                Ok(cmd) => println!("{cmd}"),
                Err(e) => panic!("Failed to fix command: {e}"),
            }
        }
        Command::Setup { name } => {
            let program_path =
                env::current_exe().expect("Could not determine the current executable path.");
            match shell.setup_alias(&name, program_path.as_path()) {
                Ok(_) => println!(
                    "{}",
                    format!("Alias setup successfully for {shell:?} as {name}").green()
                ),
                Err(e) => match e.kind() {
                    ErrorKind::AlreadyExists => {
                        println!("{}", "Alias already exists, skipping alias setup.".yellow())
                    }
                    _ => panic!("Failed to set up alias: {e}"),
                },
            }
            match dirs::config_dir()
                .ok_or(ErrorKind::NotFound.into())
                .and_then(|dir| misc::create_default_fix_rules(dir.join("theshit/fix_rules")))
            {
                Ok(_) => println!("{}", "Default rules setup successfully".green()),
                Err(e) => match e.kind() {
                    ErrorKind::AlreadyExists => {
                        println!(
                            "{}",
                            "Default rules already exist, skipping rules setup.".yellow()
                        );
                    }
                    _ => {
                        panic!("Failed to set up default rules: {e}");
                    }
                },
            }
        }
    }
}
