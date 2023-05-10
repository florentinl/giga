use std::path::Path;

/// Editor structure
/// represents the state of the program
pub struct Editor {
    file_name: Option<String>,
    content: Vec<Vec<u8>>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            file_name: None,
            content: Vec::new(),
        }
    }

    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read(path)?;
        let content = content.split(|&c| c == b'\n').map(|s| s.to_vec()).collect();

        let file_name = Path::new(path)
            .file_name()
            .map(|s| s.to_str().map(|s| s.to_string()))
            .flatten();

        Ok(Self { file_name, content })
    }

    pub fn run(&mut self) {
        // Print the name of the file if it exists
        if let Some(file_name) = &self.file_name {
            println!("Editing {}", file_name);
        }
        // Print the content for now
        for line in &self.content {
            let line = String::from_utf8_lossy(line);
            println!("{}", line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_new_empty() {
        let editor = Editor::new();
        assert_eq!(editor.content.len(), 0);
        assert_eq!(editor.file_name, None)
    }

    #[test]
    fn editor_open() {
        let path = "tests/sample.txt";
        let editor = Editor::open(path);
        assert!(editor.is_ok());

        let editor = editor.unwrap();

        let expected = vec!["Hello, World !".as_bytes(), "".as_bytes()];
        assert_eq!(editor.content, expected);
        assert_eq!(editor.file_name, Some("sample.txt".to_string()));
    }

    #[test]
    fn editor_open_error() {
        let path = "tests/does_not_exist.txt";
        let editor = Editor::open(path);
        assert!(editor.is_err());
    }
}
