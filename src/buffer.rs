/// The Buffer is the in-memory representation of the full file being edited.
/// It is a vector of lines, each line being a vector of bytes.
pub struct Buffer {
    content: Vec<Vec<u8>>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let content = bytes
            .split(|&c| c == b'\n')
            .map(|line| line.to_vec())
            .collect();

        Self { content }
    }
}

impl ToString for Buffer {
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
    fn buffer_new_empty() {
        let buffer = Buffer::new();
        assert_eq!(buffer.content.len(), 0);
    }

    #[test]
    fn buffer_from_bytes() {
        let buffer = Buffer::from_bytes(b"Hello, World !");
        assert_eq!(buffer.content.len(), 1);
        assert_eq!(buffer.content[0], "Hello, World !".as_bytes());
    }

    #[test]
    fn buffer_to_string() {
        let buffer = Buffer::from_bytes(b"Hello, World !");
        assert_eq!(buffer.to_string(), "Hello, World !");
    }
}
