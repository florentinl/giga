use crate::file::File;

/// The View struct represents the actual portion of the File being displayed.
pub struct View {
    /// The file being displayed
    file: File,
    /// The line number of the first line being displayed
    start_line: usize,
    /// The column number of the first column being displayed
    start_col: usize,
    /// The number of lines being displayed
    height: usize,
    /// The number of columns being displayed
    width: usize,
}

impl View {
    /// Create a new View
    pub fn new(file: File, height: usize, width: usize) -> Self {
        Self {
            file,
            start_line: 0,
            start_col: 0,
            height,
            width,
        }
    }
}

impl ToString for View {
    fn to_string(&self) -> String {
        self.file
            .content
            .iter()
            .skip(self.start_line)
            .take(self.height)
            .map(|line| {
                let start = self.start_col.min(line.len());
                let end = (self.start_col + self.width).min(line.len());

                String::from_utf8_lossy(&line[start..end]).to_string()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_new() {
        let view = View::new(File::new(), 10, 10);
        assert_eq!(view.start_line, 0);
        assert_eq!(view.start_col, 0);
        assert_eq!(view.height, 10);
        assert_eq!(view.width, 10);
    }

    #[test]
    fn view_to_string() {
        let view = View::new(File::from_bytes(b"Hello, World !\n"), 1, 10);
        assert_eq!(view.to_string(), "Hello, Wor");
    }
}
