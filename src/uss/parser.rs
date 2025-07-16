//! USS Parser using tree-sitter-css
//!
//! Since USS syntax is nearly identical to CSS, we can use the existing
//! tree-sitter-css grammar directly.

use tree_sitter::{Parser, Tree};

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
    use crate::uss::constants::*;

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
        assert_eq!(root.kind(), NODE_STYLESHEET);
        assert_eq!(root.child_count(), 1);
        
        // Check the rule node
        let rule = root.child(0).unwrap();
        assert_eq!(rule.kind(), NODE_RULE_SET);
        assert_eq!(rule.child_count(), 2);
        
        // Check selectors
        let selectors = rule.child(0).unwrap();
        assert_eq!(selectors.kind(), NODE_SELECTORS);
        let class_selector = selectors.child(0).unwrap();
        assert_eq!(class_selector.kind(), NODE_CLASS_SELECTOR);
        
        // Check class name
        let class_name = class_selector.child(1).unwrap(); // Skip the '.' token
        assert_eq!(class_name.kind(), NODE_CLASS_NAME);
        assert_eq!(class_name.utf8_text(content.as_bytes()).unwrap(), "my-class");
        
        // Check declaration block
        let block = rule.child(1).unwrap();
        assert_eq!(block.kind(), NODE_BLOCK);
        
        // Find the declaration (skip braces)
        let declaration = block.child(1).unwrap(); // Skip opening brace
        assert_eq!(declaration.kind(), NODE_DECLARATION);
        
        // Check property name
        let property = declaration.child(0).unwrap();
        assert_eq!(property.kind(), NODE_PROPERTY_NAME);
        assert_eq!(property.utf8_text(content.as_bytes()).unwrap(), "color");
        
        // Check value (skip colon)
        let value = declaration.child(2).unwrap(); // Skip colon
        assert_eq!(value.kind(), NODE_PLAIN_VALUE);
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
        assert_eq!(rule.kind(), NODE_RULE_SET);
        
        // Check type selector
        let selectors = rule.child(0).unwrap();
        assert_eq!(selectors.kind(), NODE_SELECTORS);
        let type_selector = selectors.child(0).unwrap();
        assert_eq!(type_selector.kind(), NODE_TAG_NAME);
        assert_eq!(type_selector.utf8_text(content.as_bytes()).unwrap(), "Button");
        
        // Check Unity-specific property
        let block = rule.child(1).unwrap();
        assert_eq!(block.kind(), NODE_BLOCK);
        
        // Find the declaration (skip opening brace)
        let declaration = block.child(1).unwrap();
        assert_eq!(declaration.kind(), NODE_DECLARATION);
        
        // Check property name
        let property = declaration.child(0).unwrap();
        assert_eq!(property.kind(), NODE_PROPERTY_NAME);
        assert_eq!(property.utf8_text(content.as_bytes()).unwrap(), "-unity-font");
        
        // Check resource function call (skip colon)
        let value = declaration.child(2).unwrap();
        assert_eq!(value.kind(), NODE_CALL_EXPRESSION);
        
        // Check function name
        let function_name = value.child(0).unwrap();
        assert_eq!(function_name.kind(), NODE_FUNCTION_NAME);
        assert_eq!(function_name.utf8_text(content.as_bytes()).unwrap(), "resource");
        
        // Check arguments
        let arguments = value.child(1).unwrap();
        assert_eq!(arguments.kind(), NODE_ARGUMENTS);
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
        assert_eq!(root.kind(), NODE_STYLESHEET);
        let rule = root.child(0).unwrap();
        assert_eq!(rule.kind(), NODE_RULE_SET);
        
        // Check that we have selectors
        let selectors = rule.child(0).unwrap();
        assert_eq!(selectors.kind(), NODE_SELECTORS);
        
        // Verify we can find different selector types in the tree
        let mut found_tag = false;
        let mut found_class = false;
        let mut found_pseudo = false;
        let mut found_declaration = false;
        
        fn search_tree_for_selectors(node: tree_sitter::Node, content: &str, 
                                    found_tag: &mut bool, found_class: &mut bool, 
                                    found_pseudo: &mut bool, found_decl: &mut bool) {
            match node.kind() {
                NODE_TAG_NAME => {
                    if node.utf8_text(content.as_bytes()).unwrap() == "Button" {
                        *found_tag = true;
                    }
                }
                NODE_CLASS_SELECTOR => *found_class = true,
                NODE_PSEUDO_CLASS_SELECTOR => *found_pseudo = true,
                NODE_DECLARATION => *found_decl = true,
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