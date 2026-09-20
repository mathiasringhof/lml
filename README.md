# lml

A minimal Rust CLI/TUI starting point. The application currently opens a welcome
screen; press `q`, `Esc`, or `Ctrl+C` to quit.

Install [rustup](https://rustup.rs/) and
[just](https://just.systems/man/en/packages.html) (`brew install just` on macOS),
then run:

```sh
just setup
just run
just run --help
just check
```

Run `just` to list available commands, including `build`, `fmt`, `lint`, and
`test`. Cargo handles Rust builds; the `justfile` coordinates development tasks
and explicitly selects the toolchain from `rust-toolchain.toml`, even when
Homebrew's Rust comes first in `PATH`. CI uses just 1.58.0.

`--help` and `--version` work without a terminal. Launching the TUI requires both
stdin and stdout to be terminals. Ratatui manages terminal setup, restoration
after normal/error returns, and its panic restoration hook.

## Development

- `src/main.rs`: argument parsing, terminal preflight, and process exit status.
- `src/app.rs`: terminal event loop, key handling, and rendering.
- `tests/cli.rs`: CLI behavior without a real terminal.
- `justfile`: development commands and the quality gate used by GitHub CI.

Use a single binary crate until there is a concrete need for a library or
workspace. Add application behavior as small, testable functions; introduce
async tasks, persistence, and extra dependencies when features require them.

The compiler is pinned to Rust 1.98.1 with edition 2024. `rust-version` declares
the same supported minimum; no older compiler compatibility is promised.
Keep `Cargo.lock` in version control and use `--locked` in checks. Update the
toolchain and dependencies deliberately, running the full check after changes.

See [AGENTS.md](AGENTS.md) for agent instructions and
[docs/rust-setup.md](docs/rust-setup.md) for lint choices and research.
GitHub CI runs on Linux and macOS. Release publishing and Matt Pocock's skills
can be configured when their details are provided.

Licensed under the [MIT license](LICENSE).
