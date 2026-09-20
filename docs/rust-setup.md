# Development notes

The project uses one synchronous Rust binary crate. Terminal I/O stays at the
boundary; picker state and rendering are tested without a live terminal.

## Toolchain and checks

`rust-toolchain.toml` pins Rust and its Clippy/rustfmt components. The justfile
selects that toolchain explicitly, including when Homebrew Cargo comes first
on `PATH`. Keep `Cargo.lock` tracked and use `--locked`; update dependencies
and the toolchain deliberately.

`just check` runs formatting, Clippy, and tests. CI runs the same command on
macOS and Linux. When terminal handling changes, also verify launch, exit,
interruption, and restoration in a real terminal or pseudo-terminal.

## Lint policy

Compiler warnings and Clippy's `all` and `pedantic` groups are errors; unsafe
code is forbidden. Propagate runtime errors with `Result`. Assertions and
`expect` are appropriate in tests.

The exceptions in `Cargo.toml` avoid unnecessary library-style documentation
on application internals and arbitrary function-length limits. Prefer small,
sound fixes to new exceptions. When an exception is necessary, use a scoped
`#[expect(..., reason = "...")]` so obsolete suppressions are detected.

Do not enable whole Clippy restriction or nursery groups without evaluating
the individual lints. See [Clippy's guidance](https://doc.rust-lang.org/clippy/lints.html)
and [AGENTS.md](../AGENTS.md) for the repository's contribution rules.
