mod api;
mod cli;
mod embedded_ui;
mod init;
mod refs;
mod server;

use clap::Parser;

fn main() -> std::process::ExitCode {
    cli::run(cli::Cli::parse())
}
