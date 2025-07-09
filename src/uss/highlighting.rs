//! USS Syntax Highlighting Module
//!
//! Provides semantic token generation for USS files based on tree-sitter syntax trees.

use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use crate::uss::definitions::UssDefinitions;

/// USS semantic token provider
pub struct UssHighlighter {
    /// Semantic token legend for USS
    pub legend: SemanticTokensLegend,
}

impl UssHighlighter {
    /// Create a new USS highlighter with the semantic token legend
    pub fn new() -> Self {
        Self {
            legend: SemanticTokensLegend {
                token_types: vec![
                    SemanticTokenType::CLASS,        // 0 - .class-selector
                    SemanticTokenType::VARIABLE,     // 1 - #id-selector, CSS variables
                    SemanticTokenType::TYPE,         // 2 - tag_name (Button, Label)
                    SemanticTokenType::PROPERTY,     // 3 - property_name
                    SemanticTokenType::NUMBER,       // 4 - numeric values, colors
                    SemanticTokenType::STRING,       // 5 - string_value
                    SemanticTokenType::COMMENT,      // 6 - comments
                    SemanticTokenType::MODIFIER,     // 7 - pseudo-class selectors
                    SemanticTokenType::KEYWORD,      // 8 - at-rules (@import, @media)
                    SemanticTokenType::FUNCTION,     // 9 - function_name
                ],
                token_modifiers: vec![
                    SemanticTokenModifier::DECLARATION, // 0 - Unity-specific properties
                ],
            },
        }
    }
    
    /// Generate semantic tokens for a USS document
    pub fn generate_tokens(&self, tree: &Tree, content: &str) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        let root = tree.root_node();
        
        // Collect all tokens first
        let mut raw_tokens = Vec::new();
        Self::walk_node_for_tokens(&root, content, &mut raw_tokens);
        
        // Sort tokens by position
        raw_tokens.sort_by(|a, b| {
            a.line.cmp(&b.line).then(a.start_char.cmp(&b.start_char))
        });
        
        // Convert to delta-encoded semantic tokens
        let mut prev_line = 0;
        let mut prev_start = 0;
        
        for token in raw_tokens {
            let delta_line = token.line - prev_line;
            let delta_start = if delta_line == 0 {
                token.start_char - prev_start
            } else {
                token.start_char
            };
            
            tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.modifiers,
            });
            
            prev_line = token.line;
            prev_start = token.start_char;
        }
        
        tokens
    }
    
    /// Walk syntax tree nodes to collect semantic tokens
    fn walk_node_for_tokens(node: &Node, content: &str, tokens: &mut Vec<RawToken>) {
        let node_type = node.kind();
        
        // Skip certain structural nodes that don't need highlighting
        if matches!(node_type, "stylesheet" | "rule_set" | "block" | "selectors" | "declaration" | "arguments") {
            // Process children for structural nodes
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    Self::walk_node_for_tokens(&child, content, tokens);
                }
            }
            return;
        }
        
        // Map CSS node types to semantic token types
        let (token_type, modifiers) = match node_type {
            "class_selector" => (0, 0), // CLASS
            "id_selector" => (1, 0),    // VARIABLE
            "tag_name" => (2, 0),       // TYPE
            "property_name" => {
                // Check if it's a CSS custom property
                if let Ok(text) = node.utf8_text(content.as_bytes()) {
                    if text.starts_with("--") {
                        (1, 0) // VARIABLE (CSS custom property)
                    } else if text.starts_with("-unity-") {
                        (3, 1) // PROPERTY with DECLARATION modifier for Unity-specific
                    } else {
                        (3, 0) // PROPERTY
                    }
                } else {
                    (3, 0) // PROPERTY
                }
            },
            "plain_value" => {
                // Check if this plain_value is actually a color keyword
                if let Ok(text) = node.utf8_text(content.as_bytes()) {
                    let definitions = UssDefinitions::new();
                    if definitions.is_valid_color_keyword(text) {
                        (4, 0) // NUMBER (colors) - same as color_value
                    } else {
                        (4, 0) // NUMBER (regular values)
                    }
                } else {
                    (4, 0) // NUMBER
                }
            },
            "integer_value" | "float_value" => (4, 0), // NUMBER
            "string_value" => (5, 0),   // STRING
            "color_value" => (4, 0),    // NUMBER (colors)
            "comment" => (6, 0),        // COMMENT
            "pseudo_class_selector" => (7, 0), // MODIFIER
            "at_rule" => (8, 0),        // KEYWORD
            "import_statement" => {
                // Process children for import statements to highlight parts separately
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        // Check if this child is the @import keyword
                        if let Ok(text) = child.utf8_text(content.as_bytes()) {
                            if text == "@import" {
                                // Highlight @import as KEYWORD
                                let start = child.start_position();
                                let end = child.end_position();
                                tokens.push(RawToken {
                                    line: start.row as u32,
                                    start_char: start.column as u32,
                                    length: (end.column - start.column) as u32,
                                    token_type: 8, // KEYWORD
                                    modifiers: 0,
                                });
                            } else {
                                // Process other children normally (like string_value)
                                Self::walk_node_for_tokens(&child, content, tokens);
                            }
                        } else {
                            Self::walk_node_for_tokens(&child, content, tokens);
                        }
                    }
                }
                return;
            },
            "function_name" => (9, 0),  // FUNCTION
            "class_name" => (0, 0),     // CLASS (for .class-name)
            "id_name" => (1, 0),        // VARIABLE (for #id-name)
            _ => {
                // Process children for unhandled nodes
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        Self::walk_node_for_tokens(&child, content, tokens);
                    }
                }
                return;
            }
        };
        
        // Create token for this node
        let start = node.start_position();
        let end = node.end_position();
        
        tokens.push(RawToken {
            line: start.row as u32,
            start_char: start.column as u32,
            length: (end.column - start.column) as u32,
            token_type,
            modifiers,
        });
        
        // For some nodes, we might want to process children as well
        // (e.g., to handle nested structures)
        match node_type {
            "call_expression" => {
                // Process function name and arguments separately
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        Self::walk_node_for_tokens(&child, content, tokens);
                    }
                }
            }
            _ => {
                // For leaf nodes, don't process children to avoid duplication
            }
        }
    }
}

