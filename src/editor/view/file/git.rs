use std::io::Write;
use std::process::Stdio;
use std::{collections::HashMap, process::Command};

use gix::{self};

#[derive(Clone, Copy, Debug)]
pub enum PatchType {
    Added,
    Deleted,
    Changed,
}

pub struct Patch {
    start: usize,
    count: usize,
    patch_type: PatchType,
}

pub trait Vcs {
    fn get_ref(&self) -> String;
    fn compute_diff(
        &mut self,
        file_path: &str,
        file_name: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn diff(&self) -> Option<HashMap<usize, PatchType>>;
}

pub struct Git {
    repo: gix::Repository,
    diff: Option<HashMap<usize, PatchType>>,
}

impl Git {
    pub fn open() -> Option<Self> {
        if let Ok(repo) = gix::discover(".") {
            Some(Self { repo, diff: None })
        } else {
            None
        }
    }
}
impl Vcs for Git {
    fn get_ref(&self) -> String {
        match self.repo.head_name().unwrap() {
            Some(name) => {
                let reference = name.to_string();
                // Remove the "refs/heads/" prefix
                reference.trim_start_matches("refs/heads/").to_string()
            }
            None => {
                // No branch, it is a detached head
                let commit = self.repo.head_commit().unwrap();
                commit.short_id().unwrap().to_string()
            }
        }
    }
    fn compute_diff(
        &mut self,
        file_path: &str,
        file_name: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let diff = get_diff_result(content, file_path, file_name)?;
        let patches = parse_diff_result(&diff)?;

        let mut marks = HashMap::new();

        for patch in patches {
            match patch.patch_type {
                PatchType::Added => {
                    for i in 0..patch.count {
                        marks.insert(patch.start + i, PatchType::Added);
                    }
                }
                PatchType::Deleted => {
                    for i in 0..patch.count {
                        marks.insert(patch.start + i, PatchType::Deleted);
                    }
                }
                PatchType::Changed => {
                    for i in 0..patch.count {
                        marks.insert(patch.start + i, PatchType::Changed);
                    }
                }
            }
        }
        self.diff = Some(marks);
        Ok(())
    }

    fn diff(&self) -> Option<HashMap<usize, PatchType>> {
        self.diff.clone()
    }
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
) -> Result<String, Box<dyn std::error::Error>> {
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
fn parse_diff_result(diff: &str) -> Result<Vec<Patch>, Box<dyn std::error::Error>> {
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
