use std::process::exit;

use crate::{command::Command, file::File, tui::Tui, view::View};
use termion::input::TermRead;

/// Editor structure
/// represents the state of the program
pub struct Editor {
    /// The name of the file being edited
    file_name: Option<String>,
    /// The current view of the file
    view: View,
    /// The Tui responsible for drawing the editor
    tui: Tui,

    /// The mode of the editor
    mode: Mode,

    /// Staging area for history (a single CommandBlock)
    last_forward_commands: Command,
    last_backward_commands: Command,

    /// Command history
    forward_history: Vec<Command>,
    backward_history: Vec<Command>,

    /// History index (used to navigate the history)
    history_index: usize,
}

/// Mode of the editor
pub enum Mode {
    /// Normal mode
    Normal,
    /// Insert mode
    Insert,
}

impl Editor {
    /// Create a new editor
    pub fn new(file_name: Option<&str>) -> Self {
        Self {
            file_name: file_name.map(|s| s.to_string()),
            view: View::new(File::new(), 0, 0),
            tui: Tui::new(),
            mode: Mode::Normal,
            last_forward_commands: Command::CommandBlock(vec![]),
            last_backward_commands: Command::CommandBlock(vec![]),
            forward_history: vec![],
            backward_history: vec![],
            history_index: 0,
        }
    }

    /// Open a file in the editor
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let content = File::from_string(&content);
        let view = View::new(content, 0, 0);

        Ok(Self {
            file_name: Some(path.to_string()),
            view,
            tui: Tui::new(),
            mode: Mode::Normal,
            last_forward_commands: Command::CommandBlock(vec![]),
            last_backward_commands: Command::CommandBlock(vec![]),
            forward_history: vec![Command::CommandBlock(vec![])],
            backward_history: vec![Command::CommandBlock(vec![])],
            history_index: 0,
        })
    }
    fn save(&self) {
        if let Some(path) = &self.file_name {
            let content = self.view.dump_file();
            std::fs::write(path.clone() + ".tmp", content).unwrap_or_default();
            std::fs::rename(path.clone() + ".tmp", path).unwrap_or_default();
        }
    }

    fn terminate(&mut self) {
        self.tui.cleanup();
        exit(0);
    }

    /// Execute an editor command
    /// - Quit: exit the program
    /// - Move: move the cursor
    /// - Save: save the file
    /// - ToggleMode: toogle editor mode
    /// - Insert: insert a character
    /// - Delete: delete a character
    fn execute(&mut self, cmd: Command) {
        // Execute the command and get the inverse command if it exists
        let inverse_cmd: Option<Command> = self.execute_and_invert(&cmd);

        if matches!(self.mode, Mode::Insert) {
            // If we are in insert mode, we need to save the command in the forward_history
            // except if it is a toggle mode command.
            if let Command::CommandBlock(ref mut cmds) = self.last_forward_commands {
                cmds.push(cmd.clone());
            }

            // If we are in insert mode and the command is not a toggle mode command,
            // we need to save the inverse command in the backward_history
            if let Command::CommandBlock(ref mut cmds) = self.last_backward_commands {
                assert!(inverse_cmd.is_some(), "No inverse command for {:?}", cmd);
                cmds.push(inverse_cmd.unwrap());
            }
        }
    }

    /// Execute and returns the inverse of the command
    /// All Insert mode commands must be invertible
    fn execute_and_invert(&mut self, cmd: &Command) -> Option<Command> {
        let inverse_cmd = match cmd {
            Command::Quit => {
                self.terminate();
                None
            }
            Command::Save => {
                self.save();
                None
            }
            Command::ToggleMode => {
                self.toggle_mode();
                Some(Command::ToggleMode)
            }
            Command::Move(x, y) => {
                let (dx, dy) = self.view.navigate(*x, *y);
                Some(Command::Move(-dx, -dy))
            }
            Command::Insert(c) => {
                self.view.insert(*c);
                Some(Command::Delete)
            }
            Command::InsertNewLine => {
                self.view.insert_new_line();
                Some(Command::Delete)
            }
            Command::Delete => match self.view.delete() {
                '\n' => Some(Command::InsertNewLine),
                '\0' => Some(Command::CommandBlock(vec![])),
                c => Some(Command::Insert(c)),
            },
            Command::Undo => {
                self.undo();
                None
            }
            Command::Redo => {
                self.redo();
                None
            }
            Command::CommandBlock(cmds) => cmds
                .into_iter()
                .map(|cmd| self.execute_and_invert(cmd))
                .collect::<Option<Vec<_>>>()
                .map(|cmds| Command::CommandBlock(cmds)),
        };
        inverse_cmd
    }

    /// Undo the last command
    fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let cmd = self.backward_history[self.history_index].clone();
            self.execute_and_invert(&cmd);
        }
    }

    /// Redo the last command
    fn redo(&mut self) {
        if self.history_index < self.forward_history.len() {
            let cmd = self.forward_history[self.history_index].clone();
            self.execute_and_invert(&cmd);
            self.history_index += 1;
        }
    }

    /// Toggle the mode of the editor between normal and insert
    fn toggle_mode(&mut self) {
        match self.mode {
            Mode::Normal => self.insert_mode(),
            Mode::Insert => self.normal_mode(),
        }
    }

    /// Go to insert mode
    fn insert_mode(&mut self) {
        // We truncate the history to the current history index
        self.truncate_history();

        // Actually go to insert mode
        self.mode = Mode::Insert;
    }

    /// Truncate the history to the current history index
    fn truncate_history(&mut self) {
        self.forward_history.truncate(self.history_index);
        self.backward_history.truncate(self.history_index);
    }

    /// Go to normal mode
    fn normal_mode(&mut self) {
        // Commit the staging area to the history
        self.commit_history();

        // Clear the staging area
        self.last_forward_commands = Command::CommandBlock(vec![]);
        self.last_backward_commands = Command::CommandBlock(vec![]);

        self.mode = Mode::Normal;
    }

    /// Commit the current commands to the history
    fn commit_history(&mut self) {
        // Flatten the last commands and remove all the toggle mode commands
        let forward_commit = self
            .last_forward_commands
            .flatten()
            .filter(|cmd| !matches!(cmd, Command::ToggleMode));
        // Reverse the backward commands to undo them in the right order
        let backward_commit = self
            .last_backward_commands
            .flatten()
            .filter(|cmd| !matches!(cmd, Command::ToggleMode))
            .rev();

        // No need to add an empty commit to the history
        if !forward_commit.is_empty() {
            // Add the commits to the history
            self.forward_history.push(forward_commit);
            self.backward_history.push(backward_commit);

            self.history_index += 1;
        }
    }

    /// Run the editor loop
    pub fn run(&mut self) {
        // set view size
        let (width, height) = self.tui.get_term_size();
        // height - 1 to leave space for the status bar
        // width - 3 to leave space for the line numbers
        self.view
            .resize((height - 1) as usize, (width - 4) as usize);

        // draw initial view
        self.tui.clear();
        self.tui.draw_view(&self.view, &self.file_name, &self.mode);

        let stdin = std::io::stdin().keys();

        for c in stdin {
            if let Ok(c) = c {
                if let Ok(cmd) = Command::parse(c, &self.mode) {
                    self.execute(cmd);
                    self.tui.draw_view(&self.view, &self.file_name, &self.mode)
                }
            }
        }
    }
}
