//! USS Color Provider Module
//!
//! Provides color information for USS files to enable color decorations in VS Code.

use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Tree};
use crate::uss::definitions::UssDefinitions;

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
            "color_value" => {
                if let Some(color_info) = self.extract_color_from_node(node, content) {
                    colors.push(color_info);
                }
            }
            "plain_value" => {
                // Check if this plain_value is a color keyword or hex color
                let text = node.utf8_text(content.as_bytes()).unwrap_or("");
                if text.starts_with('#') {
                    if let Some(color_info) = self.extract_color_from_node(node, content) {
                        colors.push(color_info);
                    }
                } else if let Some(color_info) = self.extract_color_keyword_from_node(node, content) {
                    colors.push(color_info);
                }
            }
            "call_expression" => {
                // Check for rgb(), rgba(), hsl(), hsla() functions
                if let Some(color_info) = self.extract_color_function_from_node(node, content) {
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
    
    /// Extract color information from a color_value node (hex colors)
    fn extract_color_from_node(&self, node: &Node, content: &str) -> Option<ColorInformation> {
        let color_text = node.utf8_text(content.as_bytes()).ok()?;
        
        if color_text.starts_with('#') {
            let hex_part = &color_text[1..];
            
            // Parse hex color
            let (r, g, b, a) = match hex_part.len() {
                3 => {
                    // #RGB -> #RRGGBB
                    let r = u8::from_str_radix(&hex_part[0..1].repeat(2), 16).ok()?;
                    let g = u8::from_str_radix(&hex_part[1..2].repeat(2), 16).ok()?;
                    let b = u8::from_str_radix(&hex_part[2..3].repeat(2), 16).ok()?;
                    (r, g, b, 255)
                }
                6 => {
                    // #RRGGBB
                    let r = u8::from_str_radix(&hex_part[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex_part[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex_part[4..6], 16).ok()?;
                    (r, g, b, 255)
                }
                8 => {
                    // #RRGGBBAA
                    let r = u8::from_str_radix(&hex_part[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex_part[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex_part[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&hex_part[6..8], 16).ok()?;
                    (r, g, b, a)
                }
                _ => return None,
            };
            
            let range = self.node_to_range(node, content)?;
            
            Some(ColorInformation {
                range,
                color: Color {
                    red: r as f32 / 255.0,
                    green: g as f32 / 255.0,
                    blue: b as f32 / 255.0,
                    alpha: a as f32 / 255.0,
                },
            })
        } else {
            None
        }
    }
    
    /// Extract color information from a plain_value node (color keywords)
    fn extract_color_keyword_from_node(&self, node: &Node, content: &str) -> Option<ColorInformation> {
        let color_text = node.utf8_text(content.as_bytes()).ok()?;
        
        // Check if this is a valid color keyword
        if let Some(hex_value) = self.definitions.get_color_hex_value(color_text) {
            let range = self.node_to_range(node, content)?;
            
            // Parse the hex value (remove # if present)
            let hex_part = if hex_value.starts_with('#') { &hex_value[1..] } else { &hex_value };
            let r = u8::from_str_radix(&hex_part[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex_part[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex_part[4..6], 16).ok()?;
            
            Some(ColorInformation {
                range,
                color: Color {
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
    
    /// Extract color information from a call_expression node (rgb, rgba, hsl, hsla functions)
    fn extract_color_function_from_node(&self, node: &Node, content: &str) -> Option<ColorInformation> {
        // Get the function name (first child should be function_name)
        let function_name_node = node.child(0)?;
        let function_name = function_name_node.utf8_text(content.as_bytes()).ok()?;
        
        match function_name {
            "rgb" | "rgba" => {
                self.extract_rgb_color(node, content)
            }
            "hsl" | "hsla" => {
                self.extract_hsl_color(node, content)
            }
            _ => None,
        }
    }
    
    /// Extract RGB/RGBA color values
    fn extract_rgb_color(&self, node: &Node, content: &str) -> Option<ColorInformation> {
        // Get the arguments node (second child should be arguments)
        let arguments_node = node.child(1)?;
        let mut values = Vec::new();
        
        // Extract numeric values from arguments
        for i in 0..arguments_node.child_count() {
            if let Some(child) = arguments_node.child(i) {
                if matches!(child.kind(), "integer_value" | "float_value") {
                    if let Ok(text) = child.utf8_text(content.as_bytes()) {
                        if let Ok(value) = text.parse::<f64>() {
                            values.push(value);
                        }
                    }
                }
            }
        }
        
        if values.len() >= 3 {
            let r = (values[0] / 255.0).clamp(0.0, 1.0);
            let g = (values[1] / 255.0).clamp(0.0, 1.0);
            let b = (values[2] / 255.0).clamp(0.0, 1.0);
            let a = if values.len() >= 4 { values[3].clamp(0.0, 1.0) } else { 1.0 };
            
            let range = self.node_to_range(node, content)?;
            
            Some(ColorInformation {
                range,
                color: Color {
                    red: r as f32,
                    green: g as f32,
                    blue: b as f32,
                    alpha: a as f32,
                },
            })
        } else {
            None
        }
    }
    
    /// Extract HSL/HSLA color values
    fn extract_hsl_color(&self, node: &Node, content: &str) -> Option<ColorInformation> {
        let arguments_node = node.child_by_field_name("arguments")?;
        let mut values = Vec::new();
        
        // Extract numeric values from arguments
        for i in 0..arguments_node.child_count() {
            if let Some(child) = arguments_node.child(i) {
                if matches!(child.kind(), "integer_value" | "float_value") {
                    if let Ok(text) = child.utf8_text(content.as_bytes()) {
                        if let Ok(value) = text.parse::<f64>() {
                            values.push(value);
                        }
                    }
                }
            }
        }
        
        if values.len() >= 3 {
            let h = (values[0] % 360.0) / 360.0;
            let s = (values[1] / 100.0).clamp(0.0, 1.0);
            let l = (values[2] / 100.0).clamp(0.0, 1.0);
            let a = if values.len() >= 4 { values[3].clamp(0.0, 1.0) } else { 1.0 };
            
            // Convert HSL to RGB
            let (r, g, b) = self.hsl_to_rgb(h, s, l);
            
            let range = self.node_to_range(node, content)?;
            
            Some(ColorInformation {
                range,
                color: Color {
                    red: r as f32,
                    green: g as f32,
                    blue: b as f32,
                    alpha: a as f32,
                },
            })
        } else {
            None
        }
    }
    
    /// Convert HSL to RGB
    fn hsl_to_rgb(&self, h: f64, s: f64, l: f64) -> (f64, f64, f64) {
        if s == 0.0 {
            // Achromatic (gray)
            (l, l, l)
        } else {
            let hue_to_rgb = |p: f64, q: f64, mut t: f64| {
                if t < 0.0 { t += 1.0; }
                if t > 1.0 { t -= 1.0; }
                if t < 1.0/6.0 { p + (q - p) * 6.0 * t }
                else if t < 1.0/2.0 { q }
                else if t < 2.0/3.0 { p + (q - p) * (2.0/3.0 - t) * 6.0 }
                else { p }
            };
            
            let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
            let p = 2.0 * l - q;
            
            let r = hue_to_rgb(p, q, h + 1.0/3.0);
            let g = hue_to_rgb(p, q, h);
            let b = hue_to_rgb(p, q, h - 1.0/3.0);
            
            (r, g, b)
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
        color: &Color,
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