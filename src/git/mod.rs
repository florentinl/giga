use std::process::Command;

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
