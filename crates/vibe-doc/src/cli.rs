pub fn run(command: &str) {
    match command {
        "serve" | "lint" | "tag" | "next-index" => {
            eprintln!("`{command}` is scaffolded but not implemented yet.");
        }
        _ => {
            println!("vibe-doc\n\nCommands: serve, lint, tag, next-index");
        }
    }
}
