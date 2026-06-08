//! Command-line entry point for `vdoc`.

mod args;
mod commands;
mod error;
mod format;

use args::Cli;
use clap::Parser;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match commands::run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error::CliError::ReportedIssues) => ExitCode::from(1),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn depends_on_core_crate() {
        assert_eq!(vibe_doc_core::CRATE_NAME, "vibe-doc-core");
    }
}
