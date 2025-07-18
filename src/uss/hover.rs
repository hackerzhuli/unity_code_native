//! USS Hover Provider
//!
//! Provides hover information for USS properties, including:
//! - Property descriptions
//! - Documentation URLs
//! - Unity version compatibility

use std::path::PathBuf;

use crate::unity_project_manager::UnityProjectManager;
use crate::uss::definitions::UssDefinitions;
use crate::language::tree_utils::{find_node_of_type_at_position, find_node_at_position, has_error_nodes};
use crate::uss::import_node::ImportNode;
use crate::uss::url_function_node::UrlFunctionNode;
use crate::uss::function_node::FunctionNode;
use crate::uss::uss_utils::convert_uss_string;
use crate::language::asset_url::{project_url_to_path, project_url_to_relative_path, validate_url};
use std::collections::HashMap;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Url};
use tree_sitter::{Node, Tree};
use crate::uss::constants::*;

/// USS Hover Provider
/// 
/// Provides comprehensive hover information for USS (Unity Style Sheets) code.
/// This includes property descriptions, documentation URLs, Unity version compatibility,
/// function information, unit descriptions, and UXML element details.
pub struct UssHoverProvider {
    definitions: UssDefinitions,
}

impl UssHoverProvider {
    /// Creates a new USS hover provider with initialized property and keyword definitions.
    /// 
    /// # Returns
    /// 
    /// A new `UssHoverProvider` instance ready to provide hover information for USS code.
    pub fn new() -> Self {
        Self {
            definitions: UssDefinitions::new(),
        }
    }

    /// Provides hover information for a specific position in USS code.
    /// 
    /// This method analyzes the syntax tree at the given position and returns appropriate
    /// hover information based on the context. It follows a priority-based approach to
    /// determine what type of hover information to display.
    /// 
    /// # Arguments
    /// 
    /// * `tree` - The parsed syntax tree of the USS document
    /// * `source` - The complete source code of the USS document
    /// * `position` - The LSP position where hover was requested (line/character)
    /// * `unity_manager` - Unity project manager for resolving file paths and versions
    /// * `source_url` - Optional URL of the current USS file for relative path resolution
    /// * `uxml_elements` - Optional map of UXML element names to their fully qualified types
    /// 
    /// # Returns
    /// 
    /// Returns `Some(Hover)` with markdown content if hover information is available,
    /// or `None` if no relevant information can be provided for the position.
    /// 
    /// # Priority Order
    /// 
    /// The method checks for hover targets in this order:
    /// 1. **Functions** - url(), rgb(), rgba(), var(), resource()
    /// 2. **Units** - px, %, deg, rad, s, ms, etc.
    /// 3. **Import statements** - @import declarations
    /// 4. **Tag selectors** - UXML element names (Button, Label, etc.)
    /// 5. **Pseudo-class selectors** - :hover, :focus, :active, etc.
    /// 6. **Declaration nodes** - CSS properties and their values
    pub fn hover(
        &self,
        tree: &Tree,
        source: &str,
        position: Position,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
        uxml_elements: Option<&HashMap<String, String>>,
    ) -> Option<Hover> {
        // Find the deepest node at the position
        let node = find_node_at_position(tree.root_node(), position)?;
        
        // Check if the node or any parent has errors - if so, don't show hover
        if has_error_nodes(node) {
            return None;
        }

        // Priority 1: Functions (including URL functions)
        if let Some(call_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_CALL_EXPRESSION) {
            if !has_error_nodes(call_node) {
                if let Some(hover) = self.hover_for_function(call_node, source, unity_manager, source_url) {
                    return Some(hover);
                }
            }
        }

        // Priority 2: Units
        if let Some(unit_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_UNIT) {
            if !has_error_nodes(unit_node) {
                if let Some(hover) = self.hover_for_unit(unit_node, source) {
                    return Some(hover);
                }
            }
        }

        // Priority 3: Import statements
        if let Some(import_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_IMPORT_STATEMENT) {
            if !has_error_nodes(import_node) {
                return self.hover_for_import_statement(import_node, source, unity_manager, source_url);
            }
        }

        // Priority 4: Tag selectors
        if let Some(tag_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_TAG_NAME) {
            if !has_error_nodes(tag_node) {
                if let Some(hover) = self.hover_for_tag_selector(tag_node, source, uxml_elements) {
                    return Some(hover);
                }
            }
        }

        // Priority 5: Pseudo-class selectors
        if let Some(pseudo_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_PSEUDO_CLASS_SELECTOR) {
            if !has_error_nodes(pseudo_node) {
                if let Some(hover) = self.hover_for_pseudo_class(pseudo_node, source, unity_manager) {
                    return Some(hover);
                }
            }
        }

        // Priority 6: Declaration nodes (properties and their values)
        if let Some(declaration_node) = find_node_of_type_at_position(tree.root_node(), source, position, NODE_DECLARATION) {
            if !has_error_nodes(declaration_node) {
                return self.hover_for_declaration(declaration_node, source, position, unity_manager);
            }
        }

        None
    }

