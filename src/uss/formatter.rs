//! USS Formatter Implementation
//!
//! Provides formatting capabilities for USS files using the malva CSS formatter.
//! Follows the formatting rules specified in USSFormatter.md.

use malva::{config::FormatOptions, format_text, Syntax};
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};

/// USS Formatter that handles formatting requests
pub struct UssFormatter {
    format_options: FormatOptions,
}

impl UssFormatter {
    /// Create a new USS formatter with default options
    pub fn new() -> Self {
        Self {
            format_options: FormatOptions::default(),
        }
    }

    /// Create a new USS formatter with custom options
    pub fn new_with_options(format_options: FormatOptions) -> Self {
        Self { format_options }
    }

    /// Format the entire document
    pub fn format_document(&self, content: &str, tree: &Tree) -> Result<Vec<TextEdit>, String> {
        // Check if there are any error nodes in the entire tree
        if self.has_error_nodes(tree.root_node()) {
            log::debug!("Document contains error nodes, skipping formatting");
            return Ok(Vec::new());
        }

        // Format the entire content
        match format_text(content, Syntax::Css, &self.format_options) {
            Ok(formatted) => {
                if formatted == content {
                    // No changes needed
                    Ok(Vec::new())
                } else {
                    // Return a single edit that replaces the entire document
                    Ok(vec![TextEdit {
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: self.get_document_end_position(content),
                        },
                        new_text: formatted,
                    }])
                }
            }
            Err(e) => {
                log::warn!("Failed to format USS document: {}", e);
                Err(format!("Formatting failed: {}", e))
            }
        }
    }

    /// Format a specific range in the document
    pub fn format_range(
        &self,
        content: &str,
        tree: &Tree,
        range: Range,
    ) -> Result<Vec<TextEdit>, String> {
        // Find the actual range that contains whole top-level nodes
        let actual_range = self.find_actual_format_range(content, tree, range)?;
        
        if actual_range.is_none() {
            log::debug!("No valid range found for formatting");
            return Ok(Vec::new());
        }
        
        let actual_range = actual_range.unwrap();
        
        // Extract the content for the actual range
        let range_content = self.extract_range_content(content, actual_range)?;
        
        // Check if there are error nodes in the range
        if self.has_error_nodes_in_range(tree.root_node(), actual_range, content) {
            log::debug!("Range contains error nodes, skipping formatting");
            return Ok(Vec::new());
        }

        // Format the range content
        match format_text(&range_content, Syntax::Css, &self.format_options) {
            Ok(formatted) => {
                if formatted == range_content {
                    // No changes needed
                    Ok(Vec::new())
                } else {
                    // Return an edit for the actual range
                    Ok(vec![TextEdit {
                        range: actual_range,
                        new_text: formatted,
                    }])
                }
            }
            Err(e) => {
                log::warn!("Failed to format USS range: {}", e);
                Err(format!("Range formatting failed: {}", e))
            }
        }
    }

    /// Find the actual range to format, ensuring it contains whole top-level nodes
    /// and doesn't start/end in the middle of lines with other content
    fn find_actual_format_range(
        &self,
        content: &str,
        tree: &Tree,
        requested_range: Range,
    ) -> Result<Option<Range>, String> {
        let lines: Vec<&str> = content.lines().collect();
        let root = tree.root_node();

        // Convert LSP positions to byte offsets
        let start_offset = self.position_to_offset(content, requested_range.start)?;
        let end_offset = self.position_to_offset(content, requested_range.end)?;

        // Find top-level nodes that are completely within the range
        let mut valid_nodes = Vec::new();
        let mut cursor = root.walk();
        
        for child in root.children(&mut cursor) {
            let node_start = child.start_byte();
            let node_end = child.end_byte();
            
            // Only include nodes that are completely within the requested range
            if node_start >= start_offset && node_end <= end_offset {
                valid_nodes.push(child);
            }
        }

        if valid_nodes.is_empty() {
            return Ok(None);
        }

        // Get the range from first to last valid node
        let first_node = valid_nodes.first().unwrap();
        let last_node = valid_nodes.last().unwrap();
        
        let range_start_pos = self.offset_to_position(content, first_node.start_byte())?;
        let range_end_pos = self.offset_to_position(content, last_node.end_byte())?;
        
        // Check if the range starts/ends cleanly (not in the middle of lines with other content)
        let start_line_idx = range_start_pos.line as usize;
        let end_line_idx = range_end_pos.line as usize;
        
        if start_line_idx < lines.len() {
            let start_line = lines[start_line_idx];
            let before_start = &start_line[..range_start_pos.character as usize];
            if !before_start.trim().is_empty() {
                // There's content before our range on the same line
                return Ok(None);
            }
        }
        
        if end_line_idx < lines.len() {
            let end_line = lines[end_line_idx];
            let after_end = &end_line[range_end_pos.character as usize..];
            if !after_end.trim().is_empty() {
                // There's content after our range on the same line
                return Ok(None);
            }
        }

        Ok(Some(Range {
            start: range_start_pos,
            end: range_end_pos,
        }))
    }

    /// Check if a node or any of its descendants contains error nodes
    fn has_error_nodes(&self, node: Node) -> bool {
        if node.is_error() || node.kind() == "ERROR" {
            return true;
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self.has_error_nodes(child) {
                return true;
            }
        }
        
        false
    }

    /// Check if there are error nodes within a specific range
    fn has_error_nodes_in_range(&self, node: Node, range: Range, content: &str) -> bool {
        let start_offset = self.position_to_offset(content, range.start).unwrap_or(0);
        let end_offset = self.position_to_offset(content, range.end).unwrap_or(content.len());
        
        self.has_error_nodes_in_byte_range(node, start_offset, end_offset)
    }
    
    /// Check if there are error nodes within a byte range
    fn has_error_nodes_in_byte_range(&self, node: Node, start_byte: usize, end_byte: usize) -> bool {
        // Check if this node overlaps with our range and is an error
        let node_start = node.start_byte();
        let node_end = node.end_byte();
        
        if node_start < end_byte && node_end > start_byte {
            // Node overlaps with our range
            if node.is_error() || node.kind() == "ERROR" {
                return true;
            }
        }
        
        // Check children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self.has_error_nodes_in_byte_range(child, start_byte, end_byte) {
                return true;
            }
        }
        
        false
    }

    /// Extract content for a specific range
    fn extract_range_content(&self, content: &str, range: Range) -> Result<String, String> {
        let start_offset = self.position_to_offset(content, range.start)?;
        let end_offset = self.position_to_offset(content, range.end)?;
        
        if start_offset > content.len() || end_offset > content.len() || start_offset > end_offset {
            return Err("Invalid range offsets".to_string());
        }
        
        Ok(content[start_offset..end_offset].to_string())
    }

    /// Convert LSP position to byte offset
    fn position_to_offset(&self, content: &str, position: Position) -> Result<usize, String> {
        let lines: Vec<&str> = content.lines().collect();
        
        if position.line as usize >= lines.len() {
            return Err(format!("Line {} is out of bounds", position.line));
        }
        
        let mut offset = 0;
        
        // Add bytes for all previous lines (including newline characters)
        for i in 0..position.line as usize {
            offset += lines[i].len() + 1; // +1 for newline character
        }
        
        // Add bytes for characters in the current line
        let current_line = lines[position.line as usize];
        if position.character as usize > current_line.len() {
            return Err(format!("Character {} is out of bounds for line {}", position.character, position.line));
        }
        
        offset += position.character as usize;
        
        Ok(offset)
    }

    /// Convert byte offset to LSP position
    fn offset_to_position(&self, content: &str, offset: usize) -> Result<Position, String> {
        if offset > content.len() {
            return Err(format!("Offset {} is out of bounds", offset));
        }
        
        let mut current_offset = 0;
        let mut line = 0;
        
        for line_content in content.lines() {
            let line_end = current_offset + line_content.len();
            
            if offset <= line_end {
                // The offset is within this line
                let character = offset - current_offset;
                return Ok(Position {
                    line: line as u32,
                    character: character as u32,
                });
            }
            
            current_offset = line_end + 1; // +1 for newline character
            line += 1;
        }
        
        // If we reach here, the offset is at the very end of the file
        Ok(Position {
            line: line as u32,
            character: 0,
        })
    }

    /// Get the end position of the document
    fn get_document_end_position(&self, content: &str) -> Position {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            Position { line: 0, character: 0 }
        } else {
            Position {
                line: (lines.len() - 1) as u32,
                character: lines.last().unwrap().len() as u32,
            }
        }
    }
}

