//! Tree printer utility for USS syntax tree debugging
//!
//! This module provides functionality to print tree-sitter syntax trees
//! in a readable format for debugging and development purposes.

use std::collections::BTreeMap;
use tree_sitter::Node;

/// Print a complete syntax tree to stdout
pub fn print_tree_to_stdout(node: Node, content: &str) {
    println!("=== USS Syntax Tree ===");
    println!("Content size: {} bytes", content.len());
    println!("Tree has errors: {}", node.has_error());
    println!();
    
    print_tree(node, content, 0);
    
    println!();
    println!("=== Tree Statistics ===");
    let stats = collect_node_stats(node);
    for (node_type, count) in stats {
        println!("{}: {}", node_type, count);
    }
}

/// Print a syntax tree node and its children recursively
pub fn print_tree(node: Node, content: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let text = node.utf8_text(content.as_bytes()).unwrap_or("<invalid>");
    
    // Truncate very long text for readability
    let display_text = if text.len() > 50 {
        format!("{}...", &text[..47])
    } else {
        text.to_string()
    };
    
    // Escape newlines and tabs for better display
    let display_text = display_text
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r");
    
    println!("{}{}[{}:{}] '{}'", 
        indent, 
        node.kind(), 
        node.start_position().row + 1,
        node.start_position().column + 1,
        display_text
    );
    
    // Print field names for named children
    let mut field_names = Vec::new();
    for i in 0..node.child_count() {
        if let Some(field_name) = node.field_name_for_child(i as u32) {
            field_names.push((i, field_name));
        }
    }
    
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            // Show field name if available
            if let Some((_, field_name)) = field_names.iter().find(|(idx, _)| *idx == i) {
                println!("{}  [field: {}]", "  ".repeat(depth), field_name);
            }
            print_tree(child, content, depth + 1);
        }
    }
}

/// Collect statistics about node types in the tree
pub fn collect_node_stats(node: Node) -> BTreeMap<String, usize> {
    let mut stats = BTreeMap::new();
    collect_node_stats_recursive(node, &mut stats);
    stats
}

fn collect_node_stats_recursive(node: Node, stats: &mut BTreeMap<String, usize>) {
    *stats.entry(node.kind().to_string()).or_insert(0) += 1;
    
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_node_stats_recursive(child, stats);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::{Language, Parser};

    #[test]
    fn test_tree_printing() {
        let mut parser = Parser::new();
        let language = tree_sitter_css::language();
        parser.set_language(language).expect("Error loading CSS language");

        let content = ".test { color: red; }";
        let tree = parser.parse(content, None).expect("Error parsing USS");
        let root = tree.root_node();

        // Test that printing doesn't panic
        print_tree(root, content, 0);
        
        // Test statistics collection
        let stats = collect_node_stats(root);
        assert!(!stats.is_empty());
        assert!(stats.contains_key("stylesheet"));
    }
}