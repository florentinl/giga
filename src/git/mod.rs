use std::{error::Error, io::Write, process::Command};

/// The Diff is used to show ticks on the left of the editor
/// to show which lines have been Changed/added/Deleted since the last commit
pub type Diff = Vec<Patches>;

#[derive(Debug, PartialEq)]
pub enum Patches {
    /// {count} lines have been Changed starting at {start}
    Changed { start: usize, count: usize },
    /// {count} lines have been added starting at {start}
    Added { start: usize, count: usize },
    /// Lines have been Deleted starting at {start}
    Deleted { start: usize },
}

/// Compute the diff between the current commit and the string given in parameter
/// for the given file path.
pub fn compute_diff(
    content: &str,
    file_path: &str,
    file_name: &str,
) -> Result<Diff, Box<dyn Error>> {
    let diff_result = get_diff_result(content, file_path, file_name)?;
    Ok(parse_diff_result(&diff_result)?)
}

/// Get the result of the `diff` command between the current commit and the string given in parameter
/// for the given file path. The exact command is:
///
/// ```sh
/// diff -u <(git show HEAD:{file_name}) <(echo {content})
/// ```
/// and should be run where the file is located (`file_path`).
fn get_diff_result(
    content: &str,
    file_path: &str,
    file_name: &str,
) -> Result<String, Box<dyn Error>> {
    // Get the file_name relative to the current git repository
    let file_name = Command::new("git")
        .current_dir(file_path)
        .args(&["ls-files", "--full-name", file_name])
        .output()?
        .stdout;
    let file_name = String::from_utf8_lossy(&file_name).trim().to_string();

    // Execute the shell command
    let mut diff = Command::new("bash")
        .current_dir(file_path)
        .arg("-c")
        .arg(format!("diff <(git show HEAD:{}) -", file_name))
        .spawn()?;

    let diff_input = diff.stdin.as_mut().unwrap();
    diff_input.write_all(content.as_bytes())?;

    let mut diff_output = diff.wait_with_output()?;

    let status_code = diff_output.status.code();
    if matches!(status_code, Some(0 | 1)) {
        // Remove the trailing newline
        diff_output.stdout.pop();
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
/// Only the lines starting with `@@` are parsed.
fn parse_diff_result(diff: &str) -> Result<Diff, Box<dyn Error>> {
    let mut result = vec![];

    for line in diff.lines() {
        // We only care for lines starting with a digit (the line number)
        if line.starts_with(char::is_numeric) {
            // Add patch
            if line.contains('a') {
                let parts = line.split('a').collect::<Vec<_>>();
                let mut added = parts[1].split(',');
                let start = added.next().unwrap_or_default().parse::<usize>()? - 1;
                let count = added
                    .next()
                    .map(|s| s.parse::<usize>().unwrap() - start)
                    .unwrap_or(1);
                result.push(Patches::Added { start, count });
            } else if line.contains('d') {
                let parts = line.split('d').collect::<Vec<_>>();
                let start = parts[1].parse::<usize>()? - 1;
                result.push(Patches::Deleted { start });
            } else if line.contains('c') {
                let parts = line.split('c').collect::<Vec<_>>();
                let mut changed = parts[1].split(',');
                let start = changed.next().unwrap_or_default().parse::<usize>()? - 1;
                let count = changed
                    .next()
                    .map(|s| s.parse::<usize>().unwrap() - start)
                    .unwrap_or(1);
                result.push(Patches::Changed { start, count });
            }
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
        let branch = String::from_utf8(output.stdout).unwrap();
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
        let expected = "1c1,3
< Hello, World !
---
> Hello
> World
> ";
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
        let expected = vec![Patches::Changed { start: 0, count: 3 }];

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
        let expected = vec![
            Patches::Changed { start: 0, count: 1 },
            Patches::Changed {
                start: 4,
                count: 10,
            },
            Patches::Changed {
                start: 37,
                count: 1,
            },
            Patches::Deleted { start: 38 },
            Patches::Changed {
                start: 41,
                count: 1,
            },
            Patches::Changed {
                start: 44,
                count: 1,
            },
            Patches::Added {
                start: 48,
                count: 2,
            },
            Patches::Added {
                start: 56,
                count: 41,
            },
            Patches::Added {
                start: 101,
                count: 1,
            },
            Patches::Deleted { start: 104 },
            Patches::Changed {
                start: 124,
                count: 37,
            },
        ];
        assert_eq!(parsed, expected);
    }
}
