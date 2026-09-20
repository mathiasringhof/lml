# lml

A macOS-first terminal picker for local model servers. Supports
[DwarfStar (ds4)](https://github.com/antirez/ds4),
[llama.cpp](https://github.com/ggml-org/llama.cpp), and
[oMLX](https://github.com/jundot/omlx).

> [!NOTE]
> **Built with agentic engineering.** AI coding agents contribute to design,
> implementation, tests, and review, under human direction.

## Install

Install [rustup](https://rustup.rs/) and
[just](https://just.systems/man/en/packages.html), then:

```sh
git clone https://github.com/mathiasringhof/lml.git
cd lml
just setup
just install
```

The release binary is installed into `~/.local/bin`, which must be on your
`PATH`. Use `just install /another/root` to install into `/another/root/bin`.
Install your model runtimes and download models separately.

## Use

```sh
lml setup             # Configure runtimes and import local models
lml                   # Open the picker
lml import            # Add newly downloaded models
lml run <profile-id>  # Launch a saved profile directly
```

Type to filter by name, runtime, or ID. Arrow keys select, Enter launches,
and Esc or Ctrl-C exits. Backspace edits the filter; PgUp/PgDn scroll details.
Launching restores the ordinary terminal and hands it to the server. Logs
appear normally; stopping the server returns you to the shell.

Setup guides you through executable locations and model directories. Skip
DwarfStar with a blank answer; skip other runtimes with `-`. Setup refuses to
overwrite an existing catalog; edit it to add runtimes, then run `lml import`.

**oMLX is one server entry for its configured models.** It runs `omlx serve`;
configure models and settings in oMLX itself. lml supplies no oMLX overrides
and does not copy or edit its settings.

## Configuration

Edit `~/.config/lml/config.toml`, or `$XDG_CONFIG_HOME/lml/config.toml` when set.
Use `--config /path/to/config.toml` with any command to choose another catalog.
Setup generates the runtime entries and launch profiles; for example:

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

Copy a per-model profile with a new, unique ID to create an argument variant.
DwarfStar and llama.cpp receive `-m MODEL` followed by the profile's literal
`args`; no shell evaluation occurs. Imported profiles start with empty arguments.
oMLX accepts only its executable and a single profile without a model or arguments.

Relative paths resolve from the catalog directory; `~/` expands to your home.
Bare executable names fall back to `PATH`. For per-model servers, `working_dir`
controls the working directory, including relative paths in arguments; it defaults to the
catalog directory. DwarfStar needs its checkout directory, supplied by setup.

Import adds profiles without changing existing IDs, arguments, or comments.
It deduplicates by runtime and resolved model path, scanning `.gguf` files
recursively. Standard split files produce one profile from the first shard;
`mmproj` projectors are excluded. Use sources compatible with each runtime:
import does not validate model contents, compatibility, or tuning.

Missing models stay in the catalog and fail on launch. Deleting a model's last
profile makes it eligible for import again; moving a model makes it a new
candidate. Unreadable sources abort import without saving partial additions.
The picker never imports automatically.

## Development

`just check` runs formatting, Clippy, and all tests, as CI does on macOS and Linux.
Use `just test import` to filter tests, `just run` to run locally, and
`just release` to build `target/release/lml`. Run `just` for all commands.

See the [development notes](docs/rust-setup.md), [glossary](CONTEXT.md),
[design decisions](docs/adr), and [agent instructions](AGENTS.md).
Report bugs and feature requests in [GitHub Issues](https://github.com/mathiasringhof/lml/issues).

[MIT license](LICENSE).
