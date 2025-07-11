//! USS Parser using tree-sitter-css
//!
//! Since USS syntax is nearly identical to CSS, we can use the existing
//! tree-sitter-css grammar directly.

use tree_sitter::{Language, Parser, Tree};

/// USS parser wrapper around tree-sitter-css
pub struct UssParser {
    parser: Parser,
}

impl UssParser {
    /// Create a new USS parser
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut parser = Parser::new();
        let language = tree_sitter_css::language();
        parser.set_language(language)?;
        
        Ok(Self { parser })
    }
    
    /// Parse USS content and return the syntax tree
    pub fn parse(&mut self, content: &str, old_tree: Option<&Tree>) -> Option<Tree> {
        self.parser.parse(content, old_tree)
    }
}

impl Default for UssParser {
    fn default() -> Self {
        Self::new().expect("Failed to create USS parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_creation() {
        let parser = UssParser::new();
        assert!(parser.is_ok());
    }
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = UssParser::new().unwrap();
        let content = ".my-class { color: red; }";
        let tree = parser.parse(content, None);
        assert!(tree.is_some());
        
        let tree = tree.unwrap();
        let root = tree.root_node();
        assert!(!root.has_error());
        
        // Verify the tree structure
        assert_eq!(root.kind(), "stylesheet");
        assert_eq!(root.child_count(), 1);
        
        // Check the rule node
        let rule = root.child(0).unwrap();
        assert_eq!(rule.kind(), "rule_set");
        assert_eq!(rule.child_count(), 2);
        
        // Check selectors
        let selectors = rule.child(0).unwrap();
        assert_eq!(selectors.kind(), "selectors");
        let class_selector = selectors.child(0).unwrap();
        assert_eq!(class_selector.kind(), "class_selector");
        
        // Check class name
        let class_name = class_selector.child(1).unwrap(); // Skip the '.' token
        assert_eq!(class_name.kind(), "class_name");
        assert_eq!(class_name.utf8_text(content.as_bytes()).unwrap(), "my-class");
        
        // Check declaration block
        let block = rule.child(1).unwrap();
        assert_eq!(block.kind(), "block");
        
        // Find the declaration (skip braces)
        let declaration = block.child(1).unwrap(); // Skip opening brace
        assert_eq!(declaration.kind(), "declaration");
        
        // Check property name
        let property = declaration.child(0).unwrap();
        assert_eq!(property.kind(), "property_name");
        assert_eq!(property.utf8_text(content.as_bytes()).unwrap(), "color");
        
        // Check value (skip colon)
        let value = declaration.child(2).unwrap(); // Skip colon
        assert_eq!(value.kind(), "plain_value");
        assert_eq!(value.utf8_text(content.as_bytes()).unwrap(), "red");
    }
    
    #[test]
    fn test_unity_specific_properties() {
        let mut parser = UssParser::new().unwrap();
        let content = "Button { -unity-font: resource(\"Arial\"); }";
        let tree = parser.parse(content, None);
        assert!(tree.is_some());
        
        let tree = tree.unwrap();
        let root = tree.root_node();
        assert!(!root.has_error());
        
        // Verify the tree structure for Unity-specific properties
        let rule = root.child(0).unwrap();
        assert_eq!(rule.kind(), "rule_set");
        
        // Check type selector
        let selectors = rule.child(0).unwrap();
        assert_eq!(selectors.kind(), "selectors");
        let type_selector = selectors.child(0).unwrap();
        assert_eq!(type_selector.kind(), "tag_name");
        assert_eq!(type_selector.utf8_text(content.as_bytes()).unwrap(), "Button");
        
        // Check Unity-specific property
        let block = rule.child(1).unwrap();
        assert_eq!(block.kind(), "block");
        
        // Find the declaration (skip opening brace)
        let declaration = block.child(1).unwrap();
        assert_eq!(declaration.kind(), "declaration");
        
        // Check property name
        let property = declaration.child(0).unwrap();
        assert_eq!(property.kind(), "property_name");
        assert_eq!(property.utf8_text(content.as_bytes()).unwrap(), "-unity-font");
        
        // Check resource function call (skip colon)
        let value = declaration.child(2).unwrap();
        assert_eq!(value.kind(), "call_expression");
        
        // Check function name
        let function_name = value.child(0).unwrap();
        assert_eq!(function_name.kind(), "function_name");
        assert_eq!(function_name.utf8_text(content.as_bytes()).unwrap(), "resource");
        
        // Check arguments
        let arguments = value.child(1).unwrap();
        assert_eq!(arguments.kind(), "arguments");
    }
    
    #[test]
    fn test_complex_selector_parsing() {
        let mut parser = UssParser::new().unwrap();
        let content = "Button.primary:hover { background-color: #007acc; }";
        let tree = parser.parse(content, None);
        assert!(tree.is_some());
        
        let tree = tree.unwrap();
        let root = tree.root_node();
        assert!(!root.has_error());
        
        // Verify basic structure
        assert_eq!(root.kind(), "stylesheet");
        let rule = root.child(0).unwrap();
        assert_eq!(rule.kind(), "rule_set");
        
        // Check that we have selectors
        let selectors = rule.child(0).unwrap();
        assert_eq!(selectors.kind(), "selectors");
        
        // Verify we can find different selector types in the tree
        let mut found_tag = false;
        let mut found_class = false;
        let mut found_pseudo = false;
        let mut found_declaration = false;
        
        fn search_tree_for_selectors(node: tree_sitter::Node, content: &str, 
                                    found_tag: &mut bool, found_class: &mut bool, 
                                    found_pseudo: &mut bool, found_decl: &mut bool) {
            match node.kind() {
                "tag_name" => {
                    if node.utf8_text(content.as_bytes()).unwrap() == "Button" {
                        *found_tag = true;
                    }
                }
                "class_selector" => *found_class = true,
                "pseudo_class_selector" => *found_pseudo = true,
                "declaration" => *found_decl = true,
                _ => {}
            }
            
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    search_tree_for_selectors(child, content, found_tag, found_class, found_pseudo, found_decl);
                }
            }
        }
        
        search_tree_for_selectors(root, content, &mut found_tag, &mut found_class, &mut found_pseudo, &mut found_declaration);
        
        assert!(found_tag, "Should find tag name 'Button'");
        assert!(found_class, "Should find class selector");
        assert!(found_pseudo, "Should find pseudo-class selector");
        assert!(found_declaration, "Should find declaration");
    }
}