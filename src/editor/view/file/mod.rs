//! # In-memory representation of the file being edited
//!
//! The File structure is the in-memory representation of the full file being edited with syntax highlighting.
//! It is a vector of lines, each line being a vector of ColorChar (a char and its associated color). There are
//! two types of operations on the File:
//! - Read operations: they are used to display the file on the screen
//! - Write operations: they are used to modify the file -> Trigger a recolorization of the file
pub mod git;

use std::collections::HashMap;

use ropey::Rope;

use self::git::{Git, PatchType, Vcs};

/// In-memory representation of a syntax-highlighted file
pub struct File {
    /// File path
    pub file_dir: String,

    /// File name
    pub file_name: String,

    /// The content of the file
    content: Rope,

    /// Optional version control system
    vcs: Option<Git>,
}

pub trait EditorFile {
    fn new(file_path: &str) -> Self;
    fn from_string(content: &str, file_name: &str, file_path: &str) -> Self;
    fn line(&self, index: usize) -> Option<Vec<char>>;
    fn len(&self) -> usize;
    fn insert(&mut self, line: usize, col: usize, c: char);
    fn delete(&mut self, line: usize, col: usize);
    fn split_line(&mut self, line: usize, col: usize);
    fn delete_line(&mut self, line: usize);
    fn git_ref(&self) -> Option<String>;
    fn refresh_diff(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn diff(&self) -> Option<HashMap<usize, PatchType>>;
}

impl EditorFile for File {
    fn new(file_path: &str) -> Self {
        Self {
            file_dir: file_path.into(),
            file_name: "New file".to_string(),
            content: Rope::new(),
            vcs: Git::open(),
        }
    }

    /// Create a File abstraction from a string
    fn from_string(content: &str, file_name: &str, file_path: &str) -> Self {
        // Replace tabs with 4 spaces
        let content = content.replace('\t', "    ");
        let content = Rope::from_str(&content);
        Self {
            file_name: file_name.into(),
            file_dir: file_path.into(),
            content,
            vcs: Git::open(),
        }
    }

    /// Get the nth line of the file
    fn line(&self, index: usize) -> Option<Vec<char>> {
        if self.content.len_lines() <= index {
            None
        } else {
            Some(
                self.content
                    .line(index)
                    .to_string()
                    .trim_end_matches('\n')
                    .chars()
                    .collect(),
            )
        }
    }

    /// Get the number of lines in the file
    fn len(&self) -> usize {
        self.content.len_lines()
    }

    /// Insert a char at the given position in the file
    /// - line >= len || col > line_len: do nothing
    /// - else insert the byte at the given position
    fn insert(&mut self, line: usize, col: usize, c: char) {
        if line >= self.content.len_lines() {
            return;
        }
        // Convert line and col to char_idx
        let line_len = self.content.line(line).len_chars();
        if col > line_len {
            return;
        }
        let char_idx = self.content.line_to_char(line) + col;
        self.content.insert_char(char_idx, c);
    }

    /// Delete a char at the given position
    /// - col == 0: join the line with the previous one (except if it's the first line)
    /// - 0 < col <= line_len: delete the byte at the given position
    /// - col > line_len: do nothing
    fn delete(&mut self, line: usize, col: usize) {
        if line >= self.content.len_lines() {
            return;
        }

        let line_len = self.content.line(line).len_chars();
        if col == 0 {
            if line > 0 {
                // Join the line with the previous one
                let line = self.content.line_to_char(line);
                // Remove the newline character
                self.content.remove(line - 1..line);
            }
        } else if col <= line_len {
            let char_idx = self.content.line_to_char(line) + col - 1;
            self.content.remove(char_idx..char_idx + 1);
        }
    }

    /// Split a line at the given position
    /// - line >= len: do nothing
    /// - col > line_len: do nothing
    /// - else split the line at the given position
    fn split_line(&mut self, line: usize, col: usize) {
        if line >= self.content.len_lines() {
            return;
        }
        let line_len = self.content.line(line).len_chars();
        if col > line_len {
            return;
        }
        let char_idx = self.content.line_to_char(line) + col;
        self.content.insert_char(char_idx, '\n');
    }

