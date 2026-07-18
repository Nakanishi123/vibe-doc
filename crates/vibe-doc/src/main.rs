mod api;
mod cli;
mod embedded_ui;
mod server;

fn main() {
    let command = std::env::args().nth(1).unwrap_or_else(|| "help".to_owned());
    cli::run(&command);
}
