//! USS Formatter Implementation
//!
//! Provides formatting capabilities for USS files using the malva CSS formatter.
//! Follows the formatting rules specified in USSFormatter.md.

use malva::{config::FormatOptions, format_text, Syntax};
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use crate::language::tree_utils::{byte_to_position, position_to_byte_offset, node_to_range, has_error_nodes};
use crate::uss::constants::NODE_ERROR;

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

    /// Format the entire document
    pub fn format_document(&self, content: &str, tree: &Tree) -> Result<Vec<TextEdit>, String> {
        // Use format_range with the full document range
        let full_range = Range {
            start: Position { line: 0, character: 0 },
            end: self.get_document_end_position(content, tree),
        };
        
        self.format_range(content, tree, full_range)
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

        // Find the actual first and last nodes that start/end cleanly
        let mut actual_first_idx = 0;
        let mut actual_last_idx = valid_nodes.len() - 1;
        
        // Find first node that starts cleanly (no previous sibling on same line)
        while actual_first_idx < valid_nodes.len() {
            let current_node = valid_nodes[actual_first_idx];
            let current_range = node_to_range(current_node, content);
            
            // Check if there's a previous sibling on the same line
            let mut has_prev_sibling_on_same_line = false;
            if let Some(prev_sibling) = current_node.prev_sibling() {
                let prev_range = node_to_range(prev_sibling, content);
                if prev_range.end.line == current_range.start.line {
                    has_prev_sibling_on_same_line = true;
                }
            }
            
            if !has_prev_sibling_on_same_line {
                break;
            }
            actual_first_idx += 1;
        }
        
        // Find last node that ends cleanly (no next sibling on same line)
        while actual_last_idx > actual_first_idx {
            let current_node = valid_nodes[actual_last_idx];
            let current_range = node_to_range(current_node, content);
            
            // Check if there's a next sibling on the same line
            let mut has_next_sibling_on_same_line = false;
            if let Some(next_sibling) = current_node.next_sibling() {
                let next_range = node_to_range(next_sibling, content);
                if next_range.start.line == current_range.end.line {
                    has_next_sibling_on_same_line = true;
                }
            }
            
            if !has_next_sibling_on_same_line {
                break;
            }
            actual_last_idx -= 1;
        }
        
        // If we couldn't find any clean nodes, return None
        if actual_first_idx >= valid_nodes.len() || actual_last_idx < actual_first_idx {
            return Ok(None);
        }
        
        let first_node = valid_nodes[actual_first_idx];
        let last_node = valid_nodes[actual_last_idx];
        
        let first_range = node_to_range(first_node, content);
        let last_range = node_to_range(last_node, content);
        
        let range_start_pos = first_range.start;
        let range_end_pos = last_range.end;

        Ok(Some(Range {
            start: range_start_pos,
            end: range_end_pos,
        }))
    }

    /// Check if there are error nodes within a specific range
    fn has_error_nodes_in_range(&self, node: Node, range: Range, content: &str) -> bool {
        let start_offset = self.position_to_offset(content, range.start).unwrap_or(0);
        let end_offset = self.position_to_offset(content, range.end).unwrap_or(content.len());
        
        // Find top-level nodes that overlap with the range and check each one
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let node_start = child.start_byte();
            let node_end = child.end_byte();
            
            // Check if this top-level node overlaps with our range
            if node_start < end_offset && node_end > start_offset {
                if has_error_nodes(child) {
                    return true;
                }
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
        position_to_byte_offset(content, position)
            .ok_or_else(|| format!("Invalid position: line {}, character {}", position.line, position.character))
    }

    /// Convert byte offset to LSP position
    fn offset_to_position(&self, content: &str, offset: usize) -> Result<Position, String> {
        if offset > content.len() {
            return Err(format!("Offset {} is out of bounds", offset));
        }
        Ok(byte_to_position(offset, content))
    }

    /// Get the end position of the document using the tree's end byte position
    fn get_document_end_position(&self, content: &str, tree: &Tree) -> Position {
        let root = tree.root_node();
        let end_byte = root.end_byte();
        byte_to_position(end_byte, content)
    }
}

impl Default for UssFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "formatter_tests.rs"]
mod tests;