use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_lml"))
        .args(args)
        .stdin(Stdio::null())
        .output()
}

#[test]
fn help_works_without_a_terminal() -> io::Result<()> {
    let output = run(&["--help"])?;
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--help"));
    assert!(stdout.contains("--version"));
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn version_works_without_a_terminal() -> io::Result<()> {
    let output = run(&["--version"])?;
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains(env!("CARGO_PKG_VERSION")));
    Ok(())
}

#[test]
fn unknown_argument_fails_before_terminal_setup() -> io::Result<()> {
    let output = run(&["--unknown-option"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--unknown-option"));
    assert!(output.stdout.is_empty());
    Ok(())
}

#[test]
fn redirected_input_fails_without_terminal_escape_sequences() -> io::Result<()> {
    let output = run(&[])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("interactive terminal"));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.contains(&0x1b));
    Ok(())
}

#[test]
fn import_adds_a_model_to_the_editable_catalog() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    fs::create_dir(directory.path().join("models"))?;
    fs::write(directory.path().join("models/Qwen.gguf"), b"GGUF")?;
    let config = directory.path().join("config.toml");
    fs::write(
        &config,
        "# My model catalog\n[runtimes.llama_cpp]\nexecutable = '/usr/bin/true'\nmodel_dirs = ['models']\n",
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .arg("import")
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let saved = fs::read_to_string(&config)?;
    assert!(saved.contains("# My model catalog"));
    assert!(saved.contains("[[profiles]]"));
    assert!(saved.contains("llama-cpp-qwen"));
    assert!(saved.contains("Qwen.gguf"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Added 1"));
    Ok(())
}

#[test]
fn repeated_import_preserves_variants_and_deduplicates_overlapping_sources() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let models = directory.path().join("models");
    fs::create_dir_all(models.join("nested"))?;
    fs::write(models.join("Qwen.gguf"), b"GGUF")?;
    fs::write(models.join("nested/New.gguf"), b"GGUF")?;
    std::os::unix::fs::symlink(models.join("Qwen.gguf"), models.join("alias.gguf"))?;
    std::os::unix::fs::symlink(&models, models.join("nested/cycle"))?;
    let config = directory.path().join("config.toml");
    let original = "# Preserve my tuning\n[runtimes.llama_cpp]\nexecutable = '/usr/bin/true'\nmodel_dirs = ['models', 'models/nested']\n\n[[profiles]]\nid = 'renamed'\nname = 'My tuning'\nruntime = 'llama_cpp'\nmodel = 'models/Qwen.gguf'\nargs = ['--ctx-size', '8192'] # keep this\n\n[[profiles]]\nid = 'variant'\nname = 'Other tuning'\nruntime = 'llama_cpp'\nmodel = 'models/alias.gguf'\nargs = ['--ctx-size', '4096']\n";
    fs::write(&config, original)?;
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_lml"))
            .arg("--config")
            .arg(&config)
            .arg("import")
            .output()
    };
    let output = invoke()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Added 1"));
    let saved = fs::read_to_string(&config)?;
    assert!(saved.starts_with(original));
    assert!(saved.contains("New.gguf"));
    let output = invoke()?;
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Added 0"));
    assert_eq!(fs::read_to_string(&config)?, saved);
    Ok(())
}