impl Default for UssFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::{Parser, Language};

    fn create_parser() -> Parser {
        let mut parser = Parser::new();
        let language = tree_sitter_css::language();
        parser.set_language(language).expect("Error loading CSS language");
        parser
    }

    #[test]
    fn test_format_simple_css() {
        let formatter = UssFormatter::new();
        let mut parser = create_parser();
        
        let content = ".test{color:red;background:blue;}";
        let tree = parser.parse(content, None).unwrap();
        
        let result = formatter.format_document(content, &tree);
        assert!(result.is_ok());
        
        let edits = result.unwrap();
        assert!(!edits.is_empty());
    }

    #[test]
    fn test_skip_formatting_with_errors() {
        let formatter = UssFormatter::new();
        let mut parser = create_parser();
        
        let content = ".test{color:red";
        let tree = parser.parse(content, None).unwrap();
        
        let result = formatter.format_document(content, &tree);
        assert!(result.is_ok());
        
        let edits = result.unwrap();
        assert!(edits.is_empty()); // Should skip formatting due to errors
    }

    #[test]
    fn test_position_to_offset() {
        let formatter = UssFormatter::new();
        let content = "line1\nline2\nline3";
        
        let offset = formatter.position_to_offset(content, Position { line: 1, character: 2 }).unwrap();
        assert_eq!(offset, 8); // "line1\nli" = 8 characters
    }

    #[test]
    fn test_offset_to_position() {
        let formatter = UssFormatter::new();
        let content = "line1\nline2\nline3";
        
        let position = formatter.offset_to_position(content, 8).unwrap();
        assert_eq!(position, Position { line: 1, character: 2 });
    }
}