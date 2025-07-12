//! USS (Unity Style Sheet) Language Server
//!
//! Provides IDE features for Unity's UI Toolkit styling language using:
//! - tree-sitter-css for parsing (USS syntax is nearly identical to CSS)
//! - tower-lsp for Language Server Protocol implementation

pub mod server;
pub mod parser;
pub mod document;
pub mod document_manager;
pub mod diagnostics;
pub mod highlighting;
pub mod definitions;
pub mod hover;
pub mod property_data;
pub mod color;
pub mod tree_printer;
pub mod value_spec;
pub mod color_provider;
pub mod variable_resolver;
pub mod value;
pub mod uss_utils;
pub mod constants;
pub mod import_node;

#[cfg(test)]
mod diagnostics_tests;

#[cfg(test)]
mod value_spec_tests;

#[cfg(test)]
mod document_tests;

#[cfg(test)]
mod value_tests;

#[cfg(test)]
mod variable_resolver_tests;