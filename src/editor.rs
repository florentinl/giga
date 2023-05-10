use std::path::Path;

use crate::buffer::Buffer;

/// Editor structure
/// represents the state of the program
pub struct Editor {
    file_name: Option<String>,
    buffer: Buffer,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            file_name: None,
            buffer: Buffer::new(),
        }
    }

    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read(path)?;
        let content = Buffer::from_bytes(&content);

        let file_name = Path::new(path)
            .file_name()
            .map(|s| s.to_str().map(|s| s.to_string()))
            .flatten();

        Ok(Self { file_name, buffer: content })
    }

    pub fn run(&mut self) {
        // Print the name of the file if it exists
        if let Some(file_name) = &self.file_name {
            println!("Editing {}", file_name);
        }
        // Print the content for now
        println!("{}", self.buffer.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_new_empty() {
        let editor = Editor::new();
        assert_eq!(editor.buffer.to_string(), "");
        assert_eq!(editor.file_name, None)
    }

    #[test]
    fn editor_open() {
        let path = "tests/sample.txt";
        let editor = Editor::open(path);
        assert!(editor.is_ok());

        let editor = editor.unwrap();

        let expected = "Hello, World !\n";
        assert_eq!(editor.buffer.to_string(), expected);
        assert_eq!(editor.file_name, Some("sample.txt".to_string()));
    }

    #[test]
    fn editor_open_error() {
        let path = "tests/does_not_exist.txt";
        let editor = Editor::open(path);
        assert!(editor.is_err());
    }
}
