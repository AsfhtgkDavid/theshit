use super::structs::Command;
use crossterm::style::Stylize;
use pyo3::types::{PyAnyMethods, PyList, PyListMethods};
use pyo3::{PyResult, Python};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

fn check_security(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let file_uid = metadata.uid();
    let current_uid = unsafe { libc::geteuid() };

    if current_uid != file_uid {
        return Err(format!(
            "{} Running with UID {}, but file '{}' is owned by UID {}. Aborting to prevent privilege escalation.",
            "SECURITY ERROR:".red().bold(),
            current_uid,
            path.display(),
            file_uid
        ));
    }

    if metadata.permissions().mode() & 0o022 == 0 {
        return Err(format!(
            "{} Python rule '{}' is writable by non-owners. Aborting to prevent privilege escalation.",
            "SECURITY ERROR:".red().bold(),
            path.display()
        ));
    }

    Ok(())
}

pub fn process_python_rules(
    command: &Command,
    rule_paths: Vec<PathBuf>,
) -> Result<Vec<String>, String> {
    let module_path = get_common_parent(&rule_paths)
        .ok_or("No common parent found for rule paths".to_string())?;
    let mut fixed_commands: Vec<String> = vec![];
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| -> PyResult<()> {
        {
            let raw_sys_path = py.import("sys")?.getattr("path")?;
            let sys_path = raw_sys_path.downcast::<PyList>()?;
            sys_path.insert(0, module_path.to_string_lossy())?;
        }

        for rule_path in rule_paths {
            if let Err(e) = check_security(&rule_path) {
                eprintln!("{}", e);
                continue;
            }

            let module_name = match get_module_name(&module_path, &rule_path) {
                Some(module_name) => module_name,
                None => continue,
            };
            let module = match py.import(&module_name) {
                Ok(module) => module,
                Err(e) => {
                    eprintln!(
                        "{}{}{}",
                        "Failed to import rule module '".yellow(),
                        rule_path.display(),
                        "': ".yellow(),
                    );
                    eprintln!("{e}");
                    continue;
                }
            };
            let match_func = module.getattr("match")?;
            let fix_func = module.getattr("fix")?;
            if match_func.is_callable() && fix_func.is_callable() {
                if match_func
                    .call1((
                        command.command(),
                        command.output().stdout(),
                        command.output().stderr(),
                    ))?
                    .extract::<bool>()?
                {
                    let fixed_command: String = fix_func
                        .call1((
                            command.command(),
                            command.output().stdout(),
                            command.output().stderr(),
                        ))?
                        .extract()?;
                    fixed_commands.push(fixed_command);
                }
            } else {
                eprintln!(
                    "{}{}{}",
                    "Rule '".yellow(),
                    rule_path.display(),
                    "' is missing required functions (match, fix)".yellow()
                );
            }
        }
        Ok(())
    })
    .map_err(|err| format!("Failed to process Python rules: {err}"))?;
    Ok(fixed_commands)
}

fn get_module_name(modules_dir_path: &Path, rule_path: &Path) -> Option<String> {
    let mut module_path = match rule_path.strip_prefix(modules_dir_path) {
        Ok(module_path) => module_path.parent().unwrap_or(Path::new("")).to_path_buf(),
        Err(_) => {
            eprintln!(
                "{}{}{}",
                "Rule path '".yellow(),
                rule_path.display(),
                "' is not a subpath of the common parent".yellow()
            );
            return None;
        }
    };
    match rule_path.file_stem() {
        Some(module_stem) => {
            module_path.push(module_stem);
        }
        None => {
            eprintln!(
                "{}{}{}",
                "Rule path '".yellow(),
                rule_path.display(),
                "' has no valid file stem".yellow()
            );
            return None;
        }
    }
    Some(module_path.to_string_lossy().replace(['/', '\\'], "."))
}

fn get_common_parent(paths: &[PathBuf]) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }

    if paths.len() == 1 {
        return Some(paths[0].parent().unwrap_or(Path::new("")).to_path_buf());
    }

    let mut iter = paths.iter();
    let first = iter.next()?.components().collect::<Vec<_>>();

    let common = iter.fold(first, |acc, path| {
        let comps = path.components().collect::<Vec<_>>();
        acc.iter()
            .zip(&comps)
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| *a)
            .collect()
    });

    if common.is_empty() {
        None
    } else {
        Some(common.iter().collect())
    }
}