#[test]
fn launch_preserves_arguments_logs_working_directory_and_exit_status() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let server = directory.path().join("fake server");
    fs::write(
        &server,
        "#!/bin/sh\npwd\nprintf '<%s>\\n' \"$@\"\nprintf 'server error log\\n' >&2\nexit 17\n",
    )?;
    fs::set_permissions(&server, fs::Permissions::from_mode(0o755))?;
    fs::write(directory.path().join("model with spaces.gguf"), b"GGUF")?;
    let config = directory.path().join("config.toml");
    fs::write(
        &config,
        "[runtimes.dwarfstar]\nexecutable = 'fake server'\nworking_dir = '.'\n[[profiles]]\nid = 'my-model'\nname = 'My model'\nruntime = 'dwarfstar'\nmodel = 'model with spaces.gguf'\nargs = ['--jinja', 'a template.jinja', '$(touch should-not-exist)']\n",
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .args(["run", "my-model"])
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(17),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(
            directory
                .path()
                .canonicalize()?
                .to_str()
                .expect("UTF-8 temp directory")
        )
    );
    assert!(stdout.contains("<-m>\n"));
    assert!(stdout.contains("model with spaces.gguf>\n"));
    assert!(stdout.contains("<a template.jinja>\n<$(touch should-not-exist)>\n"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("server error log"));
    assert!(!directory.path().join("should-not-exist").exists());
    assert!(!output.stdout.contains(&0x1b));
    Ok(())
}

#[test]
fn omlx_rejects_profile_settings_instead_of_overwriting_native_configuration() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let config = directory.path().join("config.toml");
    fs::write(
        &config,
        "[runtimes.omlx]\nexecutable = '/usr/bin/true'\n[[profiles]]\nid = 'omlx'\nname = 'oMLX'\nruntime = 'omlx'\nargs = ['--model-dir', '/some/other/models']\n",
    )?;
    let original = fs::read(&config)?;
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .args(["run", "omlx"])
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("configure oMLX in oMLX"));
    assert_eq!(fs::read(&config)?, original);
    Ok(())
}

#[test]
fn guided_setup_imports_models_and_one_native_omlx_server() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let dwarfstar = directory.path().join("ds4");
    fs::create_dir_all(dwarfstar.join("gguf"))?;
    fs::write(dwarfstar.join("gguf/DeepSeek.gguf"), b"GGUF")?;
    fs::write(dwarfstar.join("ds4-server"), "#!/bin/sh\nexit 0\n")?;
    fs::set_permissions(
        dwarfstar.join("ds4-server"),
        fs::Permissions::from_mode(0o755),
    )?;
    fs::create_dir(directory.path().join("models"))?;
    fs::write(directory.path().join("models/Qwen.gguf"), b"GGUF")?;
    let omlx = directory.path().join("omlx");
    fs::write(&omlx, "#!/bin/sh\nprintf '<%s>\\n' \"$@\"\n")?;
    fs::set_permissions(&omlx, fs::Permissions::from_mode(0o755))?;
    let config = directory.path().join("config.toml");
    let mut child = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .arg("setup")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let answers = format!(
        "{}\n\n/usr/bin/true\n{}\n\n{}\n",
        dwarfstar.display(),
        directory.path().join("models").display(),
        omlx.display()
    );
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(answers.as_bytes())?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let saved = fs::read_to_string(&config)?;
    assert_eq!(saved.matches("[[profiles]]").count(), 3);
    assert!(saved.contains("oMLX — configured models"));
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .args(["run", "omlx"])
        .output()?;
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "<serve>\n");
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .arg("setup")
        .stdin(Stdio::null())
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
    assert_eq!(fs::read_to_string(&config)?, saved);
    Ok(())
}

#[test]
fn ambiguous_and_invalid_catalogs_fail_without_launching_or_rewriting() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let config = directory.path().join("config.toml");
    let runtime = "[runtimes.llama_cpp]\nexecutable = '/usr/bin/true'\n";
    let profile = "[[profiles]]\nid = 'same'\nname = 'Model'\nruntime = 'llama_cpp'\nmodel = 'missing.gguf'\n";
    for (source, expected) in [
        (
            format!("{runtime}{profile}{profile}"),
            "duplicate profile ID",
        ),
        (
            format!("{runtime}[[profiles]]\nid = 'bad'\nname = 'Bad'\nruntime = 'llama_cpp'\n"),
            "requires a model",
        ),
        (profile.to_owned(), "runtime not configured"),
        (
            "[runtimes.omlx]\nexecutable = '/usr/bin/true'\nmodel_dirs = ['/models']\n".into(),
            "configure oMLX in oMLX",
        ),
    ] {
        fs::write(&config, &source)?;
        let output = Command::new(env!("CARGO_BIN_EXE_lml"))
            .arg("--config")
            .arg(&config)
            .arg("import")
            .output()?;
        assert!(
            !output.status.success(),
            "accepted invalid config: {source}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read_to_string(&config)?, source);
    }
    Ok(())
}

