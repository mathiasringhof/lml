# Rust setup decisions

This baseline adapts the local `fluxrepo-update` project's checks for a small
terminal application, using current upstream documentation.

## Strict checks with useful incentives

Adopt Flux's Clippy `all` and `pedantic` groups, deny compiler warnings, and forbid
unsafe code in this crate. Pin the compiler so lint upgrades are deliberate.
The check command also passes `-D warnings` and checks every target, including
tests. Add explicit feature combinations when the project defines features.

Additional targeted rules reject `dbg!`, `todo!`, `unimplemented!`, `unwrap`, and
`expect`. Test functions may use `unwrap`/`expect`, keeping test failures direct.
Lint suppressions require a reason; scoped `#[expect]` attributes detect stale
exceptions through Rust's `unfulfilled_lint_expectations` warning.

Allow `missing_errors_doc`, `missing_panics_doc`, and `must_use_candidate` for this
binary application's internal APIs. Allow `too_many_lines`: splitting a cohesive
function merely to meet a line limit is often counterproductive. These are
documented policy choices, not permissions to suppress unrelated findings.
Revisit documentation rules if a public library API is introduced.

Unlike Flux, retain `fn_params_excessive_bools` and `needless_pass_by_value`; use a
local, justified exception when an actual API requires it.

Do not enable the entire `restriction`, `nursery`, or `cargo` groups. Clippy
[warns against all restriction lints](https://doc.rust-lang.org/clippy/lints.html)
and describes nursery lints as still under development. Evaluate individual
rules against real bugs and maintenance cost. Avoid blanket bans on indexing,
arithmetic, casts, or panics that incentivize verbose code or concealed failures.
Lint cleanliness alone does not prove correctness or a panic-free application.

## Toolchain and application structure

- Rust 1.98.1, edition 2024, exact toolchain pin and committed lockfile.
- Clap for CLI parsing; Ratatui 0.30.2 with Crossterm 0.29 for terminal UI.
- A synchronous event loop that blocks for input; no polling timer or async
  runtime until background work requires one.
- Ratatui's `run` owns the normal terminal lifecycle and installs a panic hook.
  Its initialization may panic on terminal setup failure; the TTY preflight
  handles ordinary redirected-input/output mistakes before initialization.
- Behavior tests for help, version, invalid arguments, terminal preflight,
  quit keys, rendering, and a tiny terminal. PTY smoke checks validate the real
  terminal lifecycle when changing terminal setup.
- `just` coordinates development commands while Cargo handles builds. Recipes
  select the pinned toolchain explicitly, including when Homebrew Rust shadows
  rustup. `just check` runs formatting, Clippy, and tests in order.
- GitHub Actions runs `just check` on Linux and macOS. Add Windows when
  it becomes a supported target. Add security/dependency automation and release
  jobs when repository and distribution requirements are known.

The GitHub repository provides the MIT license. `publish = false` prevents
accidental crates.io publication during setup.

## References

- [Latest Rust release](https://blog.rust-lang.org/releases/latest/)
- [Clippy usage and lint groups](https://doc.rust-lang.org/clippy/usage.html)
- [Clippy configuration](https://doc.rust-lang.org/clippy/lint_configuration.html)
- [Ratatui installation and compatible Crossterm version](https://ratatui.rs/installation/)
- [Ratatui terminal lifecycle](https://docs.rs/ratatui/latest/ratatui/fn.run.html)
