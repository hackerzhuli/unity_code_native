//! USS Document Manager
//!
//! Manages multiple USS documents and provides operations for document lifecycle.

use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::uss::definitions::UssDefinitions;
use crate::uss::parser::UssParser;
use crate::language::document::DocumentVersion;
use super::document::UssDocument;

/// Document manager for USS files
pub struct UssDocumentManager {
    documents: HashMap<Url, UssDocument>,
    parser: UssParser,
    definitions: Arc<UssDefinitions>,
}

impl UssDocumentManager {
    /// Create a new document manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            documents: HashMap::new(),
            parser: UssParser::new()?,
            definitions: Arc::new(UssDefinitions::new()),
        })
    }
    
    /// Open a new document
    pub fn open_document(&mut self, uri: Url, content: String, version: i32) {
        // Since closed documents are removed from memory, we always create a new document
        let mut document = UssDocument::new(uri.clone(), content, version, self.definitions.clone());
        document.mark_opened(version);
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
    
    /// Close a document and remove it from memory
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
    
    /// Check if a document is currently open in a client
    pub fn is_document_open(&self, uri: &Url) -> bool {
        // Since closed documents are removed from memory, existence means it's open
        self.documents.contains_key(uri)
    }
}

impl Default for UssDocumentManager {
    fn default() -> Self {
        Self::new().expect("Failed to create USS document manager")
    }
}