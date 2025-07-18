//! CS Documentation module
//! 
//! This module provides functionality to extract XML documentation from C# source files
//! and create documentation assemblies for Unity projects.
//!
//! It handles two types of source locations:
//! 1. User code: Found in .csproj files in the Unity project root
//! 2. Package cache code: Found in .asmdef files within Library/PackageCache

pub mod assembly_manager;
pub mod source_utils;
pub mod docs_manager;
pub mod package_manager;
pub mod compile_utils;
pub mod docs_compiler;
pub mod source_assembly;
pub mod xml_doc_utils;
pub mod constants;
pub mod error;

// Re-export common types
pub use error::{CsError, CsResult};