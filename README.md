# diedeadcode

[![Crates.io](https://img.shields.io/crates/v/diedeadcode.svg)](https://crates.io/crates/diedeadcode)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/dean0x/diedeadcode/actions/workflows/ci.yml/badge.svg)](https://github.com/dean0x/diedeadcode/actions)

**Conservative dead code detection for TypeScript and JavaScript.**

`ddd` finds unused exports, unreachable functions, and dead code in your codebase with confidence scoring to minimize false positives.

## Features

- **Fast** - Built on [oxc](https://oxc.rs) for blazing-fast parsing
- **Conservative** - Confidence scoring reduces false positives
- **Transitive analysis** - Detects code that's only called by other dead code
- **Framework-aware** - Understands Next.js, React, Express patterns
- **Configurable** - Ignore patterns, entry points, and more

## Installation

```bash
cargo install diedeadcode
```

Or build from source:

```bash
git clone https://github.com/dean0x/diedeadcode.git
cd diedeadcode
cargo build --release
```

## Quick Start

```bash
# Analyze current directory
ddd .

# Analyze with verbose output
ddd . --verbose

# Initialize config file
ddd init

# Output as JSON
ddd . --format json
```

## Configuration

Create a `ddd.toml` in your project root:

```toml
# Entry points that should never be marked as dead
entry_points = [
    "src/index.ts",
    "src/main.ts",
]

# Patterns to ignore
ignore = [
    "**/*.test.ts",
    "**/*.spec.ts",
    "**/node_modules/**",
]

# Minimum confidence threshold (0-100)
min_confidence = 80
```

## How It Works

1. **Parse** - Uses oxc to build an AST for each file
2. **Extract** - Identifies all exports, functions, classes, and variables
3. **Graph** - Builds a call graph tracking all references
4. **Propagate** - BFS from entry points to find unreachable code
5. **Score** - Assigns confidence based on patterns and heuristics

### Confidence Scoring

Not all dead code warnings are equal. `ddd` scores each finding:

| Score | Meaning |
|-------|---------|
| 90-100 | High confidence - likely dead |
| 70-89 | Medium confidence - review recommended |
| <70 | Lower confidence - may be false positive |

Factors that reduce confidence:
- Dynamic imports (`import()`)
- Reflection (`eval`, `Reflect.*`)
- Framework conventions (lifecycle methods, hooks)
- String property access patterns

## Output Formats

```bash
# Table (default)
ddd .

# JSON for tooling integration
ddd . --format json

# Compact single-line per issue
ddd . --format compact
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No dead code found |
| 1 | Dead code found |
| 2 | Error during analysis |

## Performance

`ddd` is designed for large codebases:

- Parallel file parsing with rayon
- Incremental analysis (coming soon)
- Memory-efficient AST traversal

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT - see [LICENSE](LICENSE)
