//! USS Color Provider Module
//!
//! Provides color information for USS files to enable color decorations in VS Code.

use tower_lsp::lsp_types::{ColorInformation, ColorPresentation, Position, Range, TextEdit, Color as LspColor};
use tree_sitter::{Node, Tree};
use crate::uss::definitions::UssDefinitions;
use crate::uss::value::UssValue;

/// USS color information provider
pub struct UssColorProvider {
    definitions: UssDefinitions,
}

impl UssColorProvider {
    /// Create a new USS color provider
    pub fn new() -> Self {
        Self {
            definitions: UssDefinitions::new(),
        }
    }
    
    /// Extract color information from a USS document
    pub fn provide_document_colors(&self, tree: &Tree, content: &str) -> Vec<ColorInformation> {
        let mut colors = Vec::new();
        let root = tree.root_node();
        
        self.walk_node_for_colors(&root, content, &mut colors);
        
        colors
    }
    
    /// Recursively walk the syntax tree to find color values
    fn walk_node_for_colors(&self, node: &Node, content: &str, colors: &mut Vec<ColorInformation>) {
        let node_type = node.kind();
        
        match node_type {
            "color_value" | "plain_value" | "call_expression" => {
                if let Some(color_info) = self.extract_color_from_uss_value(node, content) {
                    colors.push(color_info);
                }
            }
            _ => {
                // Recursively process child nodes
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.walk_node_for_colors(&child, content, colors);
                    }
                }
            }
        }
    }
    
    /// Extract color information using UssValue parsing
    fn extract_color_from_uss_value(&self, node: &Node, content: &str) -> Option<ColorInformation> {
        // Try to parse the node as a UssValue
        let uss_value = UssValue::from_node(*node, content, &self.definitions, None).ok()?;
        
        // Convert UssValue to ColorInformation if it's a color
        match uss_value {
            UssValue::Color(uss_color) => {
                let range = self.node_to_range(node, content)?;
                Some(ColorInformation {
                    range,
                    color: LspColor {
                        red: uss_color.r as f32 / 255.0,
                        green: uss_color.g as f32 / 255.0,
                        blue: uss_color.b as f32 / 255.0,
                        alpha: uss_color.a,
                    },
                })
            }
            UssValue::Identifier(keyword) => {
                // Check if this identifier is a color keyword
                if let Some((r, g, b)) = self.definitions.get_color_rgb(&keyword) {
                    let range = self.node_to_range(node, content)?;
                    
                    Some(ColorInformation {
                        range,
                        color: LspColor {
                            red: r as f32 / 255.0,
                            green: g as f32 / 255.0,
                            blue: b as f32 / 255.0,
                            alpha: 1.0,
                        },
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Convert a tree-sitter node to an LSP Range
    fn node_to_range(&self, node: &Node, _content: &str) -> Option<Range> {
        let start_point = node.start_position();
        let end_point = node.end_position();
        
        Some(Range {
            start: Position {
                line: start_point.row as u32,
                character: start_point.column as u32,
            },
            end: Position {
                line: end_point.row as u32,
                character: end_point.column as u32,
            },
        })
    }
    
    /// Provide color presentations for a given color
    pub fn provide_color_presentations(
        &self,
        color: &LspColor,
        range: Range,
    ) -> Vec<ColorPresentation> {
        let mut presentations = Vec::new();
        
        // Convert to 0-255 range
        let r = (color.red * 255.0).round() as u8;
        let g = (color.green * 255.0).round() as u8;
        let b = (color.blue * 255.0).round() as u8;
        let a = (color.alpha * 255.0).round() as u8;
        
        // Hex format
        if color.alpha == 1.0 {
            presentations.push(ColorPresentation {
                label: format!("#{:02x}{:02x}{:02x}", r, g, b),
                text_edit: Some(TextEdit { range, new_text: format!("#{:02x}{:02x}{:02x}", r, g, b) }),
                additional_text_edits: None,
            });
        } else {
            presentations.push(ColorPresentation {
                label: format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a),
                text_edit: Some(TextEdit { range, new_text: format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a) }),
                additional_text_edits: None,
            });
        }
        
        // RGB format
        if color.alpha == 1.0 {
            presentations.push(ColorPresentation {
                label: format!("rgb({}, {}, {})", r, g, b),
                text_edit: Some(TextEdit { range, new_text: format!("rgb({}, {}, {})", r, g, b) }),
                additional_text_edits: None,
            });
        } else {
            presentations.push(ColorPresentation {
                label: format!("rgba({}, {}, {}, {})", r, g, b, color.alpha),
                text_edit: Some(TextEdit { range, new_text: format!("rgba({}, {}, {}, {})", r, g, b, color.alpha) }),
                additional_text_edits: None,
            });
        }
        
        presentations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uss::parser::UssParser;
    

    
    #[test]
    fn test_hex_color_extraction() {
        let mut parser = UssParser::new().expect("Failed to create parser");
        let content = "Button { color: #ff0000; }";
        let tree = parser.parse(content, None).expect("Failed to parse");
        
        let color_provider = UssColorProvider::new();
        let colors = color_provider.provide_document_colors(&tree, content);
        
        assert_eq!(colors.len(), 1);
        let color_info = &colors[0];
        assert_eq!(color_info.color.red, 1.0);
        assert_eq!(color_info.color.green, 0.0);
        assert_eq!(color_info.color.blue, 0.0);
        assert_eq!(color_info.color.alpha, 1.0);
    }
    
    #[test]
    fn test_color_keyword_extraction() {
        let mut parser = UssParser::new().expect("Failed to create parser");
        let content = "Button { color: red; }";
        let tree = parser.parse(content, None).expect("Failed to parse");
        

        let color_provider = UssColorProvider::new();
        let colors = color_provider.provide_document_colors(&tree, content);
        
        assert_eq!(colors.len(), 1);
        let color_info = &colors[0];
        assert_eq!(color_info.color.red, 1.0);
        assert_eq!(color_info.color.green, 0.0);
        assert_eq!(color_info.color.blue, 0.0);
        assert_eq!(color_info.color.alpha, 1.0);
    }
    
    #[test]
    fn test_rgb_color_extraction() {
        let mut parser = UssParser::new().expect("Failed to create parser");
        let content = "Button { color: rgb(255, 0, 0); }";
        let tree = parser.parse(content, None).expect("Failed to parse");
        

        let color_provider = UssColorProvider::new();
        let colors = color_provider.provide_document_colors(&tree, content);
        
        assert_eq!(colors.len(), 1);
        let color_info = &colors[0];
        assert_eq!(color_info.color.red, 1.0);
        assert_eq!(color_info.color.green, 0.0);
        assert_eq!(color_info.color.blue, 0.0);
        assert_eq!(color_info.color.alpha, 1.0);
    }
}