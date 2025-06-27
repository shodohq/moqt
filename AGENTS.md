# AGENT Instructions

This repository is a Rust workspace for the Media over QUIC Transport project.
The detailed protocol specification is available in `draft-ietf-moq-transport-12.txt`.

## Programmatic Checks
Run the following commands before committing changes:

```bash
cargo fmt --all -- --check
cargo test
```

If formatting fails, apply it with `cargo fmt --all`.

## Pull Request Guidelines
Include a short summary of the changes and the outcome of the tests.
