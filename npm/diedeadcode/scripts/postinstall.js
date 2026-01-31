#!/usr/bin/env node

/**
 * Postinstall script for diedeadcode
 *
 * This script runs after npm install and attempts to download the native binary
 * if the platform-specific optional dependency failed to install.
 *
 * This can happen when:
 * - Using --no-optional flag
 * - Platform package not available for this platform
 * - Network issues during optional dependency installation
 */

const { existsSync, mkdirSync, createWriteStream, chmodSync } = require("fs");
const { join } = require("path");
const os = require("os");
const https = require("https");

const PACKAGE_VERSION = require("../package.json").version;
const NPM_REGISTRY = "https://registry.npmjs.org";

/**
 * Check if running on musl libc
 */
function isMusl() {
  if (existsSync("/etc/alpine-release")) {
    return true;
  }
  try {
    const { spawnSync } = require("child_process");
    const { stdout } = spawnSync("ldd", ["--version"], {
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });
    return stdout && stdout.includes("musl");
  } catch {
    return false;
  }
}

/**
 * Get the platform-specific package info
 */
function getPlatformInfo() {
  const platform = os.platform();
  const arch = os.arch();

  const platformMap = {
    darwin: {
      arm64: {
        package: "@diedeadcode/cli-darwin-arm64",
        binary: "ddd",
      },
      x64: {
        package: "@diedeadcode/cli-darwin-x64",
        binary: "ddd",
      },
    },
    linux: {
      arm64: isMusl()
        ? { package: "@diedeadcode/cli-linux-arm64-musl", binary: "ddd" }
        : { package: "@diedeadcode/cli-linux-arm64", binary: "ddd" },
      x64: isMusl()
        ? { package: "@diedeadcode/cli-linux-x64-musl", binary: "ddd" }
        : { package: "@diedeadcode/cli-linux-x64", binary: "ddd" },
    },
    win32: {
      x64: {
        package: "@diedeadcode/cli-win32-x64",
        binary: "ddd.exe",
      },
    },
  };

  return platformMap[platform]?.[arch];
}

/**
 * Check if the platform package is already installed
 */
function isPlatformPackageInstalled(packageName) {
  try {
    require.resolve(`${packageName}/package.json`);
    return true;
  } catch {
    return false;
  }
}

/**
 * Download a file from URL to destination
 */
function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = createWriteStream(dest);
    https
      .get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Follow redirect
          downloadFile(response.headers.location, dest)
            .then(resolve)
            .catch(reject);
          return;
        }
        if (response.statusCode !== 200) {
          reject(new Error(`HTTP ${response.statusCode}`));
          return;
        }
        response.pipe(file);
        file.on("finish", () => {
          file.close(resolve);
        });
      })
      .on("error", (err) => {
        reject(err);
      });
  });
}

/**
 * Fetch JSON from URL
 */
function fetchJson(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (response.statusCode !== 200) {
          reject(new Error(`HTTP ${response.statusCode}`));
          return;
        }
        let data = "";
        response.on("data", (chunk) => (data += chunk));
        response.on("end", () => {
          try {
            resolve(JSON.parse(data));
          } catch (e) {
            reject(e);
          }
        });
      })
      .on("error", reject);
  });
}

/**
 * Download the binary from npm registry
 */
async function downloadBinaryFromNpm(packageName, binaryName, destPath) {
  // Get package metadata
  const metadataUrl = `${NPM_REGISTRY}/${packageName}/${PACKAGE_VERSION}`;
  const metadata = await fetchJson(metadataUrl);

  if (!metadata.dist || !metadata.dist.tarball) {
    throw new Error("Could not find tarball URL in package metadata");
  }

  // Download tarball
  const tarballUrl = metadata.dist.tarball;
  const tmpDir = os.tmpdir();
  const tarballPath = join(tmpDir, `${packageName.replace("/", "-")}.tgz`);

  await downloadFile(tarballUrl, tarballPath);

  // Extract binary from tarball
  const { execSync } = require("child_process");
  const extractDir = join(tmpDir, `${packageName.replace("/", "-")}-extract`);

  mkdirSync(extractDir, { recursive: true });

  // Extract tarball
  execSync(`tar -xzf "${tarballPath}" -C "${extractDir}"`, {
    stdio: "ignore",
  });

  // Copy binary
  const { copyFileSync } = require("fs");
  const extractedBinary = join(extractDir, "package", binaryName);

  if (!existsSync(extractedBinary)) {
    throw new Error(`Binary not found in package: ${binaryName}`);
  }

  copyFileSync(extractedBinary, destPath);

  // Make executable on Unix
  if (os.platform() !== "win32") {
    chmodSync(destPath, 0o755);
  }

  // Cleanup
  try {
    const { rmSync } = require("fs");
    rmSync(tarballPath, { force: true });
    rmSync(extractDir, { recursive: true, force: true });
  } catch {
    // Ignore cleanup errors
  }
}

async function main() {
  const platformInfo = getPlatformInfo();

  if (!platformInfo) {
    // Unsupported platform, nothing we can do
    console.log(
      `diedeadcode: No prebuilt binary available for ${os.platform()}-${os.arch()}`
    );
    console.log("You can install from source with: cargo install diedeadcode");
    return;
  }

  // Check if platform package is already installed
  if (isPlatformPackageInstalled(platformInfo.package)) {
    // All good, platform package is installed
    return;
  }

  console.log(
    `diedeadcode: Platform package ${platformInfo.package} not found, downloading binary...`
  );

  const binDir = join(__dirname, "..", "bin");
  const destPath = join(binDir, platformInfo.binary);

  try {
    mkdirSync(binDir, { recursive: true });
    await downloadBinaryFromNpm(
      platformInfo.package,
      platformInfo.binary,
      destPath
    );
    console.log("diedeadcode: Binary downloaded successfully");
  } catch (error) {
    console.error(`diedeadcode: Failed to download binary: ${error.message}`);
    console.log("You can install from source with: cargo install diedeadcode");
  }
}

main().catch((error) => {
  console.error(`diedeadcode postinstall error: ${error.message}`);
  // Don't fail the install
  process.exit(0);
});
