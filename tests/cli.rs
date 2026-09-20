use std::io;
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
