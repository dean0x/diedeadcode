//! Vitest framework detector.

use crate::plugins::registry::FrameworkDetector;

/// Detector for Vitest test framework.
pub struct VitestDetector;

impl VitestDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VitestDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameworkDetector for VitestDetector {
    fn name(&self) -> &'static str {
        "vitest"
    }

    fn get_entry_patterns(&self) -> Vec<String> {
        vec![
            // Test files
            "**/*.test.ts".to_string(),
            "**/*.test.tsx".to_string(),
            "**/*.test.js".to_string(),
            "**/*.test.jsx".to_string(),
            "**/*.spec.ts".to_string(),
            "**/*.spec.tsx".to_string(),
            "**/*.spec.js".to_string(),
            "**/*.spec.jsx".to_string(),
            // Test directories
            "**/__tests__/**/*.ts".to_string(),
            "**/__tests__/**/*.tsx".to_string(),
            // Config files
            "vitest.config.ts".to_string(),
            "vitest.config.js".to_string(),
            "vitest.config.mts".to_string(),
            "vitest.setup.ts".to_string(),
            "vitest.setup.js".to_string(),
        ]
    }

    fn get_special_exports(&self) -> Vec<&'static str> {
        vec![
            // Vitest globals
            "describe",
            "it",
            "test",
            "expect",
            "vi",
            "beforeAll",
            "afterAll",
            "beforeEach",
            "afterEach",
            // Benchmark
            "bench",
            "suite",
            // Default export for config
            "default",
        ]
    }

    fn detect_from_dependencies(&self, deps: &[String]) -> bool {
        deps.iter().any(|d| d == "vitest")
    }
}
