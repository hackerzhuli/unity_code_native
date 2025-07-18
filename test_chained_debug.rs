use std::path::Path;
use tree_sitter::{Parser, Point};
use tower_lsp::lsp_types::Position;

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_css::LANGUAGE.into())
        .expect("Error loading CSS language");
    
    // Test chained selector without spaces
    let content = ".class1.class2 { color: red; }";
    let tree = parser.parse(content, None).expect("Error parsing USS");
    
    // Test position in the middle of 'class2'
    let position = Position::new(0, 12); // Should be in 'class2'
    let point = Point::new(position.line as usize, position.character as usize);
    
    if let Some(node) = tree.root_node().descendant_for_point_range(point, point) {
        println!("Found node at position {}: kind='{}', text='{}'", 
                 position.character,
                 node.kind(), 
                 node.utf8_text(content.as_bytes()).unwrap_or("<invalid>"));
        
        // Walk up the tree to see the hierarchy
        let mut current = node;
        let mut level = 0;
        loop {
            println!("  Level {}: kind='{}', text='{}'", 
                     level,
                     current.kind(), 
                     current.utf8_text(content.as_bytes()).unwrap_or("<invalid>"));
            
            if let Some(parent) = current.parent() {
                current = parent;
                level += 1;
            } else {
                break;
            }
        }
    } else {
        println!("No node found at position {}", position.character);
    }
    
    // Print the full tree structure
    println!("\nFull tree structure:");
    print_tree(tree.root_node(), content, 0);
}

fn print_tree(node: tree_sitter::Node, content: &str, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let text = node.utf8_text(content.as_bytes()).unwrap_or("<invalid>");
    println!("{}{}[{}:{}] '{}'", 
             indent_str, 
             node.kind(), 
             node.start_position().row, 
             node.start_position().column, 
             text);
    
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, content, indent + 1);
    }
}