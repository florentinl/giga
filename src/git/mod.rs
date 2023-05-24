use std::{
    error::Error,
    io::Write,
    process::{Command, Stdio},
};

/// The Diff is used to show ticks on the left of the editor
/// to show which lines have been modified/added/removed since the last commit
type Diff = Vec<DiffLine>;
enum DiffType {
    /// The line has been modified
    Modified,
    /// The line has been added
    Added,
    /// The line has been removed
    Removed,
}
/// A line in the diff
struct DiffLine {
    /// The line number
    line: usize,
    /// The type of modification
    diff_type: DiffType,
}

/// Get the result of the `diff` command between the current commit and the string given in parameter
/// for the given file path. The exact command is:
///
/// ```sh
/// diff -u <(git show HEAD:{file_name}) <(echo {content})
/// ```
/// and should be run where the file is located (`file_path`).
pub fn get_diff_result(
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
    let diff_output = Command::new("bash")
        .current_dir(file_path)
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .arg("-c")
        .arg(format!(
            "diff -u <(git show HEAD:{}) <(echo '{}')",
            file_name, content
        ))
        .spawn()?
        .wait_with_output()?;

    let status_code = diff_output.status.code();
    if matches!(status_code, Some(0 | 1)) {
        Ok(String::from_utf8(diff_output.stdout)?)
    } else {
        Err(String::from_utf8(diff_output.stderr)?.into())
    }
}

/// Get wether or not the current directory is a git repository
/// If it is, return the current reference name
pub fn get_ref_name(path: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(path)
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
        let diff = get_diff_result(content, file_path, file_name).unwrap();
        println!("Diff is {}", diff);
    }
}
