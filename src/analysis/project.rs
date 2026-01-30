//! Project file discovery.

use crate::config::Config;
use crate::core::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Discover TypeScript/JavaScript files in the project.
pub fn discover_files(root: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check extension
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !is_typescript_extension(ext) {
            continue;
        }

        // Check include/exclude patterns
        if !config.should_include(path) {
            continue;
        }

        files.push(path.to_path_buf());
    }

    Ok(files)
}

/// Check if an extension is a TypeScript/JavaScript file.
fn is_typescript_extension(ext: &str) -> bool {
    matches!(ext, "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
}

/// Get the source type for a file based on its extension.
pub fn get_source_type(path: &Path) -> oxc::span::SourceType {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "ts" | "mts" | "cts" => oxc::span::SourceType::ts(),
        "tsx" => oxc::span::SourceType::tsx(),
        "js" | "mjs" | "cjs" => oxc::span::SourceType::mjs(),
        "jsx" => oxc::span::SourceType::jsx(),
        _ => oxc::span::SourceType::mjs(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_typescript_extension() {
        assert!(is_typescript_extension("ts"));
        assert!(is_typescript_extension("tsx"));
        assert!(is_typescript_extension("js"));
        assert!(is_typescript_extension("jsx"));
        assert!(is_typescript_extension("mts"));
        assert!(!is_typescript_extension("rs"));
        assert!(!is_typescript_extension("py"));
    }

    #[test]
    fn test_get_source_type() {
        let ts_type = get_source_type(Path::new("foo.ts"));
        assert!(ts_type.is_typescript());

        let tsx_type = get_source_type(Path::new("foo.tsx"));
        assert!(tsx_type.is_typescript());
        assert!(tsx_type.is_jsx());

        let js_type = get_source_type(Path::new("foo.js"));
        assert!(js_type.is_javascript());
    }
}
