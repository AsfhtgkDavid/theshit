use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(long, short, help = "Specify the shell to use (e.g., bash, zsh)")]
    pub shell: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    Alias {
        #[arg(default_value_t = String::from("shit"))]
        name: String,
    },
    Fix,
    Setup {
        #[arg(default_value_t = String::from("shit"))]
        name: String,
    },
}
