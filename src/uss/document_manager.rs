//! USS Document Manager
//!
//! Manages multiple USS documents and provides operations for document lifecycle.

use std::collections::HashMap;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::uss::parser::UssParser;
use super::document::UssDocument;

/// Document manager for USS files
pub struct UssDocumentManager {
    documents: HashMap<Url, UssDocument>,
    parser: UssParser,
}

impl UssDocumentManager {
    /// Create a new document manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            documents: HashMap::new(),
            parser: UssParser::new()?,
        })
    }
    
    /// Open a new document
    pub fn open_document(&mut self, uri: Url, content: String, version: i32) {
        let mut document = UssDocument::new(uri.clone(), content, version);
        document.parse(&mut self.parser);
        self.documents.insert(uri, document);
    }
    
    /// Update an existing document
    pub fn update_document(
        &mut self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    ) {
        if let Some(document) = self.documents.get_mut(uri) {
            document.apply_changes(changes, version, &mut self.parser);
        }
    }
    
    /// Close a document
    pub fn close_document(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }
    
    /// Get a document reference
    pub fn get_document(&self, uri: &Url) -> Option<&UssDocument> {
        self.documents.get(uri)
    }
    
    /// Get a mutable document reference
    pub fn get_document_mut(&mut self, uri: &Url) -> Option<&mut UssDocument> {
        self.documents.get_mut(uri)
    }
    
    /// Get all document URIs
    pub fn document_uris(&self) -> impl Iterator<Item = &Url> {
        self.documents.keys()
    }
    
    /// Invalidate diagnostics for a specific document
    pub fn invalidate_document_diagnostics(&mut self, uri: &Url) {
        if let Some(document) = self.documents.get_mut(uri) {
            document.invalidate_diagnostics();
        }
    }
    
    /// Invalidate diagnostics for all documents (useful for dependency changes)
    pub fn invalidate_all_diagnostics(&mut self) {
        for document in self.documents.values_mut() {
            document.invalidate_diagnostics();
        }
    }
    
    /// Check if a document has valid cached diagnostics
    pub fn has_valid_diagnostics(&self, uri: &Url) -> bool {
        self.documents
            .get(uri)
            .map(|doc| doc.are_diagnostics_valid())
            .unwrap_or(false)
    }
}

impl Default for UssDocumentManager {
    fn default() -> Self {
        Self::new().expect("Failed to create USS document manager")
    }
}