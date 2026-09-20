set positional-arguments

# Explicit rustup selection also works when Homebrew Rust comes first in PATH.
toolchain := `awk -F '"' '/^channel = / {print $2}' rust-toolchain.toml`

# Show available commands.
default:
    @just --list

# Install the compiler and tools pinned by this project.
setup:
    rustup toolchain install "{{toolchain}}" --profile minimal --component clippy --component rustfmt

# Run the same quality gate as CI, stopping on the first failure.
check: fmt-check lint test

# Apply standard Rust formatting.
fmt:
    rustup run "{{toolchain}}" cargo fmt --all

# Verify formatting without changing files.
fmt-check:
    rustup run "{{toolchain}}" cargo fmt --all --check

# Lint application and test code; warnings are errors.
lint:
    rustup run "{{toolchain}}" cargo clippy --all-targets --locked -- -D warnings

# Run the test suite.
test:
    rustup run "{{toolchain}}" cargo test --locked

# Build the application.
build:
    rustup run "{{toolchain}}" cargo build --locked

# Run the application, forwarding arguments (for example: just run --help).
run *args:
    rustup run "{{toolchain}}" cargo run --locked -- "$@"
