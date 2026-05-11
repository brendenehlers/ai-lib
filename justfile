_default:
    @just --list

# Run the full test suite (workspace)
test:
    cargo test --workspace

# Run tests for one crate, e.g. `just test-crate ai-lib-anthropic`
test-crate crate:
    cargo test -p {{crate}}

# Run a single integration test file, e.g. `just test-file ai-lib-core client_builder`
test-file crate file:
    cargo test -p {{crate}} --test {{file}}

# Run doctests only across the workspace
test-doc:
    cargo test --workspace --doc

# Regenerate trybuild compile-fail .stderr snapshots after intentional API changes
update-snapshots:
    TRYBUILD=overwrite cargo test -p ai-lib-core --test compile_fail

# Fast type-check across the workspace (no codegen)
check:
    cargo check --workspace --all-targets

# Debug build of the workspace
build:
    cargo build --workspace

# Optimized release build of the workspace
build-release:
    cargo build --workspace --release

# Run clippy with warnings as errors
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format all code in place
fmt:
    cargo fmt --all

# Verify formatting without changing files (CI-friendly)
fmt-check:
    cargo fmt --all -- --check

# Run the demo binary (requires GEMINI_API_KEY and ANTHROPIC_API_KEY in env)
run:
    cargo run -p ai-lib-bin

# Build rustdoc for the workspace and open in browser
doc:
    cargo doc --workspace --no-deps --open

# Everything CI should run before merge: format check, lint, test
ci: fmt-check lint test

# Update dependencies within the bounds of Cargo.toml
update:
    cargo update

# Remove all build artifacts
clean:
    cargo clean
