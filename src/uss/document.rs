//! USS Document
//!
//! Represents a single USS document with its content, syntax tree, and version.
use std::collections::HashMap;
use tower_lsp::lsp_types::{Diagnostic, Position, Range, TextDocumentContentChangeEvent, Url};
use tree_sitter::{InputEdit, Point, Tree};

use crate::uss::parser::UssParser;
use crate::uss::variable_resolver::{VariableResolver, VariableStatus};
use crate::language::document::DocumentVersion;

/// Represents a USS document with its content, syntax tree, and version
#[derive(Debug, Clone)]
pub struct UssDocument {
    /// Document URI
    pub uri: Url,
    /// Current document content
    pub content: String,
    /// Current syntax tree
    pub tree: Option<Tree>,
    /// Enhanced document version tracking
    pub document_version: DocumentVersion,
    /// Whether the document is currently open in a client
    pub is_open: bool,
    /// Line start positions for efficient position calculations
    line_starts: Vec<usize>,
    /// Variable resolver for CSS custom properties
    pub variable_resolver: VariableResolver,
}

impl UssDocument {
    /// Create a new USS document
    pub fn new(uri: Url, content: String, version: i32) -> Self {
        let line_starts = Self::calculate_line_starts(&content);
        Self {
            uri,
            content,
            tree: None,
            document_version: DocumentVersion { major: 1, minor: version },
            is_open: false,
            line_starts,
            variable_resolver: VariableResolver::new(),
        }
    }
    
    /// Create a new USS document with explicit document version
    pub fn new_with_document_version(uri: Url, content: String, version: i32, document_version: DocumentVersion, is_open: bool) -> Self {
        let line_starts = Self::calculate_line_starts(&content);
        Self {
            uri,
            content,
            tree: None,
            document_version,
            is_open,
            line_starts,
            variable_resolver: VariableResolver::new(),
        }
    }
    
    /// Parse the document content and store the syntax tree
    pub fn parse(&mut self, parser: &mut UssParser) {
        self.tree = parser.parse(&self.content, None);
        // Extract and resolve variables after parsing
        if let Some(tree) = &self.tree {
            self.variable_resolver.add_variables_from_tree(tree.root_node(), &self.content);
        }
    }
    
