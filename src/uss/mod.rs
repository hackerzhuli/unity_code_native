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
pub mod keyword_data;
pub mod color;
pub mod color_keywords;
pub mod value_spec;
pub mod color_provider;
pub mod completion;
pub mod variable_resolver;
pub mod value;
pub mod uss_utils;
pub mod constants;
pub mod import_node;
pub mod function_node;
pub mod url_function_node;
pub mod flexible_format;
pub mod formatter;

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

#[cfg(test)]
mod url_function_node_tests;

#[cfg(test)]
mod import_node_tests;

#[cfg(test)]
mod function_node_tests;

#[cfg(test)]
mod flexible_format_tests;