#[test]
fn split_models_have_one_profile_and_projectors_are_not_models() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    fs::create_dir(directory.path().join("models"))?;
    for name in [
        "Qwen-00001-of-00002.gguf",
        "Qwen-00002-of-00002.gguf",
        "mmproj-Qwen.gguf",
        "notes.txt",
    ] {
        fs::write(directory.path().join("models").join(name), b"GGUF")?;
    }
    let config = directory.path().join("config.toml");
    fs::write(
        &config,
        "[runtimes.llama_cpp]\nexecutable = '/usr/bin/true'\nmodel_dirs = ['models']\n",
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .arg("import")
        .output()?;
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Added 1"));
    let saved = fs::read_to_string(&config)?;
    assert!(saved.contains("Qwen-00001-of-00002.gguf"));
    assert!(!saved.contains("Qwen-00002-of-00002.gguf"));
    assert!(!saved.contains("mmproj"));
    Ok(())
}

#[test]
fn import_accepts_an_explicitly_empty_profile_list() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let config = directory.path().join("config.toml");
    fs::write(
        &config,
        "profiles = [] # none yet\n[runtimes.omlx]\nexecutable = '/usr/bin/true'\n",
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .arg("import")
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let saved = fs::read_to_string(&config)?;
    assert!(saved.contains("oMLX — configured models"));
    assert!(saved.contains("# none yet"));
    Ok(())
}

#[test]
fn omlx_preserves_the_native_launch_context() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let caller = directory.path().join("caller");
    fs::create_dir(&caller)?;
    let server = directory.path().join("omlx");
    fs::write(
        &server,
        "#!/bin/sh\npwd\nprintf '<%s>\\n' \"$@\"\nprintf '%s\\n' \"$OMLX_BASE_PATH\"\n",
    )?;
    fs::set_permissions(&server, fs::Permissions::from_mode(0o755))?;
    let config = directory.path().join("config.toml");
    fs::write(
        &config,
        "[runtimes.omlx]\nexecutable = './omlx'\n[[profiles]]\nid = 'omlx'\nname = 'oMLX'\nruntime = 'omlx'\n",
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .current_dir(&caller)
        .env("OMLX_BASE_PATH", "native-settings")
        .arg("--config")
        .arg(&config)
        .args(["run", "omlx"])
        .output()?;
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "{}\n<serve>\nnative-settings\n",
            caller.canonicalize()?.display()
        )
    );
    Ok(())
}

#[test]
fn a_missing_model_is_preserved_but_cannot_be_launched() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let config = directory.path().join("config.toml");
    let source = "[runtimes.llama_cpp]\nexecutable = '/usr/bin/true'\n[[profiles]]\nid = 'missing'\nname = 'Missing'\nruntime = 'llama_cpp'\nmodel = 'gone.gguf'\n";
    fs::write(&config, source)?;
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .arg("import")
        .output()?;
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(&config)?, source);
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .args(["run", "missing"])
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("gone.gguf"));
    assert!(!output.stderr.contains(&0x1b));
    Ok(())
}

#[test]
fn incomplete_setup_does_not_create_a_catalog() -> io::Result<()> {
    let directory = tempfile::tempdir()?;
    let config = directory.path().join("config.toml");
    let output = Command::new(env!("CARGO_BIN_EXE_lml"))
        .arg("--config")
        .arg(&config)
        .arg("setup")
        .stdin(Stdio::null())
        .output()?;
    assert!(!output.status.success());
    assert!(!config.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not saved"));
    Ok(())
}
