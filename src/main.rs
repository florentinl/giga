mod editor;
mod buffer;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 2 {
        println!("Usage: giga [file]");
        std::process::exit(1);
    }

    // Optional file to edit
    let file: Option<&str> = args.get(1).map(|s| s.as_str());

    let editor = match file {
        Some(path) => editor::Editor::open(path),
        None => Ok(editor::Editor::new()),
    };

    match editor {
        Ok(mut editor) => {
            editor.run();
        },
        Err(e) => {
            eprintln!("Error: {}", e);
        },
    }
}
