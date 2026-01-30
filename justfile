# This is a justfile. See https://github.com/casey/just

check-all:
    cargo clippy --all-targets --workspace
    cargo clippy --target wasm32-unknown-unknown
    cargo doc --workspace --no-deps
    cargo fmt --check