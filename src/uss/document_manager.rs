//! USS Document Manager
//!
//! Manages multiple USS documents and provides operations for document lifecycle.

use std::collections::HashMap;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::uss::parser::UssParser;
use crate::language::document::DocumentVersion;
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
        let mut document = if let Some(existing_doc) = self.documents.get(&uri) {
            // Document was previously known (possibly closed), reuse its major version
            let mut new_major = existing_doc.document_version().major;
            if !existing_doc.is_open() {
                new_major += 1; // Increment major version when reopening
            }
            UssDocument::new_with_document_version(
                uri.clone(),
                content,
                DocumentVersion { major: new_major, minor: version },
                true
            )
        } else {
            // New document
            let mut document = UssDocument::new(uri.clone(), content, version);
            document.mark_opened(version);
            document
        };
        
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
        if let Some(document) = self.documents.get_mut(uri) {
            document.mark_closed();
            // Keep the document in memory to preserve version history
            // Remove it only if explicitly requested or after a timeout
        }
    }
    
    /// Remove a document completely from memory
    pub fn remove_document(&mut self, uri: &Url) {
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
        self.documents
            .get(uri)
            .map(|doc| doc.is_open())
            .unwrap_or(false)
    }
}

impl Default for UssDocumentManager {
    fn default() -> Self {
        Self::new().expect("Failed to create USS document manager")
    }
}