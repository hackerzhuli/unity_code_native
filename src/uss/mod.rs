//! USS (Unity Style Sheet) Language Server
//!
//! Provides IDE features for Unity's UI Toolkit styling language using:
//! - tree-sitter-css for parsing (USS syntax is nearly identical to CSS)
//! - tower-lsp for Language Server Protocol implementation

pub mod server;
pub mod parser;
pub mod diagnostics;
pub mod highlighting;
pub mod definitions;
pub mod hover;
pub mod property_data;
pub mod tree_printer;
pub mod value_spec;

#[cfg(test)]
mod diagnostics_tests;