    fn delete_line(&mut self, line: usize) {
        let start_line = self.content.line_to_char(line);
        let end_line = self.content.line_to_char(line + 1);
        self.content.remove(start_line..end_line);
    }

    fn git_ref(&self) -> Option<String> {
        match &self.vcs {
            Some(vcs) => Some(vcs.get_ref()),
            None => None,
        }
    }

    fn refresh_diff(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let None = &self.vcs {
            return Ok(());
        }
        let content = self.to_string();
        let vcs = self.vcs.as_mut().unwrap();
        vcs.compute_diff(&self.file_dir, &self.file_name, &content)
    }

    fn diff(&self) -> Option<HashMap<usize, PatchType>> {
        let vcs = self.vcs.as_ref()?;

        vcs.diff()
    }
}

/// Implement the ToString trait for File (used for saving the file)
impl ToString for File {
    fn to_string(&self) -> String {
        self.content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_new_empty() {
        let file = File::new(".");
        assert_eq!(file.content.len_lines(), 1);
    }

    #[test]
    fn file_from_string() {
        let file = File::from_string("Hello, World !", "test", "test");
        assert_eq!(file.content.len_lines(), 1);
        assert_eq!(file.content.line(0), "Hello, World !");
    }

    #[test]
    fn file_to_string() {
        let file = File::from_string("Hello, World !", "test", "test");
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_get_line() {
        let file = File::from_string("Hello, World !\n", "test", "test");

        assert_eq!(
            file.line(0).unwrap().iter().collect::<String>(),
            "Hello, World !"
        );
        assert_eq!(file.line(1).unwrap().iter().collect::<String>(), "");
        assert!(matches!(file.line(2), None))
    }

    #[test]
    fn file_get_len() {
        let file = File::from_string("Hello, World !\n", "test", "test");
        assert_eq!(file.len(), 2);
    }

    #[test]
    fn file_insert() {
        let mut file = File::from_string("Hello, World !\n", "test", "test");
        file.insert(0, 0, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n");
        file.insert(1, 0, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n!");
        file.insert(1, 1, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n!!");

        // Out of bounds line
        file.insert(2, 0, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n!!");

        // Out of bounds col
        file.insert(1, 3, '!');
        assert_eq!(file.to_string(), "!Hello, World !\n!!");
    }

    #[test]
    fn file_delete() {
        let mut file = File::from_string("HW\n", "test", "test");
        file.delete(0, 1);
        assert_eq!(file.to_string(), "W\n");
        file.delete(0, 1);
        assert_eq!(file.to_string(), "\n");
    }

    #[test]
    fn file_delete_out_of_bounds() {
        let mut file = File::from_string("HW\n", "test", "test");
        file.delete(1, 1);
        assert_eq!(file.to_string(), "HW\n");
    }

    #[test]
    fn file_delete_beginning_of_line() {
        let mut file = File::from_string("HW\nGuys !", "test", "test");
        file.delete(1, 0);
        assert_eq!(file.to_string(), "HWGuys !");
    }

    #[test]
    fn file_split_line() {
        let mut file = File::from_string("Hello, World !", "test", "test");
        file.split_line(0, 5);
        assert_eq!(file.to_string(), "Hello\n, World !");
    }

    #[test]
    fn file_split_line_out_of_bounds_line() {
        let mut file = File::from_string("Hello, World !", "test", "test");
        file.split_line(1, 5);
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_split_line_out_of_bounds_lcol() {
        let mut file = File::from_string("Hello, World !", "test", "test");
        file.split_line(0, 20);
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_from_sting_with_tabs() {
        let file = File::from_string("Hello,\tWorld !", "test", "test");
        assert_eq!(file.to_string(), "Hello,    World !");
    }
}
