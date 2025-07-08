use super::enums::Shell;
use std::str::FromStr;
use std::{env, process};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

pub fn get_current_shell() -> Option<Shell> {
    get_current_shell_by_env().or_else(get_current_shell_by_process)
}

fn get_current_shell_by_env() -> Option<Shell> {
    env::var("SH_SHELL")
        .ok()
        .and_then(|shell| Shell::from_str(shell.as_str()).ok())
}

fn get_current_shell_by_process() -> Option<Shell> {
    let mut system = System::new();
    system
        .refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
    let mut current_process = system.process(Pid::from_u32(process::id()));
    loop {
        let process = current_process?;
        let result: Option<Shell> = process
            .exe()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .and_then(|name_str| Shell::from_str(name_str).ok());
        match result {
            Some(_) => return result,
            None => {
                current_process = system.process(process.parent().unwrap_or(Pid::from_u32(0)));
            }
        }
    }
}
