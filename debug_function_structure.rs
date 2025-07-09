use crate::uss::parser::UssParser;
use crate::uss::tree_printer::print_tree_to_stdout;

fn main() {
    let mut parser = UssParser::new().unwrap();
    
    // Test rgb function structure
    let content = "rgb(255, 128, 0)";
    println!("=== Testing: {} ===", content);
    if let Some(tree) = parser.parse(content, None) {
        print_tree_to_stdout(tree.root_node(), content);
    }
    
    println!("\n\n");
    
    // Test rgba function structure
    let content2 = "rgba(255, 128, 0, 0.5)";
    println!("=== Testing: {} ===", content2);
    if let Some(tree2) = parser.parse(content2, None) {
        print_tree_to_stdout(tree2.root_node(), content2);
    }
}