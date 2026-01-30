//! Jest framework detector.

use crate::plugins::registry::FrameworkDetector;

/// Detector for Jest test framework.
pub struct JestDetector;

impl JestDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JestDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameworkDetector for JestDetector {
    fn name(&self) -> &'static str {
        "jest"
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
            "**/__tests__/**/*.js".to_string(),
            "**/__tests__/**/*.jsx".to_string(),
            // Config files
            "jest.config.js".to_string(),
            "jest.config.ts".to_string(),
            "jest.config.mjs".to_string(),
            "jest.setup.js".to_string(),
            "jest.setup.ts".to_string(),
            "setupTests.js".to_string(),
            "setupTests.ts".to_string(),
        ]
    }

    fn get_special_exports(&self) -> Vec<&'static str> {
        vec![
            // Jest globals (these are called by Jest, not imported)
            "describe",
            "it",
            "test",
            "expect",
            "beforeAll",
            "afterAll",
            "beforeEach",
            "afterEach",
            // Setup/teardown
            "setup",
            "teardown",
            // Custom matchers
            "toMatchSnapshot",
            // Default export for config
            "default",
        ]
    }

    fn detect_from_dependencies(&self, deps: &[String]) -> bool {
        deps.iter().any(|d| d == "jest" || d == "@jest/core")
    }
}
