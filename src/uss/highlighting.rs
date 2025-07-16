//! USS Syntax Highlighting Module
//!
//! Provides semantic token generation for USS files based on tree-sitter syntax trees.

use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use crate::uss::definitions::UssDefinitions;
use crate::uss::constants::*;

/// USS semantic token provider
pub struct UssHighlighter {
    /// Semantic token legend for USS
    pub legend: SemanticTokensLegend,
    /// USS language definitions for validation
    definitions: UssDefinitions,
}

impl UssHighlighter {
    /// Create a new USS highlighter with the semantic token legend
    pub fn new() -> Self {
        Self {
            definitions: UssDefinitions::new(),
            legend: SemanticTokensLegend {
                token_types: vec![
                    SemanticTokenType::NAMESPACE,    // 0 - .class-selector
                    SemanticTokenType::VARIABLE,     // 1 - #id-selector, CSS variables
                    SemanticTokenType::CLASS,        // 2 - tag_name (Button, Label)
                    SemanticTokenType::PROPERTY,     // 3 - property_name
                    SemanticTokenType::NUMBER,       // 4 - numeric values, colors
                    SemanticTokenType::STRING,       // 5 - string_value
                    SemanticTokenType::COMMENT,      // 6 - comments
                    SemanticTokenType::PARAMETER,    // 7 - pseudo-class selectors
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
        self.walk_node_for_tokens(&root, content, &mut raw_tokens);
        
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
    fn walk_node_for_tokens(&self, node: &Node, content: &str, tokens: &mut Vec<RawToken>) {
        let node_type = node.kind();
        
        // Skip certain structural nodes that don't need highlighting
        if matches!(node_type, NODE_STYLESHEET | NODE_RULE_SET | NODE_BLOCK | NODE_SELECTORS | NODE_DECLARATION | NODE_ARGUMENTS) {
            // Process children for structural nodes
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    self.walk_node_for_tokens(&child, content, tokens);
                }
            }
            return;
        }
        
        // Map CSS node types to semantic token types
        let (token_type, modifiers) = match node_type {
            NODE_CLASS_SELECTOR => (0, 0), // CLASS
            NODE_ID_SELECTOR => (1, 0),    // VARIABLE
            NODE_TAG_NAME => (2, 0),       // TYPE
            NODE_PROPERTY_NAME => {
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
            NODE_PLAIN_VALUE => {
                // Check if this plain_value is actually a color keyword
                if let Ok(text) = node.utf8_text(content.as_bytes()) {
                    if self.definitions.is_valid_color_keyword(text) {
                        (4, 0) // NUMBER (colors) - same as color_value
                    } else {
                        (4, 0) // NUMBER (regular values)
                    }
                } else {
                    (4, 0) // NUMBER
                }
            },
            NODE_INTEGER_VALUE | NODE_FLOAT_VALUE => (4, 0), // NUMBER
            NODE_STRING_VALUE => (5, 0),   // STRING
            NODE_COLOR_VALUE => (4, 0),    // NUMBER (colors)
            NODE_COMMENT => (6, 0),        // COMMENT
            //NODE_PSEUDO_CLASS_SELECTOR => (7, 0), // MODIFIER
            NODE_AT_RULE => (8, 0),        // KEYWORD
            NODE_IMPORT_STATEMENT => {
                // Process children for import statements to highlight parts separately
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        // Check if this child is the @import keyword
                        if let Ok(text) = child.utf8_text(content.as_bytes()) {
                            if text == NODE_IMPORT {
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
                                self.walk_node_for_tokens(&child, content, tokens);
                            }
                        } else {
                            self.walk_node_for_tokens(&child, content, tokens);
                        }
                    }
                }
                return;
            },
            NODE_FUNCTION_NAME => (9, 0),  // FUNCTION
            NODE_CLASS_NAME =>
            {
                if let Some(parent) = node.parent() {
                    if parent.kind() == NODE_PSEUDO_CLASS_SELECTOR {
                        // This is a pseudo-class
                        (7, 0)
                    }else{
                        (0, 0)
                    }
                }else{
                    (0, 0)
                }               
            }
            NODE_ID_NAME => (1, 0),        // VARIABLE (for #id-name)
            _ => {
                // Process children for unhandled nodes
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.walk_node_for_tokens(&child, content, tokens);
                    }
                }
                return;
            }
        };
        
        // Create token for this node
        let start = node.start_position();
        let end = node.end_position();
        
        // Calculate proper length for multiline tokens
        let length = if start.row == end.row {
            // Single line token - use column difference
            (end.column - start.column) as u32
        } else {
            // Multiline token - calculate actual byte length
            let start_byte = node.start_byte();
            let end_byte = node.end_byte();
            (end_byte - start_byte) as u32
        };
        
        tokens.push(RawToken {
            line: start.row as u32,
            start_char: start.column as u32,
            length,
            token_type,
            modifiers,
        });
        
        // For some nodes, we might want to process children as well
        // (e.g., to handle nested structures)
        match node_type {
            NODE_CALL_EXPRESSION => {
                // Process function name and arguments separately
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.walk_node_for_tokens(&child, content, tokens);
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
    
    #[test]
    fn test_multiline_comment_highlighting() {
        let mut parser = UssParser::new().expect("Failed to create parser");
        let content = "/* This is a single line comment */\n.test {\n    color: red;\n}\n\n/* This is a\n   multiline comment\n   spanning multiple lines */\n.another {\n    background-color: blue;\n}";
        let tree = parser.parse(content, None).expect("Failed to parse");
        
        let highlighter = UssHighlighter::new();
        let tokens = highlighter.generate_tokens(&tree, content);
        
        // Check that we have comment tokens
        let comment_tokens: Vec<_> = tokens.iter().filter(|token| token.token_type == 6).collect();
        
        // Should have exactly 2 comment tokens (single line and multiline)
        assert_eq!(comment_tokens.len(), 2, "Should have exactly 2 comment tokens");
        
        // First comment (single line) should have length 35
        assert_eq!(comment_tokens[0].length, 35, "Single line comment should have correct length");
        
        // Second comment (multiline) should have length 63 (actual byte length)
        assert_eq!(comment_tokens[1].length, 63, "Multiline comment should have correct byte length");
    }
}