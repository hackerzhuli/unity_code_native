//! Error types for the CS module
//!
//! This module defines custom error types using thiserror for better error handling
//! throughout the CS documentation system.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for CS module operations
#[derive(Error, Debug)]
pub enum CsError {
    /// IO errors (file operations, directory access, etc.)
    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: std::io::Error,
    },

    /// JSON parsing/serialization errors
    #[error("JSON error: {message}")]
    Json {
        message: String,
        #[source]
        source: serde_json::Error,
    },

    /// Assembly-related errors
    #[error("Assembly error: {message}")]
    Assembly { message: String },

    /// Documentation compilation errors
    #[error("Documentation compilation error: {message}")]
    DocsCompilation { message: String },

    /// Tree-sitter parsing errors
    #[error("Parse error in file {file:?}: {message}")]
    Parse { file: PathBuf, message: String },

    /// Symbol not found errors
    #[error("Symbol '{symbol}' not found in assembly '{assembly}'")]
    SymbolNotFound { symbol: String, assembly: String },

    /// Assembly not found errors
    #[error("Assembly '{name}' not found")]
    AssemblyNotFound { name: String },

    /// Source file not found errors
    #[error("Source file not found: {path:?}")]
    SourceFileNotFound { path: PathBuf },

    /// Missing required parameter errors
    #[error("Missing required parameter: {parameter}")]
    MissingParameter { parameter: String },

    /// Package manager errors
    #[error("Package manager error: {message}")]
    PackageManager { message: String },

    /// XML parsing errors
    #[error("XML parsing error: {message}")]
    XmlParsing { message: String },

    /// File system metadata errors
    #[error("Failed to get metadata for {file:?}: {message}")]
    Metadata { file: PathBuf, message: String },

    /// Tree-sitter language setup errors
    #[error("Failed to set up tree-sitter language: {message}")]
    TreeSitterLanguage { message: String },

    /// No assembly specified error
    #[error("No assembly specified")]
    NoAssemblySpecified,

    /// No documentation available error
    #[error("No documentation available for assembly '{assembly}'")]
    NoDocumentationAvailable { assembly: String },
}

/// Result type alias for CS operations
pub type CsResult<T> = Result<T, CsError>;

// Implement From traits for automatic error conversion
impl From<std::io::Error> for CsError {
    fn from(err: std::io::Error) -> Self {
        CsError::Io {
            source: err,
            message: "IO operation failed".to_string(),
        }
    }
}

impl From<serde_json::Error> for CsError {
    fn from(err: serde_json::Error) -> Self {
        CsError::Json {
            source: err,
            message: "JSON operation failed".to_string(),
        }
    }
}

/// Helper trait for converting IO errors with context
pub trait IoContext<T> {
    fn with_io_context(self, message: &str) -> CsResult<T>;
}

impl<T> IoContext<T> for Result<T, std::io::Error> {
    fn with_io_context(self, message: &str) -> CsResult<T> {
        self.map_err(|e| CsError::Io {
            message: message.to_string(),
            source: e,
        })
    }
}

/// Helper trait for converting JSON errors with context
pub trait JsonContext<T> {
    fn with_json_context(self, message: &str) -> CsResult<T>;
}

impl<T> JsonContext<T> for Result<T, serde_json::Error> {
    fn with_json_context(self, message: &str) -> CsResult<T> {
        self.map_err(|e| CsError::Json {
            message: message.to_string(),
            source: e,
        })
    }
}