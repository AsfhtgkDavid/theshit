#[cfg(not(feature = "standard_panic"))]
use crossterm::style::Stylize;
use include_dir::{Dir, DirEntry, include_dir};
use regex::Regex;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs;
use std::io::{ErrorKind, Result};
use std::path::{Path, PathBuf};

static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[cfg(not(feature = "standard_panic"))]
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| *s)
            .or_else(|| info.payload().downcast_ref::<String>().map(|s| &**s))
            .unwrap_or("Unknown panic");
        eprintln!("Panic occurred: {}", msg.red());
        std::process::exit(1);
    }));
}

macro_rules! min_of {
    ($x:expr) => ($x);
    ($x:expr, $($rest:expr),+) => (
        std::cmp::min($x, min_of!($($rest),+))
    );
}

fn copy_dir_recursive(src: &Dir, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in src.entries() {
        let dst_path = dst.join(entry.path().strip_prefix(src.path()).unwrap());
        match entry {
            DirEntry::Dir(dir) => copy_dir_recursive(dir, &dst_path)?,
            DirEntry::File(file) => fs::write(&dst_path, file.contents())?,
        }
    }
    Ok(())
}

pub fn create_default_fix_rules(rules_dir: PathBuf) -> Result<()> {
    if rules_dir.as_path().exists() {
        return Err(ErrorKind::AlreadyExists.into());
    }

    copy_dir_recursive(
        ASSETS_DIR
            .get_dir("rules")
            .expect("Active rules didn't find"),
        &rules_dir,
    )?;
    Ok(())
}

pub fn expand_aliases(command: &str, aliases: HashMap<String, String>) -> String {
    let binary = command.split(' ').next().expect("Could not find binary");

    if aliases.contains_key(binary) {
        command.replacen(binary, &aliases[binary], 1)
    } else {
        command.to_string()
    }
}

fn damerau_levenshtein_distance(s1: &str, s2: &str) -> usize {
    let rows = s1.len() + 1;
    let columns = s2.len() + 1;
    let s1 = s1.chars().collect::<Vec<_>>().into_boxed_slice();
    let s2 = s2.chars().collect::<Vec<_>>().into_boxed_slice();
    let mut matrix = vec![0usize; columns * rows].into_boxed_slice(); // matrix[i,j] = matrix[i*columns+j+1]

    for i in 0..rows {
        for j in 0..columns {
            if min(i, j) == 0 {
                matrix[i * columns + j] = max(i, j);
            } else {
                let indicator = if s1[i - 1] != s2[j - 1] { 1 } else { 0 };
                let part_value = min_of!(
                    matrix[(i - 1) * columns + j] + 1,
                    matrix[i * columns + j - 1] + 1,
                    matrix[(i - 1) * columns + j - 1] + indicator
                );
                if i > 1 && j > 1 && s1[i - 1] == s2[j - 2] && s1[i - 2] == s2[j - 1] {
                    matrix[i * columns + j] =
                        min(part_value, matrix[(i - 2) * columns + j - 2] + 1);
                } else {
                    matrix[i * columns + j] = part_value;
                }
            }
        }
    }

    matrix[matrix.len() - 1]
}

pub fn string_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }
    let max_len = max(s1.len(), s2.len());
    let distance = damerau_levenshtein_distance(s1, s2);
    1.0 - (distance as f64 / max_len as f64)
}

pub fn split_command(command: &str) -> Vec<String> {
    shell_words::split(command)
        .unwrap_or(command.split_whitespace().map(|s| s.to_string()).collect())
}

pub fn replace_argument(script: &str, from: &str, to: &str) -> String {
    let end_pattern = format!(r" {}$", regex::escape(from));
    let end_regex = Regex::new(&end_pattern).unwrap();

    if end_regex.is_match(script) {
        return end_regex.replace(script, format!(" {to}")).to_string();
    }

    let middle_pattern = format!(" {} ", regex::escape(from));
    let replacement = format!(" {to} ");

    script.replacen(&middle_pattern, &replacement, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_command() {
        assert_eq!(
            split_command("echo hello world"),
            vec!["echo", "hello", "world"]
        );
        assert_eq!(
            split_command("echo 'hello world'"),
            vec!["echo", "hello world"]
        );
        assert_eq!(
            split_command("echo \"hello world\""),
            vec!["echo", "hello world"]
        );
        assert_eq!(split_command("echo"), vec!["echo"]);
        assert_eq!(split_command(""), Vec::<String>::new());
    }

    #[test]
    fn test_replace_argument() {
        let script = "echo hello world";
        assert_eq!(
            replace_argument(script, "world", "everyone"),
            "echo hello everyone"
        );
        assert_eq!(replace_argument(script, "hello", "hi"), "echo hi world");
        assert_eq!(
            replace_argument(script, "echo", "print"),
            "echo hello world"
        );
        assert_eq!(replace_argument(script, "notfound", "replacement"), script);
    }

    #[test]
    fn creates_fix_rules() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();

        assert!(
            create_default_fix_rules(temp_dir_path.to_path_buf().join("theshit/fix_rules")).is_ok()
        );
        assert!(temp_dir_path.join("theshit/fix_rules/active").exists());
        assert!(temp_dir_path.join("theshit/fix_rules/additional").exists());
    }

    #[test]
    fn returns_error_when_fix_rules_already_exist() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();
        let rules_dir = temp_dir_path.join("theshit/fix_rules");
        fs::create_dir_all(&rules_dir).unwrap();

        let result = create_default_fix_rules(rules_dir);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::AlreadyExists);
    }

    fn get_mock_alias() -> HashMap<String, String> {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -l".to_string());
        aliases.insert("la".to_string(), "ls -la".to_string());
        aliases.insert("grep".to_string(), "grep --color=auto".to_string());
        aliases.insert("cls".to_string(), "clear".to_string());
        aliases
    }

    #[test]
    fn test_expand_simple_alias() {
        let aliases = get_mock_alias();

        let result = expand_aliases("ll", aliases);
        assert_eq!(result, "ls -l");
    }

    #[test]
    fn test_expand_alias_with_arguments() {
        let aliases = get_mock_alias();

        let result = expand_aliases("ll /home/user", aliases);
        assert_eq!(result, "ls -l /home/user");
    }

    #[test]
    fn test_expand_alias_with_multiple_arguments() {
        let aliases = get_mock_alias();

        let result = expand_aliases("grep pattern file.txt", aliases);
        assert_eq!(result, "grep --color=auto pattern file.txt");
    }

    #[test]
    fn test_no_alias_found() {
        let aliases = get_mock_alias();

        let result = expand_aliases("vim file.txt", aliases);
        assert_eq!(result, "vim file.txt");
    }

    #[test]
    fn test_empty_aliases() {
        let aliases = HashMap::new();

        let result = expand_aliases("ls", aliases);
        assert_eq!(result, "ls");
    }

    #[test]
    fn test_alias_only_replaces_first_occurrence() {
        let mut aliases = HashMap::new();
        aliases.insert("test".to_string(), "echo".to_string());

        let result = expand_aliases("test test again", aliases);
        assert_eq!(result, "echo test again");
    }

    #[test]
    fn test_single_word_command() {
        let aliases = get_mock_alias();

        let result = expand_aliases("cls", aliases);
        assert_eq!(result, "clear");
    }

    #[test]
    fn test_string_similarity() {
        assert_eq!(string_similarity("kittegn1", "sitting"), 0.5);
        assert_eq!(string_similarity("law", "lawn"), 0.75);
        assert_eq!(string_similarity("", ""), 1.0);
        assert_eq!(string_similarity("abc", "abc"), 1.0);
    }
}
