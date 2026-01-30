//! Configuration loading from ddd.toml or package.json.

use super::schema::{Config, EntryConfig};
use crate::core::{DddError, Result};
use std::path::{Path, PathBuf};

/// Configuration file names in priority order.
const CONFIG_FILES: &[&str] = &["ddd.toml", ".dddrc.toml", "ddd.json", ".dddrc.json"];

/// Load configuration from the given directory or its parents.
pub fn load_config(start_dir: &Path) -> Result<(Config, Option<PathBuf>)> {
    // Search for config file
    if let Some(config_path) = find_config_file(start_dir) {
        let config = load_config_file(&config_path)?;
        return Ok((config, Some(config_path)));
    }

    // Try to load from package.json
    if let Some(pkg_path) = find_package_json(start_dir) {
        if let Some(config) = load_from_package_json(&pkg_path)? {
            return Ok((config, Some(pkg_path)));
        }
    }

    // Return default config
    Ok((Config::default(), None))
}

/// Find config file by searching up the directory tree.
fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        for filename in CONFIG_FILES {
            let candidate = current.join(filename);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        if !current.pop() {
            break;
        }
    }

    None
}

/// Find package.json by searching up the directory tree.
fn find_package_json(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        let candidate = current.join("package.json");
        if candidate.exists() {
            return Some(candidate);
        }

        if !current.pop() {
            break;
        }
    }

    None
}

/// Load configuration from a config file.
fn load_config_file(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path).map_err(|e| DddError::io_error(path.to_path_buf(), e))?;

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "toml" => toml::from_str(&content).map_err(|e| {
            DddError::config_error(format!("Failed to parse {}: {}", path.display(), e))
        }),
        "json" => serde_json::from_str(&content).map_err(|e| {
            DddError::config_error(format!("Failed to parse {}: {}", path.display(), e))
        }),
        _ => Err(DddError::config_error(format!(
            "Unsupported config file format: {}",
            path.display()
        ))),
    }
}

/// Try to load ddd configuration from package.json "ddd" field.
fn load_from_package_json(path: &Path) -> Result<Option<Config>> {
    let content = std::fs::read_to_string(path).map_err(|e| DddError::io_error(path.to_path_buf(), e))?;

    let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        DddError::config_error(format!("Failed to parse {}: {}", path.display(), e))
    })?;

    // Check for "ddd" field in package.json
    if let Some(ddd_config) = pkg.get("ddd") {
        let config: Config = serde_json::from_value(ddd_config.clone()).map_err(|e| {
            DddError::config_error(format!("Invalid ddd config in package.json: {}", e))
        })?;
        return Ok(Some(config));
    }

    Ok(None)
}

/// Extract entry points from package.json.
pub fn extract_entry_points_from_package_json(path: &Path) -> Result<EntryConfig> {
    let content = std::fs::read_to_string(path).map_err(|e| DddError::io_error(path.to_path_buf(), e))?;

    let pkg: PackageJson = serde_json::from_str(&content).map_err(|e| {
        DddError::config_error(format!("Failed to parse {}: {}", path.display(), e))
    })?;

    let mut entry_files = Vec::new();
    let pkg_dir = path.parent().unwrap_or(Path::new("."));

    // Main entry point
    if let Some(main) = &pkg.main {
        let main_path = pkg_dir.join(main);
        if main_path.exists() || likely_typescript_file(&main_path) {
            entry_files.push(normalize_entry_path(pkg_dir, main));
        }
    }

    // Module entry point
    if let Some(module) = &pkg.module {
        let module_path = pkg_dir.join(module);
        if module_path.exists() || likely_typescript_file(&module_path) {
            entry_files.push(normalize_entry_path(pkg_dir, module));
        }
    }

    // Types entry point
    if let Some(types) = &pkg.types {
        entry_files.push(normalize_entry_path(pkg_dir, types));
    }

    // Bin entries
    if let Some(bin) = &pkg.bin {
        match bin {
            BinField::Single(path) => {
                entry_files.push(normalize_entry_path(pkg_dir, path));
            }
            BinField::Map(map) => {
                for path in map.values() {
                    entry_files.push(normalize_entry_path(pkg_dir, path));
                }
            }
        }
    }

    // Exports field (complex)
    if let Some(exports) = &pkg.exports {
        extract_exports_entry_points(exports, pkg_dir, &mut entry_files);
    }

    Ok(EntryConfig {
        files: entry_files,
        patterns: Vec::new(),
        auto_detect: true,
        exports: Vec::new(),
    })
}

/// Normalize an entry path, trying TypeScript extensions if .js is specified.
fn normalize_entry_path(pkg_dir: &Path, entry: &str) -> PathBuf {
    let path = pkg_dir.join(entry);

    // If it's a .js file, check for .ts equivalent
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if ext == "js" || ext == "mjs" || ext == "cjs" {
            let ts_ext = match ext {
                "mjs" => "mts",
                "cjs" => "cts",
                _ => "ts",
            };
            let ts_path = path.with_file_name(format!("{}.{}", stem, ts_ext));
            if ts_path.exists() {
                return ts_path;
            }
            let tsx_path = path.with_file_name(format!("{}.tsx", stem));
            if tsx_path.exists() {
                return tsx_path;
            }
        }
    }

    path
}

/// Check if a path likely refers to a TypeScript file.
fn likely_typescript_file(path: &Path) -> bool {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        let parent = path.parent().unwrap_or(Path::new("."));
        let ts_path = parent.join(format!("{}.ts", stem));
        let tsx_path = parent.join(format!("{}.tsx", stem));
        return ts_path.exists() || tsx_path.exists();
    }
    false
}

/// Extract entry points from package.json exports field.
fn extract_exports_entry_points(
    exports: &serde_json::Value,
    pkg_dir: &Path,
    entry_files: &mut Vec<PathBuf>,
) {
    match exports {
        serde_json::Value::String(path) => {
            entry_files.push(normalize_entry_path(pkg_dir, path));
        }
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                // Handle conditional exports
                if key == "import" || key == "require" || key == "default" || key == "types" {
                    extract_exports_entry_points(value, pkg_dir, entry_files);
                } else if key.starts_with('.') {
                    // Subpath export
                    extract_exports_entry_points(value, pkg_dir, entry_files);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_exports_entry_points(item, pkg_dir, entry_files);
            }
        }
        _ => {}
    }
}

/// Partial package.json structure for entry point extraction.
#[derive(Debug, serde::Deserialize)]
struct PackageJson {
    main: Option<String>,
    module: Option<String>,
    types: Option<String>,
    bin: Option<BinField>,
    exports: Option<serde_json::Value>,
}

/// The "bin" field can be a string or a map.
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum BinField {
    Single(String),
    Map(std::collections::HashMap<String, String>),
}

/// Generate default ddd.toml content.
pub fn generate_default_config() -> String {
    let config = Config::default();
    toml::to_string_pretty(&config).unwrap_or_else(|_| String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let (config, path) = load_config(temp_dir.path()).unwrap();
        assert!(path.is_none());
        assert!(config.entry.auto_detect);
    }

    #[test]
    fn test_load_toml_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("ddd.toml");

        std::fs::write(
            &config_path,
            r#"
            [entry]
            autoDetect = false

            [output]
            format = "json"
            "#,
        )
        .unwrap();

        let (config, path) = load_config(temp_dir.path()).unwrap();
        assert!(path.is_some());
        assert!(!config.entry.auto_detect);
    }

    #[test]
    fn test_generate_default_config() {
        let config_str = generate_default_config();
        assert!(config_str.contains("[entry]"));
        assert!(config_str.contains("[output]"));
    }
}
