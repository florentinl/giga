//! # Provide git integration for the editor
//!
//! This module is used to perform all the git operations. For now there are only two:
//! - querying the current branch
//! - querying the diff between the current commit and the current file

use std::{
    error::Error,
    io::Write,
    process::{Command, Stdio},
};

/// The Diff is used to show ticks on the left of the editor
/// to show which lines have been Changed/added/Deleted since the last commit
pub type Diff = Vec<Patch>;

#[derive(Debug, PartialEq)]
pub enum PatchType {
    Added,
    Deleted,
    Changed,
}

#[derive(Debug, PartialEq)]
pub struct Patch {
    pub start: usize,
    pub count: usize,
    pub patch_type: PatchType,
}

/// Compute the diff between the current commit and the string given in parameter
/// for the given file path.
pub fn compute_diff(
    content: &str,
    file_path: &str,
    file_name: &str,
) -> Result<Diff, Box<dyn Error>> {
    let diff_result = get_diff_result(content, file_path, file_name)?;
    parse_diff_result(&diff_result)
}

/// Get the result of the `diff` command between the current commit and the string given in parameter
/// for the given file path. The exact command is:
/// ```sh
/// diff -u <(git show HEAD:{file_name}) <(echo {content})
/// ```
/// and should be run where the file is located (`file_path`).
fn get_diff_result(
    content: &str,
    file_path: &str,
    file_name: &str,
) -> Result<String, Box<dyn Error>> {
    // Get the file_name relative to the file_path git repository
    let file_name = Command::new("git")
        .current_dir(file_path)
        .args(["ls-files", "--full-name", file_name])
        .output()?
        .stdout;
    let file_name = String::from_utf8_lossy(&file_name).trim().to_string();

    if file_name.is_empty() {
        // It is a new file in the git repository so we return the number of lines
        // as the number of lines added at the beginning of the file
        return Ok(format!("0a1,{}", content.lines().count().max(1)));
    }

    // Execute the shell command
    // It would be better to not rely on bash but I don't know how to emulate the process substitution
    // with Rust.
    let mut diff = Command::new("bash")
        .current_dir(file_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("-c")
        .arg(format!("diff <(git show HEAD:{}) -", file_name))
        .spawn()?;

    // Write the content to diff on the stdin of the process
    let diff_input = diff.stdin.as_mut().unwrap();
    diff_input.write_all(content.as_bytes())?;

    // Wait for the process to finish
    let diff_output = diff.wait_with_output()?;

    // Check the exit code and parse stdout/stderr accordingly
    let status_code = diff_output.status.code();
    if matches!(status_code, Some(0 | 1)) {
        Ok(String::from_utf8(diff_output.stdout)?)
    } else {
        Err(String::from_utf8(diff_output.stderr)?.into())
    }
}

/// Parse the diff result and return a vector of Patches
/// The diff result is a string of the form:
/// ```diff
/// 1c1,3
/// < Hello, World !
/// ---
/// > Hello
/// > World
/// >
/// ```
/// Only the lines starting with digits are parsed.
fn parse_diff_result(diff: &str) -> Result<Diff, Box<dyn Error>> {
    let mut result = vec![];

    for line in diff.lines() {
        // We only care for lines starting with a digit (the line number)
        if line.starts_with(char::is_numeric) {
            // Only keep the part after the 'a'/'d'/'c' character since we want to know
            // the position of the line in the current file.
            let (_, rhs) = line.split_once(char::is_alphabetic).unwrap_or_default();
            let mut rhs = rhs.split(',');
            // Diff is 1-based and we want 0-based
            let start = rhs
                .next()
                .unwrap_or_default()
                .parse::<usize>()?
                .saturating_sub(1);
            let count = rhs
                .next()
                .map(|s| s.parse::<usize>().unwrap_or_default().saturating_sub(start))
                .unwrap_or(1);
            let patch_type = if line.contains('a') {
                PatchType::Added
            } else if line.contains('d') {
                PatchType::Deleted
            } else {
                PatchType::Changed
            };
            result.push(Patch {
                start,
                count,
                patch_type,
            });
        }
    }
    Ok(result)
}

/// Get wether or not the current directory is a git repository
/// If it is, return the current reference name
pub fn get_ref_name(path: &str) -> Option<String> {
    let output = Command::new("git")
        .current_dir(path)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .ok()?;
    if output.status.success() {
        let branch = String::from_utf8(output.stdout).ok()?;
        Some(branch.trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_diff_result() {
        let content = "Hello\nWorld\n";
        let file_path = "tests";
        let file_name = "sample.txt";
        let expected = "1c1,2
< Hello, World !
---
> Hello
> World
";
        let diff = get_diff_result(content, file_path, file_name);
        assert!(diff.is_ok());
        assert_eq!(diff.unwrap(), expected);
    }

    #[test]
    fn test_parse_diff_result() {
        let diff = "1c1,3
< Hello, World !
---
> Hello
> World
> ";
        let expected = vec![Patch {
            start: 0,
            count: 3,
            patch_type: PatchType::Changed,
        }];

        let parsed = parse_diff_result(diff);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_long_parse_diff_result() {
        // The diff is in the file `tests/long_diff.txt`
        let diff = include_str!("../../tests/long_diff.txt");

        let parsed = parse_diff_result(diff);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        use PatchType::*;
        let expected = vec![
            Patch {
                start: 0,
                count: 1,
                patch_type: Changed,
            },
            Patch {
                start: 4,
                count: 10,
                patch_type: Changed,
            },
            Patch {
                start: 37,
                count: 1,
                patch_type: Changed,
            },
            Patch {
                start: 38,
                count: 1,
                patch_type: Deleted,
            },
            Patch {
                start: 41,
                count: 1,
                patch_type: Changed,
            },
            Patch {
                start: 44,
                count: 1,
                patch_type: Changed,
            },
            Patch {
                start: 48,
                count: 2,
                patch_type: Added,
            },
            Patch {
                start: 56,
                count: 41,
                patch_type: Added,
            },
            Patch {
                start: 101,
                count: 1,
                patch_type: Added,
            },
            Patch {
                start: 104,
                count: 1,
                patch_type: Deleted,
            },
            Patch {
                start: 124,
                count: 37,
                patch_type: Changed,
            },
        ];
        println!("{:?}", parsed);
        assert_eq!(parsed, expected);
    }
}
