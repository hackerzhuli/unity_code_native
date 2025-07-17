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

pub mod assembly_manager;
pub mod source_finder;
pub mod docs_manager;
pub mod package_manager;

// Re-export the main managers
pub use assembly_manager::AssemblyManager;
pub use docs_manager::CsDocsManager;
pub use package_manager::UnityPackageManager;

pub mod source_assembly;