    /// Apply incremental changes to the document
    pub fn apply_changes(
        &mut self,
        changes: Vec<TextDocumentContentChangeEvent>,
        new_version: i32,
        parser: &mut UssParser,
    ) {
        // Update document version minor when content changes (if document is open)
        if self.is_open {
            self.document_version.minor = new_version;
        }
        
        // Clear variable resolver when content changes
        self.variable_resolver.clear();
        
        for change in changes {
            if let Some(range) = change.range {
                // Incremental change
                self.apply_incremental_change(change, range, parser);
            } else {
                // Full document change
                self.content = change.text;
                self.line_starts = Self::calculate_line_starts(&self.content);
                self.tree = parser.parse(&self.content, None);
                
                // Re-extract and resolve variables after full document change
                if let Some(tree) = &self.tree {
                    self.variable_resolver.add_variables_from_tree(tree.root_node(), &self.content);
                }
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
        
        // Re-extract and resolve variables after incremental parsing
        if let Some(tree) = &self.tree {
            self.variable_resolver.add_variables_from_tree(tree.root_node(), &self.content);
        }
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
    

    


    /// Get all variables defined in this document
    /// 
    /// **Note**: The variable resolver has limitations:
    /// - Only resolves variables defined within this document
    /// - Does not support imported variables from other USS files
    /// - Variable resolution may be incomplete in complex scenarios
    pub fn get_variables(&self) -> &HashMap<String, VariableStatus> {
        self.variable_resolver.get_variables()
    }

    /// Get a specific variable by name
    /// 
    /// **Note**: The variable resolver has limitations:
    /// - Only resolves variables defined within this document
    /// - Does not support imported variables from other USS files
    /// - Variable resolution may be incomplete in complex scenarios
    pub fn get_variable(&self, name: &str) -> Option<&VariableStatus> {
        self.variable_resolver.get_variable(name)
    }

    /// Check if variables have been resolved
    /// 
    /// **Note**: The variable resolver has limitations:
    /// - Only resolves variables defined within this document
    /// - Does not support imported variables from other USS files
    /// - Variable resolution may be incomplete in complex scenarios
    pub fn are_variables_resolved(&self) -> bool {
        self.variable_resolver.are_variables_resolved()
    }
    
    /// Re-extract and resolve variables with a source URL for proper relative URL resolution
    /// This should be called after parsing when the project URL is available
    pub fn extract_variables_with_source_url(&mut self, source_url: Option<&Url>) {
        if let Some(tree) = &self.tree {
            self.variable_resolver.add_variables_from_tree_with_source_url(tree.root_node(), &self.content, source_url);
        }
    }

    /// Convert byte offset to LSP position
    pub fn byte_to_position(&self, byte: usize) -> Position {
        if byte == 0 {
            return Position { line: 0, character: 0 };
        }
        
        // Find the line containing this byte
        let mut line = 0;
        for (i, &line_start) in self.line_starts.iter().enumerate() {
            if byte < line_start {
                line = i.saturating_sub(1);
                break;
            }
            line = i;
        }
        
        // Calculate character position within the line
        let line_start_byte = self.line_starts[line];
        let line_end_byte = if line + 1 < self.line_starts.len() {
            self.line_starts[line + 1] - 1
        } else {
            self.content.len()
        };
        
        let line_content = &self.content[line_start_byte..line_end_byte];
        let byte_in_line = byte - line_start_byte;
        
        // Convert byte position to character position
        let mut character = 0;
        for (char_byte_offset, _) in line_content.char_indices() {
            if char_byte_offset >= byte_in_line {
                break;
            }
            character += 1;
        }
        
        Position {
            line: line as u32,
            character: character as u32,
        }
    }
    
    /// Get the current document version
    pub fn document_version(&self) -> DocumentVersion {
        self.document_version
    }
    
    /// Check if the document is currently open in a client
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    
    /// Mark the document as opened in a client
    /// This increments the major version and resets minor version to the current LSP version
    pub fn mark_opened(&mut self, lsp_version: i32) {
        self.is_open = true;
        self.document_version.major += 1;
        self.document_version.minor = lsp_version;
    }
    
    /// Mark the document as closed in a client
    /// This increments the major version and resets minor version to 0
    pub fn mark_closed(&mut self) {
        self.is_open = false;
        self.document_version.major += 1;
        self.document_version.minor = 0;
    }
    
    /// Update the document version when filesystem changes are detected (for closed documents)
    /// This should only be called when the document is not open in a client
    pub fn increment_filesystem_version(&mut self) {
        if !self.is_open {
            self.document_version.minor += 1;
        }
    }
    
    /// Set the document version explicitly
    pub fn set_document_version(&mut self, version: DocumentVersion) {
        self.document_version = version;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;
    use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent};

    fn create_test_document() -> UssDocument {
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".test { color: red; }".to_string();
        UssDocument::new(uri, content, 1)
    }

    #[test]
    fn test_document_creation() {
        let doc = create_test_document();
        assert_eq!(doc.document_version().minor, 1);
        assert_eq!(doc.document_version.major, 1);
        assert_eq!(doc.document_version.minor, 1);
        assert!(!doc.is_open);
        assert_eq!(doc.content, ".test { color: red; }");
        assert!(doc.tree.is_none());
    }

    #[test]
     fn test_document_parsing() {
         let mut doc = create_test_document();
         let mut parser = UssParser::new().unwrap();
         
         doc.parse(&mut parser);
         assert!(doc.tree.is_some());
     }

    #[test]
    fn test_position_conversion() {
        let doc = create_test_document();
        
        // Test byte to position conversion
        let pos = doc.byte_to_position(0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
        
        let pos = doc.byte_to_position(5);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_position_to_byte_conversion() {
        let doc = create_test_document();
        
        let byte = doc.position_to_byte(Position { line: 0, character: 0 });
        assert_eq!(byte, 0);
        
        let byte = doc.position_to_byte(Position { line: 0, character: 5 });
        assert_eq!(byte, 5);
    }

    #[test]
     fn test_incremental_changes() {
         let mut doc = create_test_document();
         let mut parser = UssParser::new().unwrap();
         
         doc.parse(&mut parser);
         
         // Mark document as open to test version tracking
         doc.mark_opened(1);
        
        // Replace "red" (positions 15-18) with "blue"
        let changes = vec![TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position { line: 0, character: 15 },
                end: Position { line: 0, character: 18 },
            }),
            range_length: Some(3),
            text: "blue".to_string(),
        }];
        
        doc.apply_changes(changes, 2, &mut parser);
        assert_eq!(doc.document_version().minor, 2);
        assert_eq!(doc.document_version.minor, 2); // Minor version should update when open
        assert_eq!(doc.content, ".test { color: blue; }");
    }

    #[test]
    fn test_multiline_content() {
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".test {\n  color: red;\n  background: blue;\n}".to_string();
        let doc = UssDocument::new(uri, content, 1);
        
        // Test line start calculation
        let pos = doc.byte_to_position(8); // Start of second line
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);
        
        let pos = doc.byte_to_position(22); // Start of third line
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);
    }



    #[test]
    fn test_document_version_tracking() {
        let mut doc = create_test_document();
        
        // Initial state: document is closed
        assert!(!doc.is_open());
        assert_eq!(doc.document_version().major, 1);
        assert_eq!(doc.document_version().minor, 1);
        
        // Mark as opened
        doc.mark_opened(5);
        assert!(doc.is_open());
        assert_eq!(doc.document_version().major, 2); // Major incremented
        assert_eq!(doc.document_version().minor, 5); // Minor set to LSP version
        
        // Mark as closed
        doc.mark_closed();
        assert!(!doc.is_open());
        assert_eq!(doc.document_version().major, 3); // Major incremented again
        assert_eq!(doc.document_version().minor, 0); // Minor reset to 0
        
        // Test filesystem version increment (only works when closed)
        doc.increment_filesystem_version();
        assert_eq!(doc.document_version().major, 3); // Major unchanged
        assert_eq!(doc.document_version().minor, 1); // Minor incremented
        
        // Reopen and test that filesystem increment doesn't work
        doc.mark_opened(10);
        doc.increment_filesystem_version();
        assert_eq!(doc.document_version().major, 4); // Major incremented from reopening
        assert_eq!(doc.document_version().minor, 10); // Minor unchanged (filesystem increment ignored when open)
    }

    #[test]
    fn test_document_version_with_explicit_constructor() {
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".test { color: red; }".to_string();
        let version = DocumentVersion { major: 5, minor: 3 };
        
        let doc = UssDocument::new_with_document_version(uri, content, 1, version, true);
        
        assert_eq!(doc.document_version().major, 5);
        assert_eq!(doc.document_version().minor, 3);
        assert!(doc.is_open());
    }


}