//! Tree dumper utility to analyze USS syntax tree structure
//!
//! This utility parses a USS file and outputs the complete syntax tree
//! to help understand the tree structure for language server development.

use std::env;
use std::fs;
use tree_sitter::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <uss_file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_css::LANGUAGE.into()).expect("Error loading CSS language");

    let tree = parser.parse(&content, None).expect("Error parsing USS file");
    let root = tree.root_node();

    println!("=== USS Syntax Tree for {} ===", file_path);
    println!("File size: {} bytes", content.len());
    println!("Tree has errors: {}", root.has_error());
    println!();
    
    print_tree(root, &content, 0);
    
    println!();
    println!("=== Tree Statistics ===");
    let stats = collect_node_stats(root);
    for (node_type, count) in stats {
        println!("{}: {}", node_type, count);
    }
}

fn print_tree(node: tree_sitter::Node, content: &str, depth: usize) {
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

fn collect_node_stats(node: tree_sitter::Node) -> std::collections::BTreeMap<String, usize> {
    let mut stats = std::collections::BTreeMap::new();
    collect_node_stats_recursive(node, &mut stats);
    stats
}

fn collect_node_stats_recursive(node: tree_sitter::Node, stats: &mut std::collections::BTreeMap<String, usize>) {
    *stats.entry(node.kind().to_string()).or_insert(0) += 1;
    
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_node_stats_recursive(child, stats);
        }
    }
}