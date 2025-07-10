//! USS Document
//!
//! Represents a single USS document with its content, syntax tree, and version.
use std::collections::HashMap;
use tower_lsp::lsp_types::{Diagnostic, Position, Range, TextDocumentContentChangeEvent, Url};
use tree_sitter::{InputEdit, Point, Tree};

use crate::uss::parser::UssParser;
use crate::uss::variable_resolver::{VariableResolver, VariableDefinition, VariableResolutionStatus};



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
    /// Variable resolver for CSS custom properties
    variable_resolver: VariableResolver,
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
            variable_resolver: VariableResolver::new(),
        }
    }
    
    /// Parse the document content and store the syntax tree
    pub fn parse(&mut self, parser: &mut UssParser) {
        self.tree = parser.parse(&self.content, None);
        // Invalidate diagnostics when content is parsed
        self.invalidate_diagnostics();
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
        self.version = new_version;
        
        // Invalidate diagnostics when content changes
        self.invalidate_diagnostics();
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

    /// Get all variables defined in this document
    pub fn get_variables(&self) -> &HashMap<String, VariableDefinition> {
        self.variable_resolver.get_variables()
    }

    /// Get a specific variable by name
    pub fn get_variable(&self, name: &str) -> Option<&VariableDefinition> {
        self.variable_resolver.get_variable(name)
    }

    /// Check if variables have been resolved
    pub fn are_variables_resolved(&self) -> bool {
        self.variable_resolver.are_variables_resolved()
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
        assert_eq!(doc.version, 1);
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
        assert_eq!(doc.version, 2);
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
    fn test_diagnostics_caching() {
        let mut doc = create_test_document();
        
        // Initially no cached diagnostics
        assert!(doc.get_cached_diagnostics().is_none());
        
        // Set some diagnostics
          let diagnostics = vec![];
          doc.cache_diagnostics(diagnostics);
        
        // Should now have cached diagnostics
        assert!(doc.get_cached_diagnostics().is_some());
        
        // Invalidate diagnostics
        doc.invalidate_diagnostics();
        assert!(doc.get_cached_diagnostics().is_none());
    }


}