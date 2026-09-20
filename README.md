# lml

A terminal picker and launcher for local model servers, targeting macOS.
DwarfStar (ds4) and llama.cpp have per-model launch profiles. oMLX has one
**oMLX — configured models** entry that starts its server as already configured.

## Use

Install and configure your runtimes separately, then:

```sh
lml setup             # Guided setup and initial model import
lml                   # Pick a saved launch profile
lml import            # Add models downloaded since the last import
lml run <profile-id>  # Launch directly, without the picker
```

Type to filter immediately (case-insensitive subsequence matching of names,
runtimes, and IDs). Use arrow keys to select and Enter to launch. Backspace edits
the filter. PgUp/PgDn scroll the selected profile's details when they overflow.
Esc or Ctrl-C exits directly, including when filtering; ordinary
letters never quit. The selected profile shows its ID, model path when
applicable, and launch command.

On launch, lml restores the ordinary terminal and replaces itself with the
foreground server. Logs and input go directly to the server. Ctrl-C reaches it
normally, and its exit status reaches the shell. There is no return to the
picker or background server management. A missing model, executable, or working
directory produces an error; lml does not install or download anything.

`setup` asks for the DwarfStar checkout (default model source: its `gguf/`
directory), the llama-server executable and model directories, and the oMLX
executable. Executables on `PATH` are suggested. Skip DwarfStar with a blank
answer; skip the other runtimes with `-`. Model directories are entered one at
a time. Setup refuses to overwrite an existing catalog; edit its runtime
entries to add runtimes later, then run `import`.

## Configuration

The catalog is `$XDG_CONFIG_HOME/lml/config.toml` when that variable is set,
otherwise `~/.config/lml/config.toml`. `--config /path/to/config.toml` overrides
it for any command. Edit this TOML file in your usual editor.

```toml
[runtimes.dwarfstar]
executable = "~/src/ds4/ds4-server"
working_dir = "~/src/ds4"
model_dirs = ["~/src/ds4/gguf"]

[runtimes.llama_cpp]
executable = "llama-server"
model_dirs = ["~/models", "/Volumes/Models/gguf"]

[runtimes.omlx]
executable = "omlx"

[[profiles]]
id = "llama-cpp-qwen"
name = "Qwen · 8K context"
runtime = "llama_cpp"
model = "~/models/Qwen.gguf"
args = ["--ctx-size", "8192"]

[[profiles]]
id = "omlx"
name = "oMLX — configured models"
runtime = "omlx"
```

Per-model commands are `EXECUTABLE -m MODEL` followed by the profile's `args`.
Arguments are literal list elements, with no shell evaluation. Copy a profile
and give it a distinct ID to create a variant; IDs must be unique. Each profile
owns its arguments. Import creates basic profiles with empty argument lists;
it does not guess context sizes, templates, MTP settings, or runtime support.
Point each runtime at directories containing models it supports.

Paths accept `~/`. Relative model/source/executable paths are relative to the
catalog's directory. A bare executable name is searched on `PATH` when no file
of that name exists beside the catalog. A runtime's optional `working_dir`
controls relative paths inside its extra arguments; otherwise per-model servers
run from the catalog directory. Setup supplies DwarfStar's checkout as its
working directory so it can find its runtime resources.

**oMLX configuration stays in oMLX.** Its entry runs only `omlx serve`, with
no setting overrides, model selection, or configuration copies. Its runtime
entry accepts only `executable`; its single profile has an ID, name, and runtime,
with no model or extra arguments. Requests to oMLX select models from its
configured collection. lml does not read or edit oMLX settings; oMLX retains its
normal native startup behavior.

## Import behavior

Import recursively scans configured directories for `.gguf` files. Multiple
sources may overlap or contain symlinks; runtime plus resolved model path is the
identity. Different runtimes can have separate profiles for the same file.
Standard split GGUF names (`-00001-of-00002.gguf`) produce one entry from the first
shard, and `mmproj` projector files are excluded. Other auxiliary files or model
formats may require manual profile editing. Discovery is not a compatibility
check or a validation of the model's contents.

Existing profiles and their comments, IDs, names, and arguments are preserved.
An import with no additions leaves the file untouched. Missing models stay in
the catalog and fail clearly when launched. Removing a model's last profile
makes it eligible for import again; moving a model to another resolved path
makes it a new candidate. Opening the picker never scans for new models.
An unreadable or missing configured source fails import without saving partial
additions. Configuration saves use atomic replacement; setup never replaces an
existing file.

## Build and check

Install [rustup](https://rustup.rs/) and
[just](https://just.systems/man/en/packages.html) (`brew install just` on macOS),
then run:

```sh
just setup          # Install the pinned Rust toolchain
just run setup      # Run lml's guided setup
just run            # Open the picker
just run --help
just check
```

`just build` creates `target/debug/lml`. Run `just` to list development commands.
Cargo handles Rust builds; the `justfile` explicitly selects the toolchain in
`rust-toolchain.toml`, even when Homebrew Rust comes first in `PATH`. CI uses
just 1.58.0 and checks Linux and macOS.

`--help`, `--version`, setup, import, and direct launch work without an
interactive terminal. The picker requires terminal stdin and stdout. Ratatui
restores the terminal after normal/error returns and through its panic hook.
See [CONTEXT.md](CONTEXT.md) for domain terminology and [docs/adr](docs/adr) for
the design decisions.

## Development

- `src/main.rs`: argument parsing, terminal preflight, and process exit status.
- `src/app.rs`: terminal event loop, picker state, filtering, and rendering.
- `src/catalog.rs`: TOML catalog, validation, discovery, and additive import.
- `src/setup.rs`: guided setup at the terminal I/O boundary.
- `src/launch.rs`: command preparation and foreground process handoff.
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
