//! USS Document Management
//!
//! Provides document state management and incremental parsing for USS files.

use std::collections::HashMap;
use tower_lsp::lsp_types::{Diagnostic, Position, Range, TextDocumentContentChangeEvent, Url};
use tree_sitter::{InputEdit, Point, Tree};

use crate::uss::parser::UssParser;

/// Represents a USS document with its content, syntax tree, and version
#[derive(Debug, Clone)]
pub struct UssDocument {
    /// Document URI
    pub uri: Url,
    /// Current document content
    pub content: String,
    /// Current syntax tree
    pub tree: Option<Tree>,
    /// Document version for LSP synchronization
    pub version: i32,
    /// Line start positions for efficient position calculations
    line_starts: Vec<usize>,
    /// Cached diagnostics for this document
    cached_diagnostics: Option<Vec<Diagnostic>>,
    /// Whether the cached diagnostics are valid (not invalidated by changes)
    diagnostics_valid: bool,
}

impl UssDocument {
    /// Create a new USS document
    pub fn new(uri: Url, content: String, version: i32) -> Self {
        let line_starts = Self::calculate_line_starts(&content);
        Self {
            uri,
            content,
            tree: None,
            version,
            line_starts,
            cached_diagnostics: None,
            diagnostics_valid: false,
        }
    }
    
    /// Parse the document content and store the syntax tree
    pub fn parse(&mut self, parser: &mut UssParser) {
        self.tree = parser.parse(&self.content, None);
        // Invalidate diagnostics when content is parsed
        self.invalidate_diagnostics();
    }
    
    /// Apply incremental changes to the document
    pub fn apply_changes(
        &mut self,
        changes: Vec<TextDocumentContentChangeEvent>,
        new_version: i32,
        parser: &mut UssParser,
    ) {
        self.version = new_version;
        
        // Invalidate diagnostics when content changes
        self.invalidate_diagnostics();
        
        for change in changes {
            if let Some(range) = change.range {
                // Incremental change
                self.apply_incremental_change(change, range, parser);
            } else {
                // Full document change
                self.content = change.text;
                self.line_starts = Self::calculate_line_starts(&self.content);
                self.tree = parser.parse(&self.content, None);
            }
        }
    }
    
    /// Apply an incremental change to the document
    fn apply_incremental_change(
        &mut self,
        change: TextDocumentContentChangeEvent,
        range: Range,
        parser: &mut UssParser,
    ) {
        let start_byte = self.position_to_byte(range.start);
        let end_byte = self.position_to_byte(range.end);
        
        // Apply the text change
        let new_content = format!(
            "{}{}{}",
            &self.content[..start_byte],
            change.text,
            &self.content[end_byte..]
        );
        
        // Calculate the edit for tree-sitter
        let old_end_byte = end_byte;
        let new_end_byte = start_byte + change.text.len();
        
        let start_point = self.position_to_point(range.start);
        let old_end_point = self.position_to_point(range.end);
        
        // Calculate new end point
        let new_end_point = if change.text.contains('\n') {
            let lines: Vec<&str> = change.text.split('\n').collect();
            let line_count = lines.len() - 1;
            Point {
                row: start_point.row + line_count,
                column: if line_count > 0 {
                    lines.last().unwrap().len()
                } else {
                    start_point.column + change.text.len()
                },
            }
        } else {
            Point {
                row: start_point.row,
                column: start_point.column + change.text.len(),
            }
        };
        
        let edit = InputEdit {
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position: start_point,
            old_end_position: old_end_point,
            new_end_position: new_end_point,
        };
        
        // Update the tree incrementally if we have one
        if let Some(ref mut tree) = self.tree {
            tree.edit(&edit);
        }
        
        // Update content and line starts
        self.content = new_content;
        self.line_starts = Self::calculate_line_starts(&self.content);
        
        // Re-parse with the old tree for incremental parsing
        self.tree = parser.parse(&self.content, self.tree.as_ref());
    }
    
    /// Convert LSP position to byte offset
    fn position_to_byte(&self, position: Position) -> usize {
        let line = position.line as usize;
        let character = position.character as usize;
        
        if line >= self.line_starts.len() {
            return self.content.len();
        }
        
        let line_start_byte = self.line_starts[line];
        
        // Get the line content to properly handle character to byte conversion
        let line_end_byte = if line + 1 < self.line_starts.len() {
            self.line_starts[line + 1] - 1 // -1 to exclude the newline
        } else {
            self.content.len()
        };
        
        let line_content = &self.content[line_start_byte..line_end_byte];
        
        // Convert character position to byte position within the line
        let mut char_count = 0;
        for (byte_offset, _) in line_content.char_indices() {
            if char_count == character {
                return line_start_byte + byte_offset;
            }
            char_count += 1;
        }
        
        // If character position is at or beyond the end of the line
        line_start_byte + line_content.len()
    }
    
    /// Convert LSP position to tree-sitter Point
    fn position_to_point(&self, position: Position) -> Point {
        Point {
            row: position.line as usize,
            column: position.character as usize,
        }
    }
    
    /// Calculate line start positions for efficient position calculations
    fn calculate_line_starts(content: &str) -> Vec<usize> {
        let mut line_starts = vec![0];
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        line_starts
    }
    
    /// Get the syntax tree reference
    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }
    
    /// Get the document content
    pub fn content(&self) -> &str {
        &self.content
    }
    
    /// Get the document version
    pub fn version(&self) -> i32 {
        self.version
    }
    
    /// Get cached diagnostics if they are valid
    pub fn get_cached_diagnostics(&self) -> Option<&Vec<Diagnostic>> {
        if self.diagnostics_valid {
            self.cached_diagnostics.as_ref()
        } else {
            None
        }
    }
    
    /// Cache diagnostics for this document
    pub fn cache_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.cached_diagnostics = Some(diagnostics);
        self.diagnostics_valid = true;
    }
    
    /// Invalidate cached diagnostics
    pub fn invalidate_diagnostics(&mut self) {
        self.diagnostics_valid = false;
    }
    
    /// Check if cached diagnostics are valid
    pub fn are_diagnostics_valid(&self) -> bool {
        self.diagnostics_valid
    }
}

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