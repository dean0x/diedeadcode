//! Error types for dead code analysis.

// Suppress false positive warnings from thiserror/miette derive macros
// The fields are used in error message templates but the generated code
// triggers "value assigned but never read" warnings
#![allow(unused_assignments)]

use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the dead code analyzer.
#[derive(Error, Debug, Diagnostic)]
pub enum DddError {
    #[error("Failed to parse file: {path}")]
    #[diagnostic(code(ddd::parse_error), help("Check for syntax errors in the file"))]
    ParseError {
        path: PathBuf,
        #[source]
        source: ParseErrorDetails,
    },

    #[error("Failed to resolve import: {specifier} from {from_file}")]
    #[diagnostic(code(ddd::resolve_error), help("Check that the module exists and the path is correct"))]
    ResolveError {
        specifier: String,
        from_file: PathBuf,
    },

    #[error("Configuration error: {message}")]
    #[diagnostic(code(ddd::config_error))]
    ConfigError { message: String },

    #[error("Failed to read file: {path}")]
    #[diagnostic(code(ddd::io_error))]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read directory: {path}")]
    #[diagnostic(code(ddd::io_error))]
    DirectoryError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("No TypeScript/JavaScript files found in: {path}")]
    #[diagnostic(code(ddd::no_files), help("Specify a directory containing .ts, .tsx, .js, or .jsx files"))]
    NoFilesFound { path: PathBuf },

    #[error("Invalid glob pattern: {pattern}")]
    #[diagnostic(code(ddd::invalid_pattern))]
    InvalidGlobPattern {
        pattern: String,
        #[source]
        source: glob::PatternError,
    },

    #[error("Analysis failed: {message}")]
    #[diagnostic(code(ddd::analysis_error))]
    AnalysisError { message: String },

    #[error("Plugin error: {plugin_name} - {message}")]
    #[diagnostic(code(ddd::plugin_error))]
    PluginError { plugin_name: String, message: String },
}

/// Details about a parse error.
#[derive(Error, Debug)]
#[error("{message} at line {line}, column {column}")]
pub struct ParseErrorDetails {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

impl ParseErrorDetails {
    pub fn new(message: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            message: message.into(),
            line,
            column,
        }
    }
}

/// Result type alias for operations that can fail with DddError.
pub type Result<T> = std::result::Result<T, DddError>;

impl DddError {
    /// Create a parse error from file path and error details.
    pub fn parse_error(path: PathBuf, message: impl Into<String>, line: u32, column: u32) -> Self {
        Self::ParseError {
            path,
            source: ParseErrorDetails::new(message, line, column),
        }
    }

    /// Create a resolve error.
    pub fn resolve_error(specifier: impl Into<String>, from_file: PathBuf) -> Self {
        Self::ResolveError {
            specifier: specifier.into(),
            from_file,
        }
    }

    /// Create a config error.
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    /// Create an IO error for file operations.
    pub fn io_error(path: PathBuf, source: std::io::Error) -> Self {
        Self::IoError { path, source }
    }

    /// Create an IO error for directory operations.
    pub fn directory_error(path: PathBuf, source: std::io::Error) -> Self {
        Self::DirectoryError { path, source }
    }

    /// Create a no files found error.
    pub fn no_files_found(path: PathBuf) -> Self {
        Self::NoFilesFound { path }
    }

    /// Create an analysis error.
    pub fn analysis_error(message: impl Into<String>) -> Self {
        Self::AnalysisError {
            message: message.into(),
        }
    }

    /// Create a plugin error.
    pub fn plugin_error(plugin_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::PluginError {
            plugin_name: plugin_name.into(),
            message: message.into(),
        }
    }
}

/// Extension trait for converting std::io::Result to DddError.
pub trait IoResultExt<T> {
    fn map_io_err(self, path: PathBuf) -> Result<T>;
    fn map_dir_err(self, path: PathBuf) -> Result<T>;
}

impl<T> IoResultExt<T> for std::io::Result<T> {
    fn map_io_err(self, path: PathBuf) -> Result<T> {
        self.map_err(|e| DddError::io_error(path, e))
    }

    fn map_dir_err(self, path: PathBuf) -> Result<T> {
        self.map_err(|e| DddError::directory_error(path, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DddError::parse_error(
            PathBuf::from("test.ts"),
            "Unexpected token",
            10,
            5,
        );
        assert!(err.to_string().contains("test.ts"));
    }

    #[test]
    fn test_config_error() {
        let err = DddError::config_error("Invalid entry point");
        assert!(err.to_string().contains("Invalid entry point"));
    }
}
