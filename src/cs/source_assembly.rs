use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Represents lightweight information about a C# assembly source code
/// Source files are derived from source_location when needed to keep this struct lightweight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceAssembly {
    /// The name of the assembly (e.g., "Assembly-CSharp"), no extension
    pub name: String,
    /// Whether this is user code (true) or package code (false)
    pub is_user_code: bool,
    /// The location where this assembly info was found (csproj or asmdef path)
    /// For .csproj files: source files can be read from the project file
    /// For .asmdef files: source files can be found by searching the parent directory of the asmdef file
    pub source_location: PathBuf,
}