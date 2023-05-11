mod command;
mod editor;
mod file;
mod tui;
mod view;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 2 {
        println!("Usage: giga [file]");
        std::process::exit(1);
    }

    // Optional file to edit
    let file: Option<&str> = args.get(1).map(|s| s.as_str());

    let mut editor = match file {
        Some(path) => editor::Editor::open(path).unwrap_or(editor::Editor::new(file)),
        None => editor::Editor::new(Some("Newfile")),
    };
    editor.run();
}
