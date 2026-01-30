//! Express.js framework detector.

use crate::plugins::registry::FrameworkDetector;

/// Detector for Express.js applications.
pub struct ExpressDetector;

impl ExpressDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExpressDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameworkDetector for ExpressDetector {
    fn name(&self) -> &'static str {
        "express"
    }

    fn get_entry_patterns(&self) -> Vec<String> {
        vec![
            // Common entry points
            "src/index.ts".to_string(),
            "src/index.js".to_string(),
            "src/app.ts".to_string(),
            "src/app.js".to_string(),
            "src/server.ts".to_string(),
            "src/server.js".to_string(),
            "index.ts".to_string(),
            "index.js".to_string(),
            "app.ts".to_string(),
            "app.js".to_string(),
            "server.ts".to_string(),
            "server.js".to_string(),
            // Routes
            "src/routes/**/*.ts".to_string(),
            "src/routes/**/*.js".to_string(),
            "routes/**/*.ts".to_string(),
            "routes/**/*.js".to_string(),
            // Controllers
            "src/controllers/**/*.ts".to_string(),
            "src/controllers/**/*.js".to_string(),
            "controllers/**/*.ts".to_string(),
            "controllers/**/*.js".to_string(),
            // Middleware
            "src/middleware/**/*.ts".to_string(),
            "src/middleware/**/*.js".to_string(),
            "middleware/**/*.ts".to_string(),
            "middleware/**/*.js".to_string(),
        ]
    }

    fn get_special_exports(&self) -> Vec<&'static str> {
        vec![
            // Common middleware exports
            "router",
            "app",
            // HTTP method handlers
            "get",
            "post",
            "put",
            "delete",
            "patch",
            "options",
            "head",
            "all",
            "use",
            // Default export
            "default",
        ]
    }

    fn detect_from_dependencies(&self, deps: &[String]) -> bool {
        deps.iter().any(|d| d == "express")
    }
}
