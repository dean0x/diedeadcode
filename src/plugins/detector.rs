//! Framework auto-detection from package.json.

use super::frameworks::*;
use super::registry::{FrameworkDetector, PluginRegistry};
use crate::config::Config;
use crate::core::Result;
use std::path::Path;

/// Detect frameworks from package.json and config.
pub fn detect_frameworks(
    root_path: &Path,
    config: &Config,
) -> Result<Vec<Box<dyn FrameworkDetector>>> {
    let mut detectors: Vec<Box<dyn FrameworkDetector>> = Vec::new();
    let registry = PluginRegistry::with_builtins();

    // Get dependencies from package.json
    let deps = read_dependencies(root_path);

    // Check explicitly enabled plugins
    for name in &config.plugins.enabled {
        if let Some(detector) = find_detector_by_name(&registry, name) {
            detectors.push(detector);
        }
    }

    // Auto-detect if enabled
    if config.plugins.auto_detect {
        for builtin in registry.detectors() {
            let name = builtin.name();

            // Skip if explicitly disabled
            if config.plugins.disabled.contains(&name.to_string()) {
                continue;
            }

            // Skip if already enabled
            if config.plugins.enabled.contains(&name.to_string()) {
                continue;
            }

            // Check if framework is detected
            if builtin.detect_from_dependencies(&deps) {
                detectors.push(create_detector(name));
            }
        }
    }

    Ok(detectors)
}

/// Read all dependencies from package.json.
fn read_dependencies(root_path: &Path) -> Vec<String> {
    let package_json_path = root_path.join("package.json");

    if !package_json_path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&package_json_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let pkg: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut deps = Vec::new();

    // Collect from dependencies
    if let Some(obj) = pkg.get("dependencies").and_then(|v| v.as_object()) {
        deps.extend(obj.keys().cloned());
    }

    // Collect from devDependencies
    if let Some(obj) = pkg.get("devDependencies").and_then(|v| v.as_object()) {
        deps.extend(obj.keys().cloned());
    }

    // Collect from peerDependencies
    if let Some(obj) = pkg.get("peerDependencies").and_then(|v| v.as_object()) {
        deps.extend(obj.keys().cloned());
    }

    deps
}

/// Find a detector by name in the registry.
fn find_detector_by_name(
    registry: &PluginRegistry,
    name: &str,
) -> Option<Box<dyn FrameworkDetector>> {
    for detector in registry.detectors() {
        if detector.name() == name {
            return Some(create_detector(name));
        }
    }
    None
}

/// Create a detector instance by name.
fn create_detector(name: &str) -> Box<dyn FrameworkDetector> {
    match name {
        "nextjs" => Box::new(NextJsDetector::new()),
        "jest" => Box::new(JestDetector::new()),
        "vitest" => Box::new(VitestDetector::new()),
        "express" => Box::new(ExpressDetector::new()),
        _ => Box::new(NextJsDetector::new()), // Fallback, shouldn't happen
    }
}
