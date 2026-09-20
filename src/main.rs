mod app;
mod catalog;
mod launch;
mod setup;

use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Use a different TOML catalog.
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    /// Create a catalog with guided runtime and model-directory setup.
    Setup,
    /// Add newly discovered models without changing saved profiles.
    Import,
    /// Launch a saved profile directly, without opening the picker.
    Run { id: String },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lml: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> io::Result<()> {
    if cli.command.is_none() && (!io::stdin().is_terminal() || !io::stdout().is_terminal()) {
        return Err(io::Error::other(
            "the TUI requires an interactive terminal (stdin and stdout); use --help for usage",
        ));
    }
    let path = match cli.config {
        Some(path) => path,
        None => catalog::default_path()?,
    };
    let path = catalog::resolve(&std::env::current_dir()?, &path)?;
    match cli.command {
        Some(Action::Setup) => setup::run(&path),
        Some(Action::Import) => {
            let count = catalog::Catalog::load(&path)?.import()?;
            println!("Added {count} launch profile(s).");
            Ok(())
        }
        Some(Action::Run { id }) => {
            let catalog = catalog::Catalog::load(&path)?;
            let profile = catalog
                .config
                .profiles
                .iter()
                .find(|profile| profile.id == id)
                .ok_or_else(|| catalog::invalid(format!("unknown launch profile '{id}'")))?;
            launch::run(&catalog, profile)
        }
        None => {
            let catalog = catalog::Catalog::load(&path)?;
            let entries = catalog
                .config
                .profiles
                .iter()
                .map(|profile| {
                    Ok(app::Entry {
                        id: profile.id.clone(),
                        name: profile.name.clone(),
                        runtime: profile.runtime.name().into(),
                        model: profile
                            .model
                            .as_ref()
                            .map(|model| {
                                catalog::resolve(&catalog.directory, model)
                                    .map(|path| path.display().to_string())
                            })
                            .transpose()?,
                        command: launch::preview(&catalog, profile)?,
                    })
                })
                .collect::<io::Result<Vec<_>>>()?;
            let mut picker = app::Picker::new(entries);
            // Ratatui restores the terminal before the server inherits it, including on errors.
            let selected = ratatui::run(|terminal| app::run(terminal, &mut picker))?;
            if let Some(profile) = selected.and_then(|index| catalog.config.profiles.get(index)) {
                launch::run(&catalog, profile)?;
            }
            Ok(())
        }
    }
}
