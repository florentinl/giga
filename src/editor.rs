/// Editor structure
/// represents the state of the program
pub struct Editor {
    content: Vec<Vec<u8>>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read(path)?;
        let content = content.split(|&c| c == b'\n')
            .map(|s| s.to_vec())
            .collect();

        Ok(Self {
            content,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_new_empty() {
        let editor = Editor::new();
        assert_eq!(editor.content.len(), 0);
    }

    #[test]
    fn editor_open() {
        let path = "tests/sample.txt";
        let editor = Editor::open(path);
        assert!(editor.is_ok());

        let editor = editor.unwrap();

        let expected = vec![
            "Hello, World !".as_bytes(),
            "".as_bytes(),
        ];
        assert_eq!(editor.content, expected);
    }

    #[test]
    fn editor_open_error() {
        let path = "tests/does_not_exist.txt";
        let editor = Editor::open(path);
        assert!(editor.is_err());
    }
}
