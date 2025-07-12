use tower_lsp::lsp_types::{Position, Range};
use tree_sitter::Node;

/// Convert tree-sitter node to LSP range
pub(crate) fn node_to_range(node: Node, content: &str) -> Range {
    let start_byte = node.start_byte();
    let end_byte = node.end_byte();

    let start_position = byte_to_position(start_byte, content);
    let end_position = byte_to_position(end_byte, content);

    Range {
        start: start_position,
        end: end_position,
    }
}

/// Convert byte offset to LSP position
pub(crate) fn byte_to_position(byte_offset: usize, content: &str) -> Position {
    let mut line = 0;
    let mut character = 0;

    for (i, ch) in content.char_indices() {
        if i >= byte_offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }

    Position {
        line: line as u32,
        character: character as u32,
    }
}

/// Find the first node of a specific type in the syntax tree
/// Performs a depth-first search to locate a node with the target type
pub fn find_node_by_type<'a>(node: Node<'a>, target_type: &str) -> Option<Node<'a>> {
    if node.kind() == target_type {
        return Some(node);
    }
    
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if let Some(found) = find_node_by_type(child, target_type) {
                return Some(found);
            }
        }
    }
    None
}

/// Converts LSP position to byte offset
pub fn position_to_byte_offset(source: &str, position: Position) -> Option<usize> {
    let mut line = 0;
    let mut col = 0;
    
    for (i, ch) in source.char_indices() {
        if line == position.line as usize && col == position.character as usize {
            return Some(i);
        }
        
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    
    None
}

/// Finds a child node of a specific type at the given position
/// This is a general function that can find any node type at a position
/// by walking up the tree from the deepest node at that position
pub fn find_node_of_type_at_position<'a>(
    node: Node<'a>,
    source: &str,
    position: Position,
    target_type: &str,
) -> Option<Node<'a>> {
    let byte_offset = position_to_byte_offset(source, position)?;
    
    // Find the deepest node at this position
    let mut current = node.descendant_for_byte_range(byte_offset, byte_offset)?;
    
    // Walk up the tree to find a node of one of the target types
    loop {
        if target_type == current.kind() {
            return Some(current);
        }
        
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            return None;
        }
    }
}