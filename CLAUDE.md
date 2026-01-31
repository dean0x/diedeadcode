# diedeadcode - Project Instructions

## Overview

Conservative TypeScript/JavaScript dead code detection tool built in Rust.

## Architecture

- **Rust CLI** (`src/`) - Core analysis engine using oxc parser
- **npm distribution** (`npm/`) - Platform-specific binary packages
- **Binary name**: `ddd`

## Release Process

### Automated (after first release)

1. Tag and push: `git tag vX.Y.Z && git push --tags`
2. CI builds all 7 platform binaries
3. CI publishes to npm, crates.io, creates GitHub Release
4. CI updates Homebrew tap

### Manual (first release or if CI fails)

```bash
# 1. Tag release
git tag vX.Y.Z
git push --tags

# 2. Wait for CI builds to complete (npm publish will fail, that's ok)
gh run watch

# 3. Download built binaries
./scripts/download-artifacts.sh

# 4. Flatten artifacts (if needed)
cd artifacts
for dir in */; do
  name="${dir%/}"
  mv "$dir$name" "./${name}.tmp"
  rm -rf "$dir"
  mv "./${name}.tmp" "./$name"
done
cd ..

# 5. Generate npm packages with version
node scripts/generate-npm-packages.mjs X.Y.Z artifacts

# 6. Login to npm (if not already)
npm login

# 7. Set npm token and publish (use automation token with 2FA bypass)
echo "//registry.npmjs.org/:_authToken=YOUR_TOKEN" > ~/.npmrc
for pkg in cli-darwin-arm64 cli-darwin-x64 cli-linux-x64 cli-linux-arm64 cli-linux-x64-musl cli-linux-arm64-musl cli-win32-x64; do
  cd npm/$pkg && npm publish --access public && cd ../..
done
cd npm/diedeadcode && npm publish --access public && cd ../..

# 8. Publish to crates.io
cargo publish
```

### Supported Platforms

| Platform | npm Package |
|----------|-------------|
| macOS ARM64 | `@dean0x/cli-darwin-arm64` |
| macOS x64 | `@dean0x/cli-darwin-x64` |
| Linux x64 (glibc) | `@dean0x/cli-linux-x64` |
| Linux ARM64 (glibc) | `@dean0x/cli-linux-arm64` |
| Linux x64 (musl) | `@dean0x/cli-linux-x64-musl` |
| Linux ARM64 (musl) | `@dean0x/cli-linux-arm64-musl` |
| Windows x64 | `@dean0x/cli-win32-x64` |

### Required Secrets (for automated CI)

- `NPM_TOKEN` - npm granular access token with publish permission
- `CRATES_IO_TOKEN` - crates.io API token
- `HOMEBREW_TAP_TOKEN` - GitHub PAT for dean0x/homebrew-tap repo

### After First Release

Configure Trusted Publishing on each npm package:
1. npmjs.com → Package Settings → Publishing access
2. Add trusted publisher: `dean0x/diedeadcode`, workflow `release.yml`

## Development

```bash
# Build
cargo build --release

# Test
cargo test

# Run locally
./target/release/ddd .

# Analyze test fixtures
./target/release/ddd tests/integration/fixtures/basic
```

## Distribution Channels

| Method | Command |
|--------|---------|
| curl | `curl -fsSL https://raw.githubusercontent.com/dean0x/diedeadcode/main/install.sh \| sh` |
| npm | `npm install -D @dean0x/diedeadcode` |
| npx | `npx @dean0x/diedeadcode .` |
| cargo | `cargo install diedeadcode` |
| Homebrew | `brew install dean0x/tap/diedeadcode` |
| Binary | [GitHub Releases](https://github.com/dean0x/diedeadcode/releases/latest) |

## Homebrew Tap

To set up the Homebrew tap:
1. Create repo `dean0x/homebrew-tap` on GitHub
2. Copy `homebrew/diedeadcode.rb` to `Formula/diedeadcode.rb` in that repo
3. Users can then `brew install dean0x/tap/diedeadcode`

## Key Files

- `src/main.rs` - CLI entry point
- `src/lib.rs` - Library exports
- `src/analysis/` - Dead code detection logic
- `src/plugins/frameworks/` - Framework-specific patterns (Next.js, Jest, etc.)
- `npm/diedeadcode/bin/ddd` - Node.js shim for npm distribution
- `.github/workflows/release.yml` - Release CI/CD
