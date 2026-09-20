# Working in this repository

- Read the relevant code before changing it. Keep the single-crate structure
  until a feature needs more; do not add speculative abstractions or async.
- For behavior changes, add a focused behavior test that fails for the intended
  reason, implement the change, and refactor with tests passing. Do not invent
  tests for prose or formatting, or tests that merely repeat implementation.
- Before handing off code changes, run `just check`. It selects the
  pinned Rust toolchain and runs the same checks as CI. Report checks you could
  not run accurately; do not claim success from a partial run.
- Keep warnings and Clippy clean. Do not weaken lint levels, delete tests, or add
  dummy uses to silence failures. Prefer the smallest sound code fix. When a
  lint conflicts with a sound design, use a narrowly scoped
  `#[expect(clippy::lint_name, reason = "specific rationale")]` and explain it.
  Prefer expectations over allows so stale exceptions are detected.
- Propagate runtime failures with `Result`; do not replace `unwrap`/`expect` with
  panic, indexing, silent defaults, or ignored errors. Direct assertions and
  `expect` calls are fine in tests. `unsafe` is forbidden in this crate.
- Keep terminal I/O at the boundary. Test key handling and state changes without
  a terminal, and rendering with Ratatui's `TestBackend`. Avoid stdout/stderr
  writes while the TUI is active; report errors after restoration. Preserve
  quit-key handling and terminal restoration on errors and panics.
- CLI tests should check status and meaningful output, not exact help snapshots.
- Keep `Cargo.lock` tracked. Use `--locked`; update dependencies intentionally.
- Keep the README current when behavior or development commands change.
