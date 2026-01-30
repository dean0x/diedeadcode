# Contributing to diedeadcode

Thanks for your interest in contributing!

## Development Setup

```bash
git clone https://github.com/dean0x/diedeadcode.git
cd diedeadcode
cargo build
cargo test
```

## Running Tests

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Keep functions small and focused
- Add tests for new functionality

## Pull Request Process

1. Fork the repo and create your branch from `main`
2. Add tests for any new functionality
3. Ensure `cargo test` passes
4. Ensure `cargo clippy` has no warnings
5. Update documentation if needed
6. Submit PR with clear description

## Commit Messages

Use clear, descriptive commit messages:

```
Add confidence scoring for dynamic imports

- Reduce confidence when import() is detected
- Add tests for dynamic import patterns
```

## Reporting Issues

When reporting bugs, include:

- `ddd --version` output
- Minimal reproduction case
- Expected vs actual behavior
- OS and Rust version

## Architecture Overview

```
src/
├── analysis/       # Core analysis logic
│   ├── call_graph/ # Symbol extraction and graph building
│   ├── confidence/ # Confidence scoring
│   ├── deadness/   # Dead code propagation
│   └── entry_points/ # Entry point detection
├── cli/            # Command-line interface
├── config/         # Configuration loading
├── core/           # Core types and errors
└── output/         # Output formatting
```

## Questions?

Open an issue for questions or discussions.
