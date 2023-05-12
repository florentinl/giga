use std::vec;

/// The File structure is the in-memory representation of the full file being edited.
/// It is a vector of lines, each line being a vector of bytes.
pub struct File {
    content: Vec<Vec<u8>>,
}

impl File {
    pub fn new() -> Self {
        Self {
            content: vec![vec![]],
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

    /// Insert a byte at the given position
    pub fn insert(&mut self, line: usize, col: usize, c: u8) {
        if line >= self.content.len() {
            self.content.push(vec![c]);
        } else {
            self.content[line].insert(col, c);
        }
    }

    /// Delete a byte at the given position
    /// - col == 0: join the line with the previous one (except if it's the first line)
    /// - 0 < col <= line_len: delete the byte at the given position
    /// - col > line_len: do nothing
    pub fn delete(&mut self, line: usize, col: usize) {
        if line >= self.content.len() {
            return;
        }

        let line_len = self.content[line].len();
        if col == 0 {
            if line > 0 {
                let prev_line = self.content.remove(line);
                if let Some(line) = self.content.get_mut(line - 1) {
                    line.extend(prev_line);
                }
            }
        } else if col <= line_len {
            self.content[line].remove(col - 1);
        }
    }

    pub fn split_line(&mut self, line: usize, col: usize) {
        if line >= self.content.len() {
            return;
        } else {
            let new_line = self.content[line].split_off(col);
            self.content.insert(line + 1, new_line);
        }
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
        assert_eq!(file.content.len(), 1);
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
        assert_eq!(file.get_line(0), Some("Hello, World !".as_bytes().to_vec()));
        assert_eq!(file.get_line(1), Some("".as_bytes().to_vec()));
        assert_eq!(file.get_line(2), None);
    }

    #[test]
    fn file_get_len() {
        let file = File::from_bytes(b"Hello, World !\n");
        assert_eq!(file.len(), 2);
    }

    #[test]
    fn file_insert() {
        let mut file = File::from_bytes(b"Hello, World !\n");
        file.insert(0, 0, b'!');
        assert_eq!(file.to_string(), "!Hello, World !\n");
        file.insert(1, 0, b'!');
        assert_eq!(file.to_string(), "!Hello, World !\n!");
        file.insert(1, 1, b'!');
        assert_eq!(file.to_string(), "!Hello, World !\n!!");
        file.insert(2, 0, b'!');
        assert_eq!(file.to_string(), "!Hello, World !\n!!\n!");
    }

    #[test]
    fn file_delete() {
        let mut file = File::from_bytes(b"HW\n");
        file.delete(0, 1);
        assert_eq!(file.to_string(), "W\n");
        file.delete(0, 1);
        assert_eq!(file.to_string(), "\n");
    }

    #[test]
    fn file_delete_out_of_bounds() {
        let mut file = File::from_bytes(b"HW\n");
        file.delete(1, 1);
        assert_eq!(file.to_string(), "HW\n");
    }

    #[test]
    fn file_delete_beginning_of_line() {
        let mut file = File::from_bytes(b"HW\nGuys !");
        file.delete(1, 0);
        assert_eq!(file.to_string(), "HWGuys !");
    }
}
