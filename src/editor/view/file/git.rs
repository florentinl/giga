use git2::{DiffOptions, Patch as GitPatch};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PatchType {
    Added,
    Deleted,
    Changed,
}

#[derive(PartialEq, Debug)]
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
    repo: git2::Repository,
    diff: Option<HashMap<usize, PatchType>>,
}

impl Git {
    pub fn open() -> Option<Self> {
        if let Ok(repo) = git2::Repository::discover(".") {
            Some(Self { repo, diff: None })
        } else {
            None
        }
    }
}
impl Vcs for Git {
    fn get_ref(&self) -> String {
        match self.repo.head() {
            Ok(head) => {
                if let Some(name) = head.name() {
                    // Remove the "refs/heads/" prefix
                    name.trim_start_matches("refs/heads/").to_string()
                } else {
                    if let Some(hash) = head.shorthand() {
                        // If the head is a commit hash
                        hash.to_string()
                    } else {
                        // If the head is not utf-8 encoded
                        "".to_string()
                    }
                }
            }
            Err(_) => panic!("Error while getting the head"),
        }
    }
    fn compute_diff(
        &mut self,
        file_path: &str,
        file_name: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let patches = get_diff_result(content, file_path, file_name)?;

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
) -> Result<Vec<Patch>, Box<dyn std::error::Error>> {
    let old_content = std::fs::read_to_string(file_path.to_owned() + file_name);
    let old_buffer = match &old_content {
        Ok(content) => content.as_bytes(),
        Err(_) => b"",
    };
    let new_buffer = content.as_bytes();
    let mut options = DiffOptions::default();
    options.context_lines(0);

    let patch = GitPatch::from_buffers(old_buffer, None, new_buffer, None, Some(&mut options))?;

    let mut patches: Vec<Patch> = vec![];

    for num in 0..patch.num_hunks() {
        let hunk = patch.hunk(num)?.0;

        let new_start = hunk.new_start() as usize;

        let old_count = hunk.old_lines() as usize;
        let new_count = hunk.new_lines() as usize;

        /*
        Let's perform a guess on the patch type:
        - If the old count is the same as the new count, it's a change
        - If the old count is less than the new count, it's an addition
        - If the old count is more than the new count, it's a deletion
        */

        let patch_type = if old_count == new_count {
            PatchType::Changed
        } else if old_count < new_count {
            PatchType::Added
        } else {
            PatchType::Deleted
        };

        /*  If patch is a deletion, new_count will be 0 so we hard code it to 1
            and start will be marked beneath the deleted line.

            However, if patch is addition or change, line will be 1-indexed and we use 0-indexed line
            so we need to increment it by 1.
        */
        patches.push(Patch {
            start: if patch_type == PatchType::Deleted || new_start == 0 {
                new_start
            } else {
                new_start - 1
            },
            count: if patch_type == PatchType::Deleted {
                1
            } else {
                new_count
            },
            patch_type,
        });
    }
    Ok(patches)
}

mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    fn test_get_diff_result_new_file() {
        let content = "Hello, World !";
        let file_path = "tests";
        let file_name = "new_file.txt";
        let patches = get_diff_result(content, file_path, file_name).unwrap();
        assert_eq!(
            patches[0],
            Patch {
                start: 1,
                count: 1,
                patch_type: PatchType::Added
            }
        );
    }
}
