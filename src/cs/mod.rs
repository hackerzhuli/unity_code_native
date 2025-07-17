//! CS Documentation module
//! 
//! This module provides functionality to extract XML documentation from C# source files
//! and create documentation assemblies for Unity projects.
//!
//! It handles two types of source locations:
//! 1. User code: Found in .csproj files in the Unity project root
//! 2. Package cache code: Found in .asmdef files within Library/PackageCache

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod assembly_finder;
pub mod source_finder;
pub mod manager;

// Re-export the main manager
pub use manager::CsDocsManager;

/// Represents information about a C# assembly and its source files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyInfo {
    /// The name of the assembly (e.g., "Assembly-CSharp")
    pub name: String,
    /// List of source file paths relative to Unity project root
    pub source_files: Vec<PathBuf>,
    /// Whether this is user code (true) or package code (false)
    pub is_user_code: bool,
    /// The location where this assembly info was found (csproj or asmdef path)
    pub source_location: PathBuf,
}