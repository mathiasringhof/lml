mod app;

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Cli {}

fn main() -> ExitCode {
    Cli::parse();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lml: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other(
            "the TUI requires an interactive terminal (stdin and stdout); use --help for usage",
        ));
    }

    ratatui::run(app::run)
}