    /// Provides hover information for @import statements.
    /// 
    /// Analyzes import statements and provides information about the imported file,
    /// including file existence, resolved paths, and usage documentation.
    fn hover_for_import_statement(
        &self,
        import_node: Node,
        source: &str,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
    ) -> Option<Hover> {
        let mut diagnostics = Vec::new();
        let import = ImportNode::from_node(import_node, source, &mut diagnostics)?;
        
        let mut content = String::from("**@import Statement**\n\n");
        
        // Check if the current import resolves to a file and add a link if it exists
        if let Some(import_path) = self.extract_path_from_node(import.argument_node, source) {
            if let Some((file_path, url)) = self.resolve_import_file_path(&import_path, unity_manager, source_url) {
                if let Some(relative_path) = project_url_to_relative_path(&url){
                    content.push_str(&format!("Asset path: `{}`\n\n", relative_path));
                }
        
                if file_path.exists() {
                    if let Ok(file_url) = Url::from_file_path(&file_path) {
                        content.push_str(&format!("📁 [Open file]({})\n\n", file_url));
                    }
                } else {
                    content.push_str("❌ File not found\n\n");
                }
            }
        }

        // Add documentation about what @import does
        content.push_str("Imports styles from another USS file into the current stylesheet.\n\n");
        
        // Add usage examples
        content.push_str("**Examples:**\n");
        content.push_str("```css\n");
        content.push_str("@import url(\"project:///Assets/UI/styles/common.uss\");\n");
        content.push_str("@import \"./common.uss\";\n");
        content.push_str("```\n\n");
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for CSS functions.
    /// 
    /// Dispatches to specific hover handlers based on the function name.
    /// Supports url(), resource(), rgb(), rgba(), and var() functions.
    fn hover_for_function(
        &self,
        call_node: Node,
        source: &str,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
    ) -> Option<Hover> {
        let function_node = FunctionNode::from_node(call_node, source, None)?;
        
        match function_node.function_name.as_str() {
            "url" => self.hover_for_url_function(call_node, source, unity_manager, source_url),
            "resource" => self.hover_for_resource_function(call_node, source),
            "rgb" => self.hover_for_rgb_function(call_node, source),
            "rgba" => self.hover_for_rgba_function(call_node, source),
            "var" => self.hover_for_var_function(call_node, source),
            _ => {
                // For other functions, use the structured data if available
                if let Some(function_info) = self.definitions.get_function_info(&function_node.function_name) {
                    let content = function_info.create_documentation();
                    Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: None,
                    })
                } else {
                    None
                }
            }
        }
    }

    /// Provides hover information for url() functions.
    /// 
    /// Analyzes url() function calls and provides information about the referenced
    /// file or resource, including path resolution and file existence checking.
    fn hover_for_url_function(
        &self,
        call_node: Node,
        source: &str,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
    ) -> Option<Hover> {
        let url_function = UrlFunctionNode::from_node(call_node, source, None, source_url, None, false)?;
        
        let mut content = format!("**url()**\n\n");
        content.push_str("References an external resource by URL or file path.\n\n");

        // Try to resolve the file path and check if it exists
        if let Some((file_path, url)) = self.resolve_import_file_path(url_function.url(), unity_manager, source_url) {
            if let Some(relative_path) = project_url_to_relative_path(&url){
                content.push_str(&format!("Asset path: `{}`\n\n", relative_path));
            }
        
            if file_path.exists() {
                // Create a file:// URL for the resolved path
                if let Ok(file_url) = Url::from_file_path(&file_path) {
                    content.push_str(&format!("[📂 Open File]({})", file_url));
                }
            } else {
                content.push_str(&format!("❌ File not found"));
            }
        } else {
            content.push_str(&format!("⚠️ Could not resolve file path"));
        }
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for resource() functions.
    /// 
    /// Analyzes resource() function calls which reference Unity assets from
    /// the Resources folder. Provides documentation about the function and
    /// shows the referenced resource path.
    fn hover_for_resource_function(
        &self,
        call_node: Node,
        source: &str,
    ) -> Option<Hover> {
        let function_node = FunctionNode::from_node(call_node, source, None)?;
        
        let content = if let Some(path) = function_node.get_argument_text(0, source) {
            format!(
                "**resource()** - Unity Resource Function\n\n\
                References a Unity resource from the Resources folder.\n\n\
                **Path:** `{}`\n\n\
                The `resource()` function loads assets from Unity's Resources folder. \
                The path should be relative to any Resources folder in your project.",
                path
            )
        } else {
            "**resource()** - Unity Resource Function\n\nReferences a Unity resource from the Resources folder.\n\nSyntax: `resource(\"path/to/resource\")`".to_string()
        };
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for RGB functions
    fn hover_for_rgb_function(
        &self,
        call_node: Node,
        source: &str,
    ) -> Option<Hover> {
        let function_node = FunctionNode::from_node(call_node, source, None)?;
        
        let content = if let Some(function_info) = self.definitions.get_function_info("rgb") {
            let mut base_content = function_info.create_documentation();
            
            // Add specific argument information if available
            if function_node.argument_count() >= 3 {
                let r = function_node.get_argument_text(0, source).unwrap_or_else(|| "?".to_string());
                let g = function_node.get_argument_text(1, source).unwrap_or_else(|| "?".to_string());
                let b = function_node.get_argument_text(2, source).unwrap_or_else(|| "?".to_string());
                
                base_content.push_str(&format!("\n\n**Values:** R={}, G={}, B={}", r, g, b));
            }
            
            base_content
        } else {
            // Fallback if function info is not available
            if function_node.argument_count() >= 3 {
                let r = function_node.get_argument_text(0, source).unwrap_or_else(|| "?".to_string());
                let g = function_node.get_argument_text(1, source).unwrap_or_else(|| "?".to_string());
                let b = function_node.get_argument_text(2, source).unwrap_or_else(|| "?".to_string());
                
                format!(
                    "**rgb()** - RGB Color Function\n\n\
                    Defines a color using red, green, and blue values.\n\n\
                    **Values:** R={}, G={}, B={}\n\n\
                    Each component can be a number (0-255) or percentage (0%-100%).",
                    r, g, b
                )
            } else {
                "**rgb()** - RGB Color Function\n\nDefines a color using red, green, and blue values.\n\nSyntax: `rgb(red, green, blue)`".to_string()
            }
        };
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for RGBA functions
    fn hover_for_rgba_function(
        &self,
        call_node: Node,
        source: &str,
    ) -> Option<Hover> {
        let function_node = FunctionNode::from_node(call_node, source, None)?;
        
        let content = if let Some(function_info) = self.definitions.get_function_info("rgba") {
            let mut base_content = function_info.create_documentation();
            
            // Add specific argument information if available
            if function_node.argument_count() >= 4 {
                let r = function_node.get_argument_text(0, source).unwrap_or_else(|| "?".to_string());
                let g = function_node.get_argument_text(1, source).unwrap_or_else(|| "?".to_string());
                let b = function_node.get_argument_text(2, source).unwrap_or_else(|| "?".to_string());
                let a = function_node.get_argument_text(3, source).unwrap_or_else(|| "?".to_string());
                
                base_content.push_str(&format!("\n\n**Values:** R={}, G={}, B={}, A={}", r, g, b, a));
            }
            
            base_content
        } else {
            // Fallback if function info is not available
            if function_node.argument_count() >= 4 {
                let r = function_node.get_argument_text(0, source).unwrap_or_else(|| "?".to_string());
                let g = function_node.get_argument_text(1, source).unwrap_or_else(|| "?".to_string());
                let b = function_node.get_argument_text(2, source).unwrap_or_else(|| "?".to_string());
                let a = function_node.get_argument_text(3, source).unwrap_or_else(|| "?".to_string());
                
                format!(
                    "**rgba()** - RGBA Color Function\n\n\
                    Defines a color using red, green, blue, and alpha values.\n\n\
                    **Values:** R={}, G={}, B={}, A={}\n\n\
                    RGB components can be numbers (0-255) or percentages (0%-100%). \
                    Alpha is a decimal from 0.0 (transparent) to 1.0 (opaque).",
                    r, g, b, a
                )
            } else {
                "**rgba()** - RGBA Color Function\n\nDefines a color using red, green, blue, and alpha values.\n\nSyntax: `rgba(red, green, blue, alpha)`".to_string()
            }
        };
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for var functions (CSS custom properties)
    fn hover_for_var_function(
        &self,
        call_node: Node,
        source: &str,
    ) -> Option<Hover> {
        let function_node = FunctionNode::from_node(call_node, source, None)?;
        
        let content = if let Some(function_info) = self.definitions.get_function_info("var") {
            let mut base_content = function_info.create_documentation();
            
            // Add specific variable information if available
            if let Some(var_name) = function_node.get_argument_text(0, source) {
                base_content.push_str(&format!("\n\n**Variable:** `{}`", var_name));
                
                if let Some(fallback) = function_node.get_argument_text(1, source) {
                    base_content.push_str(&format!("\n\n**Fallback:** `{}`", fallback));
                }
            }
            
            base_content
        } else {
            // Fallback if function info is not available
            if let Some(var_name) = function_node.get_argument_text(0, source) {
                let fallback = function_node.get_argument_text(1, source)
                    .map(|s| format!("\n\n**Fallback:** `{}`", s))
                    .unwrap_or_default();
                format!(
                    "**var()** - CSS Custom Property\n\n\
                    References a custom CSS property (variable).\n\n\
                    **Variable:** `{}`{}",
                    var_name, fallback
                )
            } else {
                "**var()** - CSS Custom Property\n\nReferences a custom CSS property (variable).\n\nSyntax: `var(--property-name, fallback)`".to_string()
            }
        };
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Extracts a path string from a tree-sitter node (string or url function)
    fn extract_path_from_node(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            NODE_STRING_VALUE => {
                let raw_string = node.utf8_text(source.as_bytes()).ok()?;
                convert_uss_string(raw_string).ok()
            }
            NODE_CALL_EXPRESSION => {
                // This should be a url() function
                let url_function = UrlFunctionNode::from_node(node, source, None, None, None, false)?;
                Some(url_function.url().to_string())
            }
            _ => None,
        }
    }

    /// Resolves an import file path to an absolute file system path
    fn resolve_import_file_path(
        &self,
        import_path: &str,
        unity_manager: &UnityProjectManager,
        source_url: Option<&Url>,
    ) -> Option<(PathBuf, Url)> {
        // Try to validate the URL first to get a proper project URL
        if let Ok(validation_result) = validate_url(import_path, source_url) {
            // Convert project URL to file system path
            let project_root = unity_manager.project_path();
            if let Some(path) = project_url_to_path(project_root, &validation_result.url){
                return Some((path, validation_result.url));
            }
        }
        None
    }

    /// Provides hover information for CSS units.
    /// 
    /// Analyzes unit identifiers (px, %, deg, s, etc.) and provides documentation
    /// about their meaning and usage in USS.
    fn hover_for_unit(&self, unit_node: Node, source: &str) -> Option<Hover> {
        let unit_text = unit_node.utf8_text(source.as_bytes()).ok()?;
        
        let unit_info = self.definitions.get_unit_info(unit_text)?;
        let content = unit_info.create_documentation();

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for tag selectors (UXML elements).
    /// 
    /// Analyzes tag selectors that target UXML elements and provides information
    /// about the element type, including its fully qualified name when available.
    fn hover_for_tag_selector(&self, tag_node: Node, source: &str, uxml_elements: Option<&HashMap<String, String>>) -> Option<Hover> {
        let tag_text = tag_node.utf8_text(source.as_bytes()).ok()?;
        
        let content = if let Some(elements) = uxml_elements {
            if let Some(fully_qualified_name) = elements.get(tag_text) {
                format!(
                    "**{}** - Unity UI Element\n\n**Full Name:** `{}`\n\nThis selector targets all `{}` elements in the UI hierarchy.",
                    tag_text, fully_qualified_name, tag_text
                )
            } else {
                format!("**{}** - Unity UI Element\n\nThis selector targets all `{}` elements in the UI hierarchy.\n\n⚠️ Element not found in UXML schema.", tag_text, tag_text)
            }
        } else {
            // Fallback when elements map is not available
            format!("**{}** - Unity UI Element\n\nThis selector targets all `{}` elements in the UI hierarchy.", tag_text, tag_text)
        };
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for pseudo-class selectors.
    /// 
    /// Analyzes pseudo-class selectors (:hover, :focus, :active, etc.) and provides
    /// documentation about their behavior and usage in Unity UI.
    fn hover_for_pseudo_class(&self, pseudo_node: Node, source: &str, unity_manager: &UnityProjectManager) -> Option<Hover> {
        // Extract the pseudo-class name from the selector
        // The pseudo-class selector contains the full selector, we need to find the class_name that represents the pseudo-class
        let mut pseudo_class_name = None;
        for i in 0..pseudo_node.child_count() {
            if let Some(child) = pseudo_node.child(i) {
                if child.kind() == NODE_CLASS_NAME {
                    if let Ok(class_text) = child.utf8_text(source.as_bytes()) {
                        // Check if this is a valid pseudo-class
                        if self.definitions.is_valid_pseudo_class(class_text) {
                            pseudo_class_name = Some(class_text);
                            break;
                        }
                    }
                }
            }
        }

        let pseudo_class = pseudo_class_name?;
        let pseudo_info = self.definitions.get_pseudo_class_info(pseudo_class)?;
        
        let unity_version_for_docs = unity_manager.get_unity_version_for_docs()
            .unwrap_or_else(|| "6000.0".to_string());
        
        let content = pseudo_info.create_documentation(&unity_version_for_docs);
        
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        })
    }

    /// Provides hover information for CSS declarations (properties and their values).
    /// 
    /// Analyzes CSS property declarations and provides context-sensitive hover information.
    /// Can show property documentation or value-specific information depending on cursor position.
    fn hover_for_declaration(
        &self,
        declaration_node: Node,
        source: &str,
        position: Position,
        unity_manager: &UnityProjectManager,
    ) -> Option<Hover> {
        // Get the property name
        let property_name_node = declaration_node.child(0)?;
        if property_name_node.kind() != NODE_PROPERTY_NAME {
            return None;
        }
        
        let property_name = property_name_node.utf8_text(source.as_bytes()).ok()?;
        
        // Check if we're hovering over a value within the declaration
        if let Some(value_hover) = self.hover_for_value_in_declaration(declaration_node, source, position, &property_name, unity_manager) {
            return Some(value_hover);
        }
        
        // Default to showing property documentation if it's a known property
        if self.definitions.is_predefined_property(&property_name) {
            return Some(self.create_hover_content_for_property(&property_name, unity_manager));
        }
        
        None
    }

    /// Provides hover information for values within declarations
    fn hover_for_value_in_declaration(
        &self,
        declaration_node: Node,
        source: &str,
        position: Position,
        property_name: &str,
        unity_manager: &UnityProjectManager,
    ) -> Option<Hover> {
        // Find the value node at the position
        let value_node = find_node_at_position(declaration_node, position)?;
        
        // Only handle plain_value nodes (identifiers/keywords)
        if value_node.kind() != NODE_PLAIN_VALUE {
            return None;
        }
        
        let value_text = value_node.utf8_text(source.as_bytes()).ok()?;
        
        // Check if this is a transition property that might reference other properties
        if property_name == "transition-property" || property_name == "transition" {
            if self.definitions.is_predefined_property(value_text) {
                let property_info = self.definitions.get_property_info(value_text)?;
                if property_info.animatable != crate::uss::definitions::PropertyAnimation::None {
                    return Some(self.create_hover_content_for_property(value_text, unity_manager));
                }
            }
        }
        
        // Check if this is a keyword
        if let Some(keyword_info) = self.definitions.get_keyword_info(value_text) {
            let content = keyword_info.create_documentation(Some(property_name));
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: None,
            });
        }
        
        None
    }

    /// Creates a generic function hover
    fn create_function_hover(&self, function_name: &str, description: &str) -> Hover {
        let content = format!("**{}()**\n\n{}", function_name, description);
        
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        }
    }

    /// Creates hover content for a USS property.
    /// 
    /// Generates comprehensive hover documentation for a given USS property,
    /// including description, syntax, Unity version compatibility, and documentation links.
    fn create_hover_content_for_property(
        &self,
        property_name: &str,
        unity_manager: &UnityProjectManager,
    ) -> Hover {
        let unity_version_for_docs = unity_manager.get_unity_version_for_docs()
            .unwrap_or_else(|| "6000.0".to_string());
        let property_info = self.definitions.get_property_info(property_name);
        if property_info.is_none(){
            return Hover{
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Unknown Property".to_string(),
                }),
                range: None,
            };
        }
        
        let property_info2 = property_info.unwrap();

        let content = property_info2.create_documentation(property_name, &unity_version_for_docs);
        
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        }
    }
}

impl Default for UssHoverProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path ="hover_tests.rs"]
mod hover_tests;