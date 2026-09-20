use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::catalog::{Catalog, Profile, Runtime, invalid, resolve};

pub(crate) fn run(catalog: &Catalog, profile: &Profile) -> io::Result<()> {
    if let Some(model) = &profile.model {
        let model = resolve(&catalog.directory, model)?;
        if !model.is_file() {
            return Err(invalid(format!(
                "model file does not exist: {}",
                model.display()
            )));
        }
    }
    let error = command(catalog, profile)?.exec();
    Err(io::Error::new(
        error.kind(),
        format!("could not launch '{}': {error}", profile.id),
    ))
}

pub(crate) fn preview(catalog: &Catalog, profile: &Profile) -> io::Result<String> {
    let command = command(catalog, profile)?;
    let words = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|word| quote(&word.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ");
    match command.get_current_dir() {
        Some(directory) => Ok(format!(
            "cd {} && {words}",
            quote(&directory.to_string_lossy())
        )),
        None => Ok(words),
    }
}

fn quote(word: &str) -> String {
    if !word.is_empty()
        && word
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_./:-".contains(character))
    {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', "'\\''"))
    }
}

fn command(catalog: &Catalog, profile: &Profile) -> io::Result<Command> {
    let settings = catalog
        .config
        .runtimes
        .get(&profile.runtime)
        .ok_or_else(|| {
            invalid(format!(
                "runtime not configured for profile '{}'",
                profile.id
            ))
        })?;
    let resolved = resolve(&catalog.directory, &settings.executable)?;
    let executable = if settings.executable.components().count() > 1 || resolved.exists() {
        resolved
    } else {
        settings.executable.clone()
    };
    let mut command = Command::new(executable);
    if profile.runtime == Runtime::Omlx {
        command.arg("serve");
    } else {
        let directory = match &settings.working_dir {
            Some(path) => resolve(&catalog.directory, path)?,
            None => catalog.directory.clone(),
        };
        command.current_dir(directory);
        let model = profile
            .model
            .as_ref()
            .ok_or_else(|| invalid(format!("profile '{}' requires a model path", profile.id)))?;
        let model = resolve(&catalog.directory, model)?;
        command.arg("-m").arg(model).args(&profile.args);
    }
    Ok(command)
}
