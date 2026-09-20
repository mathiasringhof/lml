use std::fs;
use std::io::{self, BufRead, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::catalog::{Catalog, Config, Runtime, RuntimeConfig, invalid, resolve};

pub(crate) fn run(path: &Path) -> io::Result<()> {
    if path.try_exists()? {
        return Err(invalid(format!(
            "{} already exists; edit it or run `lml import`",
            path.display()
        )));
    }
    let current = std::env::current_dir()?;
    let path = resolve(&current, path)?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut config = Config::default();
    writeln!(
        output,
        "Set up lml. No runtimes or models will be installed. Enter paths without shell quotes."
    )?;
    let checkout = ask(
        &mut input,
        &mut output,
        "DwarfStar checkout directory (blank to skip): ",
    )?;
    if !checkout.is_empty() {
        let checkout = fs::canonicalize(resolve(&current, Path::new(&checkout))?)?;
        let executable = checkout.join("ds4-server");
        check_executable(&executable)?;
        let model_dirs = directories(
            &mut input,
            &mut output,
            &current,
            "DwarfStar",
            Some(&checkout.join("gguf")),
        )?;
        config.runtimes.insert(
            Runtime::Dwarfstar,
            RuntimeConfig {
                executable,
                model_dirs,
                working_dir: Some(checkout),
            },
        );
    }
    if let Some(executable) = executable(&mut input, &mut output, &current, "llama-server")? {
        let model_dirs = directories(&mut input, &mut output, &current, "llama.cpp", None)?;
        config.runtimes.insert(
            Runtime::LlamaCpp,
            RuntimeConfig {
                executable,
                model_dirs,
                working_dir: None,
            },
        );
    }
    if let Some(executable) = executable(&mut input, &mut output, &current, "omlx")? {
        config.runtimes.insert(
            Runtime::Omlx,
            RuntimeConfig {
                executable,
                model_dirs: Vec::new(),
                working_dir: None,
            },
        );
    }
    let parent = path
        .parent()
        .ok_or_else(|| invalid("configuration has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let staging = tempfile::tempdir_in(parent)?;
    let staging_path = staging.path().join("catalog.toml");
    let document =
        toml_edit::ser::to_document(&config).map_err(|error| invalid(error.to_string()))?;
    fs::write(
        &staging_path,
        format!("# lml catalog: edit profiles here; `lml import` only adds entries.\n{document}"),
    )?;
    let mut catalog = Catalog::load(&staging_path)?;
    let count = catalog.import()?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(&fs::read(staging_path)?)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(&path).map_err(|error| error.error)?;
    writeln!(
        output,
        "Created {} with {count} launch profile(s). Run `lml` to open the picker.",
        path.display()
    )?;
    Ok(())
}

fn ask(input: &mut impl BufRead, output: &mut impl Write, prompt: &str) -> io::Result<String> {
    write!(output, "{prompt}")?;
    output.flush()?;
    let mut answer = String::new();
    if input.read_line(&mut answer)? == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "setup cancelled before all questions were answered; configuration was not saved",
        ));
    }
    Ok(answer.trim().to_owned())
}

fn directories(
    input: &mut impl BufRead,
    output: &mut impl Write,
    current: &Path,
    runtime: &str,
    default: Option<&Path>,
) -> io::Result<Vec<PathBuf>> {
    let mut directories = Vec::new();
    loop {
        let prompt = if directories.is_empty() {
            match default {
                Some(path) => format!(
                    "{runtime} model directory [{}] (blank accepts and finishes): ",
                    path.display()
                ),
                None => format!("{runtime} model directory (blank to finish): "),
            }
        } else {
            format!("Another {runtime} model directory (blank to finish): ")
        };
        let answer = ask(input, output, &prompt)?;
        if answer.is_empty() {
            if directories.is_empty()
                && let Some(path) = default
            {
                directories.push(fs::canonicalize(path)?);
            }
            break;
        }
        let path = fs::canonicalize(resolve(current, Path::new(&answer))?)?;
        if !path.is_dir() {
            return Err(invalid(format!(
                "{} is not a model directory",
                path.display()
            )));
        }
        if !directories.contains(&path) {
            directories.push(path);
        }
    }
    Ok(directories)
}

fn executable(
    input: &mut impl BufRead,
    output: &mut impl Write,
    current: &Path,
    name: &str,
) -> io::Result<Option<PathBuf>> {
    let detected = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|path| path.join(name))
            .find(|path| check_executable(path).is_ok())
    });
    let prompt = match &detected {
        Some(path) => format!(
            "{name} executable [{}] (blank accepts, '-' skips): ",
            path.display()
        ),
        None => format!("{name} executable (blank or '-' to skip): "),
    };
    let answer = ask(input, output, &prompt)?;
    let chosen = if answer == "-" {
        None
    } else if answer.is_empty() {
        detected
    } else {
        Some(resolve(current, Path::new(&answer))?)
    };
    chosen
        .map(|path| {
            check_executable(&path)?;
            fs::canonicalize(path)
        })
        .transpose()
}

fn check_executable(path: &Path) -> io::Result<()> {
    let metadata = fs::metadata(path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("executable {}: {error}", path.display()),
        )
    })?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(invalid(format!(
            "{} is not an executable file",
            path.display()
        )));
    }
    Ok(())
}
