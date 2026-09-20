use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, value};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Runtime {
    Dwarfstar,
    LlamaCpp,
    Omlx,
}

impl Runtime {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Dwarfstar => "DwarfStar",
            Self::LlamaCpp => "llama.cpp",
            Self::Omlx => "oMLX",
        }
    }

    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Dwarfstar => "dwarfstar",
            Self::LlamaCpp => "llama_cpp",
            Self::Omlx => "omlx",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeConfig {
    pub(crate) executable: PathBuf,
    #[serde(default)]
    pub(crate) model_dirs: Vec<PathBuf>,
    pub(crate) working_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) runtime: Runtime,
    pub(crate) model: Option<PathBuf>,
    #[serde(default)]
    pub(crate) args: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) runtimes: BTreeMap<Runtime, RuntimeConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) profiles: Vec<Profile>,
}

impl Config {
    fn validate(&self) -> io::Result<()> {
        for (&runtime, settings) in &self.runtimes {
            if settings.executable.as_os_str().is_empty() {
                return Err(invalid(format!(
                    "{} requires an executable",
                    runtime.name()
                )));
            }
            if runtime == Runtime::Omlx
                && (!settings.model_dirs.is_empty() || settings.working_dir.is_some())
            {
                return Err(invalid(
                    "configure oMLX in oMLX; its runtime entry accepts only an executable",
                ));
            }
        }
        let mut ids = BTreeSet::new();
        let mut omlx_seen = false;
        for profile in &self.profiles {
            if profile.id.trim().is_empty() || profile.name.trim().is_empty() {
                return Err(invalid("profile IDs and names must not be empty"));
            }
            if !ids.insert(&profile.id) {
                return Err(invalid(format!("duplicate profile ID '{}'", profile.id)));
            }
            if !self.runtimes.contains_key(&profile.runtime) {
                return Err(invalid(format!(
                    "runtime not configured for profile '{}'",
                    profile.id
                )));
            }
            if profile.runtime == Runtime::Omlx {
                if profile.model.is_some() || !profile.args.is_empty() {
                    return Err(invalid(
                        "configure oMLX in oMLX; its launch profile cannot set a model or arguments",
                    ));
                }
                if omlx_seen {
                    return Err(invalid("oMLX has one configured-models launch profile"));
                }
                omlx_seen = true;
            } else if profile
                .model
                .as_ref()
                .is_none_or(|path| path.as_os_str().is_empty())
            {
                return Err(invalid(format!(
                    "profile '{}' requires a model path",
                    profile.id
                )));
            }
        }
        Ok(())
    }
}

pub(crate) struct Catalog {
    pub(crate) config: Config,
    pub(crate) directory: PathBuf,
    path: PathBuf,
    document: DocumentMut,
}

pub(crate) fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

pub(crate) fn resolve(directory: &Path, path: &Path) -> io::Result<PathBuf> {
    if let Ok(relative) = path.strip_prefix("~") {
        let home = std::env::var_os("HOME").ok_or_else(|| invalid("HOME is not set"))?;
        return Ok(PathBuf::from(home).join(relative));
    }
    Ok(directory.join(path))
}

pub(crate) fn default_path() -> io::Result<PathBuf> {
    if let Some(base) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(base).join("lml/config.toml"));
    }
    resolve(Path::new("."), Path::new("~/.config/lml/config.toml"))
}

