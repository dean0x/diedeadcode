//! Plugin registry and trait definitions.

/// Trait for framework-specific entry point detection.
pub trait FrameworkDetector: Send + Sync {
    /// Name of the framework.
    fn name(&self) -> &'static str;

    /// Glob patterns for files that should be treated as entry points.
    fn get_entry_patterns(&self) -> Vec<String>;

    /// Special export names that are entry points (e.g., getServerSideProps).
    fn get_special_exports(&self) -> Vec<&'static str>;

    /// Check if this framework is present based on package.json dependencies.
    fn detect_from_dependencies(&self, deps: &[String]) -> bool;
}

/// Registry of all available framework detectors.
pub struct PluginRegistry {
    detectors: Vec<Box<dyn FrameworkDetector>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    /// Register a framework detector.
    pub fn register(&mut self, detector: Box<dyn FrameworkDetector>) {
        self.detectors.push(detector);
    }

    /// Get all registered detectors.
    pub fn detectors(&self) -> &[Box<dyn FrameworkDetector>] {
        &self.detectors
    }

    /// Create a registry with all built-in plugins.
    pub fn with_builtins() -> Self {
        use super::frameworks::*;

        let mut registry = Self::new();
        registry.register(Box::new(NextJsDetector::new()));
        registry.register(Box::new(JestDetector::new()));
        registry.register(Box::new(VitestDetector::new()));
        registry.register(Box::new(ExpressDetector::new()));
        registry
    }
}
