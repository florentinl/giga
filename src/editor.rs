use std::thread::sleep;

use crate::{
    file::File,
    view::View, tui::Tui,
};

/// Editor structure
/// represents the state of the program
pub struct Editor {
    /// The name of the file being edited
    file_name: Option<String>,
    /// The current view of the file
    view: View,
    /// The Tui responsible for drawing the editor
    tui: Tui,
}

impl Editor {
    pub fn new(file_name: Option<&str>) -> Self {
        Self {
            file_name: file_name.map(|s| s.to_string()),
            view: View::new(File::new(), 10, 20),
            tui: Tui::new(),
        }
    }

    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read(path)?;
        let content = File::from_bytes(&content);
        let view = View::new(content, 10, 20);

        Ok(Self {
            file_name: Some(path.to_string()),
            view,
            tui: Tui::new(),
        })
    }

    pub fn run(&mut self) {
        // set view size
        let (width, height) = self.tui.get_term_size();
        // height - 1 to leave space for the status bar
        self.view.resize((height - 1) as usize, width as usize);

        // draw initial view
        self.tui.clear();
        self.tui.draw_view(&self.view, &self.file_name);
        sleep(core::time::Duration::from_secs(5));
        loop {
            // // parse input events into commands
            // let command = self.tui.read_input();
            // // redraw the view if needed
            // if matches!(command, Some(_)) {
            //     self.tui.draw_view(&self.view, &self.file_name);
            // }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_new_empty() {
        let editor = Editor::new(Some("filename"));
        assert_eq!(editor.view.to_string(), "");
        assert_eq!(editor.file_name, Some("filename".to_string()));
    }

    #[test]
    fn editor_open() {
        let path = "tests/sample.txt";
        let editor = Editor::open(path);
        assert!(editor.is_ok());

        let editor = editor.unwrap();

        let expected = "Hello, World !\n";
        assert_eq!(editor.view.to_string(), expected);
        assert_eq!(editor.file_name, Some("tests/sample.txt".to_string()));
    }

    #[test]
    fn editor_open_error() {
        let path = "tests/does_not_exist.txt";
        let editor = Editor::open(path);
        assert!(editor.is_err());
    }
}
