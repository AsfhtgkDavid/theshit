mod cargo_no_command;
mod mkdir_p;
mod sudo;
mod to_cd;
mod unsudo;

use super::structs::Command;
use strum::EnumString;

#[derive(EnumString, Debug)]
pub enum NativeRule {
    #[strum(serialize = "sudo")]
    Sudo,
    #[strum(serialize = "to_cd")]
    ToCd,
    #[strum(serialize = "unsudo")]
    Unsudo,
    #[strum(serialize = "mkdir_p")]
    MkdirP,
    #[strum(serialize = "cargo_no_command")]
    CargoNoCommand,
}

impl NativeRule {
    pub fn fix_native(self, command: &Command) -> Option<String> {
        match self {
            NativeRule::Sudo => Self::match_and_fix(sudo::is_match, sudo::fix, command),
            NativeRule::ToCd => Self::match_and_fix(to_cd::is_match, to_cd::fix, command),
            NativeRule::Unsudo => Self::match_and_fix(unsudo::is_match, unsudo::fix, command),
            NativeRule::MkdirP => Self::match_and_fix(mkdir_p::is_match, mkdir_p::fix, command),
            NativeRule::CargoNoCommand => {
                Self::match_and_fix(cargo_no_command::is_match, cargo_no_command::fix, command)
            }
        }
    }

    fn match_and_fix(
        match_function: fn(&Command) -> bool,
        fix_function: fn(&Command) -> String,
        command: &Command,
    ) -> Option<String> {
        if match_function(command) {
            Some(fix_function(command))
        } else {
            None
        }
    }
}
