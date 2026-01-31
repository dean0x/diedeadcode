#!/usr/bin/env node

/**
 * Generate npm packages for release
 *
 * This script:
 * 1. Takes a version as argument
 * 2. Copies binaries from the artifacts directory to npm packages
 * 3. Updates all package.json versions atomically
 *
 * Usage: node scripts/generate-npm-packages.mjs <version> <artifacts-dir>
 *
 * Expected artifacts structure:
 *   artifacts/
 *     ddd-darwin-arm64
 *     ddd-darwin-x64
 *     ddd-linux-x64
 *     ddd-linux-arm64
 *     ddd-linux-x64-musl
 *     ddd-linux-arm64-musl
 *     ddd-win32-x64.exe
 */

import { readFileSync, writeFileSync, copyFileSync, chmodSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const NPM_DIR = join(ROOT, "npm");

const PLATFORMS = [
  {
    name: "cli-darwin-arm64",
    artifact: "ddd-darwin-arm64",
    binary: "ddd",
  },
  {
    name: "cli-darwin-x64",
    artifact: "ddd-darwin-x64",
    binary: "ddd",
  },
  {
    name: "cli-linux-x64",
    artifact: "ddd-linux-x64",
    binary: "ddd",
  },
  {
    name: "cli-linux-arm64",
    artifact: "ddd-linux-arm64",
    binary: "ddd",
  },
  {
    name: "cli-linux-x64-musl",
    artifact: "ddd-linux-x64-musl",
    binary: "ddd",
  },
  {
    name: "cli-linux-arm64-musl",
    artifact: "ddd-linux-arm64-musl",
    binary: "ddd",
  },
  {
    name: "cli-win32-x64",
    artifact: "ddd-win32-x64.exe",
    binary: "ddd.exe",
  },
];

function updatePackageVersion(packagePath, version) {
  const pkg = JSON.parse(readFileSync(packagePath, "utf8"));
  pkg.version = version;

  // Update optionalDependencies versions for main package
  if (pkg.optionalDependencies) {
    for (const dep of Object.keys(pkg.optionalDependencies)) {
      pkg.optionalDependencies[dep] = version;
    }
  }

  writeFileSync(packagePath, JSON.stringify(pkg, null, 2) + "\n");
  console.log(`Updated ${packagePath} to version ${version}`);
}

function copyBinary(artifactsDir, platform) {
  const src = join(artifactsDir, platform.artifact);
  const dest = join(NPM_DIR, platform.name, platform.binary);

  if (!existsSync(src)) {
    throw new Error(`Artifact not found: ${src}`);
  }

  copyFileSync(src, dest);

  // Make executable on Unix
  if (!platform.binary.endsWith(".exe")) {
    chmodSync(dest, 0o755);
  }

  console.log(`Copied ${platform.artifact} -> ${platform.name}/${platform.binary}`);
}

function main() {
  const args = process.argv.slice(2);

  if (args.length < 2) {
    console.error("Usage: generate-npm-packages.mjs <version> <artifacts-dir>");
    console.error("Example: generate-npm-packages.mjs 0.1.0 ./artifacts");
    process.exit(1);
  }

  const [version, artifactsDir] = args;

  // Validate version format
  if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(version)) {
    console.error(`Invalid version format: ${version}`);
    console.error("Expected: X.Y.Z or X.Y.Z-prerelease");
    process.exit(1);
  }

  if (!existsSync(artifactsDir)) {
    console.error(`Artifacts directory not found: ${artifactsDir}`);
    process.exit(1);
  }

  console.log(`\nGenerating npm packages for version ${version}\n`);

  // Update main package version
  updatePackageVersion(join(NPM_DIR, "diedeadcode", "package.json"), version);

  // Copy binaries and update platform package versions
  for (const platform of PLATFORMS) {
    copyBinary(artifactsDir, platform);
    updatePackageVersion(join(NPM_DIR, platform.name, "package.json"), version);
  }

  console.log("\nDone! Packages are ready for publishing.");
  console.log("\nPublish order (platform packages first, then main):");
  console.log("  1. npm publish npm/cli-darwin-arm64");
  console.log("  2. npm publish npm/cli-darwin-x64");
  console.log("  3. npm publish npm/cli-linux-x64");
  console.log("  4. npm publish npm/cli-linux-arm64");
  console.log("  5. npm publish npm/cli-linux-x64-musl");
  console.log("  6. npm publish npm/cli-linux-arm64-musl");
  console.log("  7. npm publish npm/cli-win32-x64");
  console.log("  8. npm publish npm/diedeadcode");
}

main();
