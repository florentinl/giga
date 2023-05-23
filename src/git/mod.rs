use std::process::Command;

/// Get wether or not the current directory is a git repository
/// If it is, return the current reference name
pub fn get_ref_name() -> Option<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .expect("Failed to execute git command");
    if output.status.success() {
        let branch = String::from_utf8(output.stdout).unwrap();
        Some(branch.trim().to_string())
    } else {
        None
    }
}