impl Default for UssHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Raw token before delta encoding
#[derive(Debug, Clone)]
struct RawToken {
    line: u32,
    start_char: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;
    
    #[test]
    fn test_basic_highlighting() {
        let mut parser = UssParser::new().expect("Failed to create parser");
        let content = ".my-class { color: red; }";
        let tree = parser.parse(content, None).expect("Failed to parse");
        
        let highlighter = UssHighlighter::new();
        let tokens = highlighter.generate_tokens(&tree, content);
        
        // Should have tokens for class_selector, property_name, and plain_value
        assert!(!tokens.is_empty());
    }
    
    #[test]
    fn test_unity_property_highlighting() {
        let mut parser = UssParser::new().expect("Failed to create parser");
        let content = "Button { -unity-font: resource(\"Arial\"); }";
        let tree = parser.parse(content, None).expect("Failed to parse");
        
        let highlighter = UssHighlighter::new();
        let tokens = highlighter.generate_tokens(&tree, content);
        
        // Should have tokens including Unity-specific property
        assert!(!tokens.is_empty());
        
        // Check that we have a PROPERTY token with DECLARATION modifier for -unity-font
        let has_unity_property = tokens.iter().any(|token| {
            token.token_type == 3 && token.token_modifiers_bitset == 1
        });
        assert!(has_unity_property, "Should highlight Unity-specific properties");
    }
    
    #[test]
    fn test_css_variable_highlighting() {
        let mut parser = UssParser::new().expect("Failed to create parser");
        let content = ":root { --color: #fff; } .test { color: var(--color); }";
        let tree = parser.parse(content, None).expect("Failed to parse");
        
        let highlighter = UssHighlighter::new();
        let tokens = highlighter.generate_tokens(&tree, content);
        
        // Should have tokens for CSS variables
        assert!(!tokens.is_empty());
    }
    
    #[test]
    fn test_color_keyword_highlighting() {
        let mut parser = UssParser::new().expect("Failed to create parser");
        let content = "Button { color: red; background-color: blue; border-color: aliceblue; }";
        let tree = parser.parse(content, None).expect("Failed to parse");
        
        let highlighter = UssHighlighter::new();
        let tokens = highlighter.generate_tokens(&tree, content);
        
        // Should have tokens for color keywords
        assert!(!tokens.is_empty());
        
        // Check that we have NUMBER tokens (type 4) for color keywords
        let has_color_tokens = tokens.iter().any(|token| token.token_type == 4);
        assert!(has_color_tokens, "Should highlight color keywords as NUMBER tokens");
    }
}