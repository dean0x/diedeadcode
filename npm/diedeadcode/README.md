# diedeadcode

Conservative dead code detection for TypeScript and JavaScript.

`ddd` finds unused exports, unreachable functions, and dead code in your codebase with confidence scoring to minimize false positives.

## Installation

```bash
npm install -D diedeadcode
```

Or run directly with npx:

```bash
npx diedeadcode .
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
ddd analyze . --format json

# Check mode (exit code 1 if dead code found)
ddd analyze . --check
```

## Features

- **Fast** - Built on [oxc](https://oxc.rs) for blazing-fast parsing
- **Conservative** - Confidence scoring reduces false positives
- **Transitive analysis** - Detects code that's only called by other dead code
- **Framework-aware** - Understands Next.js, Express, Jest, Vitest patterns
- **Configurable** - Ignore patterns, entry points, and more

## Configuration

Run `ddd init` to generate a `ddd.toml` in your project root:

```toml
# Files to analyze
include = ["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"]
exclude = ["**/node_modules/**", "**/dist/**", "**/*.test.*"]

# Entry points
[entry]
files = ["src/index.ts", "src/main.ts"]
patterns = ["**/pages/**/*.tsx"]
autoDetect = true

# Output settings
[output]
format = "table"
minConfidence = "high"
showChains = true
```

## Alternative Installation Methods

### Cargo (Rust)

```bash
cargo install diedeadcode
```

### Homebrew (macOS)

```bash
brew install dean0x/tap/diedeadcode
```

## Documentation

See the [GitHub repository](https://github.com/dean0x/diedeadcode) for full documentation.

## License

MIT
