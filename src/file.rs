use std::vec;

/// The File structure is the in-memory representation of the full file being edited.
/// It is a vector of lines, each line being a vector of bytes.
pub struct File {
    content: Vec<Vec<char>>,
}

impl File {
    pub fn new() -> Self {
        Self {
            content: vec![vec![]],
        }
    }

    /// Create a File abstraction from a string
    pub fn from_string(str: &str) -> Self {
        let content = str.split('\n').map(|line| line.chars().collect()).collect();

        Self { content }
    }

    /// Get the nth line of the file
    pub fn get_line(&self, index: usize) -> Option<Vec<char>> {
        self.content.get(index).cloned()
    }

    /// Get the number of lines in the file
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Insert a char at the given position in the file
    /// - line >= len || col > line_len: do nothing
    /// - else insert the byte at the given position
    pub fn insert(&mut self, line: usize, col: usize, c: char) {
        match self.content.get_mut(line) {
            None => {}
            Some(line) => {
                if col > line.len() {
                    return;
                }
                line.insert(col, c);
            }
        }
    }

    /// Delete a char at the given position
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

    /// Split a line at the given position
    /// - line >= len: do nothing
    /// - col > line_len: do nothing
    /// - else split the line at the given position
    pub fn split_line(&mut self, line: usize, col: usize) {
        match self.content.get_mut(line) {
            None => {}
            Some(vec) => {
                if col > vec.len() {
                    return;
                }
                let new_line = vec.split_off(col);
                self.content.insert(line + 1, new_line);
            }
        }
    }
}

/// Implement the ToString trait for File (used for saving the file)
impl ToString for File {
    fn to_string(&self) -> String {
        self.content
            .iter()
            .map(|line| line.iter().collect::<String>())
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
        let file = File::from_string("Hello, World !");
        assert_eq!(file.content.len(), 1);
        assert_eq!(file.content[0], "Hello, World !".chars().collect::<Vec<_>>());
    }

    #[test]
    fn file_to_string() {
        let file = File::from_string("Hello, World !");
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_get_line() {
        let file = File::from_string("Hello, World !\n");
        assert_eq!(file.get_line(0), Some("Hello, World !".chars().collect()));
        assert_eq!(file.get_line(1), Some("".chars().collect()));
        assert_eq!(file.get_line(2), None);
    }

    #[test]
    fn file_get_len() {
        let file = File::from_string("Hello, World !\n");
        assert_eq!(file.len(), 2);
    }

    #[test]
    fn file_insert() {
        let mut file = File::from_string("Hello, World !\n");
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
        let mut file = File::from_string("HW\n");
        file.delete(0, 1);
        assert_eq!(file.to_string(), "W\n");
        file.delete(0, 1);
        assert_eq!(file.to_string(), "\n");
    }

    #[test]
    fn file_delete_out_of_bounds() {
        let mut file = File::from_string("HW\n");
        file.delete(1, 1);
        assert_eq!(file.to_string(), "HW\n");
    }

    #[test]
    fn file_delete_beginning_of_line() {
        let mut file = File::from_string("HW\nGuys !");
        file.delete(1, 0);
        assert_eq!(file.to_string(), "HWGuys !");
    }

    #[test]
    fn file_split_line() {
        let mut file = File::from_string("Hello, World !");
        file.split_line(0, 5);
        assert_eq!(file.to_string(), "Hello\n, World !");
    }

    #[test]
    fn file_split_line_out_of_bounds_line() {
        let mut file = File::from_string("Hello, World !");
        file.split_line(1, 5);
        assert_eq!(file.to_string(), "Hello, World !");
    }

    #[test]
    fn file_split_line_out_of_bounds_lcol() {
        let mut file = File::from_string("Hello, World !");
        file.split_line(0, 20);
        assert_eq!(file.to_string(), "Hello, World !");
    }
}