impl Catalog {
    pub(crate) fn load(path: &Path) -> io::Result<Self> {
        let path = fs::canonicalize(path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "{}: {error}; run `lml setup` to create a catalog",
                    path.display()
                ),
            )
        })?;
        let source = fs::read_to_string(&path)?;
        let config: Config = toml_edit::de::from_str(&source)
            .map_err(|error| invalid(format!("{}: {error}", path.display())))?;
        config.validate()?;
        let document = source
            .parse::<DocumentMut>()
            .map_err(|error| invalid(error.to_string()))?;
        let directory = path
            .parent()
            .ok_or_else(|| invalid("configuration has no parent directory"))?
            .to_owned();
        Ok(Self {
            config,
            directory,
            path,
            document,
        })
    }

    pub(crate) fn import(&mut self) -> io::Result<usize> {
        let mut additions = Vec::new();
        let mut represented = BTreeSet::new();
        let mut ids: BTreeSet<_> = self
            .config
            .profiles
            .iter()
            .map(|profile| profile.id.clone())
            .collect();
        for profile in &self.config.profiles {
            if let Some(model) = &profile.model {
                let path = resolve(&self.directory, model)?;
                let identity = match fs::canonicalize(&path) {
                    Ok(path) => path,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => path,
                    Err(error) => return Err(error),
                };
                represented.insert((profile.runtime, identity));
            }
        }
        for (&runtime, settings) in &self.config.runtimes {
            if runtime == Runtime::Omlx {
                if !self
                    .config
                    .profiles
                    .iter()
                    .any(|profile| profile.runtime == Runtime::Omlx)
                {
                    let mut id = "omlx".to_owned();
                    let mut suffix = 2;
                    while !ids.insert(id.clone()) {
                        id = format!("omlx-{suffix}");
                        suffix += 1;
                    }
                    additions.push(Profile {
                        id,
                        name: "oMLX — configured models".into(),
                        runtime,
                        model: None,
                        args: Vec::new(),
                    });
                }
                continue;
            }
            let mut models = BTreeSet::new();
            let mut visited = BTreeSet::new();
            for directory in &settings.model_dirs {
                discover(
                    &resolve(&self.directory, directory)?,
                    &mut visited,
                    &mut models,
                )?;
            }
            for path in models {
                if !represented.insert((runtime, path.clone())) {
                    continue;
                }
                let name = path
                    .file_stem()
                    .ok_or_else(|| invalid("model has no file name"))?
                    .to_string_lossy()
                    .into_owned();
                let stem = format!("{}-{}", runtime.key().replace('_', "-"), slug(&name));
                let mut id = stem.clone();
                let mut suffix = 2;
                while !ids.insert(id.clone()) {
                    id = format!("{stem}-{suffix}");
                    suffix += 1;
                }
                additions.push(Profile {
                    id,
                    name,
                    runtime,
                    model: Some(path),
                    args: Vec::new(),
                });
            }
        }
        let count = additions.len();
        for profile in additions {
            let empty = self
                .document
                .get("profiles")
                .and_then(Item::as_array)
                .filter(|array| array.is_empty());
            let comment = empty
                .and_then(|array| array.decor().suffix())
                .and_then(|suffix| suffix.as_str())
                .map(str::to_owned);
            if self.document.get("profiles").is_none() || empty.is_some() {
                self.document["profiles"] = Item::ArrayOfTables(ArrayOfTables::new());
            }
            let tables = self.document["profiles"]
                .as_array_of_tables_mut()
                .ok_or_else(|| invalid("profiles must use [[profiles]] tables"))?;
            let mut table = Table::new();
            if let Some(comment) = comment {
                table.decor_mut().set_prefix(format!("{comment}\n"));
            }
            table["id"] = value(&profile.id);
            table["name"] = value(&profile.name);
            table["runtime"] = value(profile.runtime.key());
            if let Some(model) = &profile.model {
                table["model"] = value(
                    model
                        .to_str()
                        .ok_or_else(|| invalid("model path is not valid UTF-8"))?,
                );
            }
            table["args"] = value(Array::new());
            tables.push(table);
            self.config.profiles.push(profile);
        }
        if count > 0 {
            let mut file = tempfile::NamedTempFile::new_in(&self.directory)?;
            file.write_all(self.document.to_string().as_bytes())?;
            file.as_file().sync_all()?;
            file.persist(&self.path).map_err(|error| error.error)?;
        }
        Ok(count)
    }
}

fn discover(
    directory: &Path,
    visited: &mut BTreeSet<PathBuf>,
    models: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    let directory = fs::canonicalize(directory).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("model source {}: {error}", directory.display()),
        )
    })?;
    if !visited.insert(directory.clone()) {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            discover(&path, visited, models)?;
        } else if metadata.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("gguf"))
        {
            let model = fs::canonicalize(path)?;
            if is_primary_model(&model) {
                models.insert(model);
            }
        }
    }
    Ok(())
}

fn is_primary_model(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return true;
    };
    let stem = stem.to_ascii_lowercase();
    if stem.starts_with("mmproj") || stem.contains("-mmproj") {
        return false;
    }
    if let Some((prefix, count)) = stem.rsplit_once("-of-")
        && let Some((_, part)) = prefix.rsplit_once('-')
        && part.len() == 5
        && count.len() == 5
        && part.bytes().all(|byte| byte.is_ascii_digit())
        && count.bytes().all(|byte| byte.is_ascii_digit())
    {
        return part == "00001";
    }
    true
}

fn slug(name: &str) -> String {
    let slug = name
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "model".into()
    } else {
        slug
    }
}
