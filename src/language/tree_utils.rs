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