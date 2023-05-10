/// The File structure is the in-memory representation of the full file being edited.
/// It is a vector of lines, each line being a vector of bytes.
pub struct File {
    content: Vec<Vec<u8>>,
}

impl File {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    /// Create a File abstraction from a byte array
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let content = bytes
            .split(|&c| c == b'\n')
            .map(|line| line.to_vec())
            .collect();

        Self { content }
    }

    /// Get the nth line of the file
    pub fn get_line(&self, index: usize) -> Option<Vec<u8>> {
        self.content.get(index).cloned()
    }

    /// Get the number of lines in the file
    pub fn len(&self) -> usize {
        self.content.len()
    }
}

impl ToString for File {
    fn to_string(&self) -> String {
        self.content
            .iter()
            .map(|line| String::from_utf8_lossy(line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_new_empty() {
        let file = File::new();
        assert_eq!(file.content.len(), 0);
    }

    #[test]
    fn file_from_bytes() {
        let file = File::from_bytes(b"Hello, World !");
        assert_eq!(file.content.len(), 1);
        assert_eq!(file.content[0], "Hello, World !".as_bytes());
    }

    #[test]
    fn file_to_string() {
        let file = File::from_bytes(b"Hello, World !");
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_get_line() {
        let file = File::from_bytes(b"Hello, World !\n");
        assert_eq!(
            file.get_line(0),
            Some("Hello, World !".as_bytes().to_vec())
        );
        assert_eq!(file.get_line(1), Some("".as_bytes().to_vec()));
        assert_eq!(file.get_line(2), None);
    }

    #[test]
    fn file_get_len() {
        let file = File::from_bytes(b"Hello, World !\n");
        assert_eq!(file.len(), 2);
    }
}
