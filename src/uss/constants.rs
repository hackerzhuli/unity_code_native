//! USS Tree-sitter Node Kind Constants
//!
//! This module contains all the constant for uss language and tree-sitter node kinds
//! used throughout the USS language support. Centralizing these constants helps
//! prevent mistakes and makes the code more maintainable.

/// Tree-sitter node kinds for USS/CSS syntax tree

// Basic structural nodes
/// Root node of the USS/CSS syntax tree
pub const NODE_STYLESHEET: &str = "stylesheet";
/// A CSS rule containing selectors and a declaration block
pub const NODE_RULE_SET: &str = "rule_set";
/// A block of declarations enclosed in curly braces
pub const NODE_BLOCK: &str = "block";
/// A single property-value pair (e.g., `color: red;`)
pub const NODE_DECLARATION: &str = "declaration";
/// Container for one or more selectors
pub const NODE_SELECTORS: &str = "selectors";

// Property and value nodes
/// CSS property name (e.g., `color`, `-unity-font`)
pub const NODE_PROPERTY_NAME: &str = "property_name";
/// Plain text value without quotes (e.g., `red`, `bold`)
pub const NODE_PLAIN_VALUE: &str = "plain_value";
/// Quoted string value (e.g., `"Arial"`, `'bold'`)
pub const NODE_STRING_VALUE: &str = "string_value";
/// Color value in various formats (e.g., `#ff0000`, `rgb(255,0,0)`)
pub const NODE_COLOR_VALUE: &str = "color_value";
/// Integer numeric value (e.g., `10`, `100`)
pub const NODE_INTEGER_VALUE: &str = "integer_value";
/// Floating-point numeric value (e.g., `1.5`, `0.75`)
pub const NODE_FLOAT_VALUE: &str = "float_value";
/// CSS unit identifier (e.g., `px`, `%`, `em`)
pub const NODE_UNIT: &str = "unit";

// Function calls
/// CSS function call (e.g., `rgb()`, `url()`, `resource()`)
pub const NODE_CALL_EXPRESSION: &str = "call_expression";
/// Name of a CSS function (e.g., `rgb`, `url`, `resource`)
pub const NODE_FUNCTION_NAME: &str = "function_name";
/// Arguments passed to a CSS function
pub const NODE_ARGUMENTS: &str = "arguments";

// Selector types
/// CSS class selector (e.g., `.my-class`)
pub const NODE_CLASS_SELECTOR: &str = "class_selector";
/// Name part of a class selector (e.g., `my-class` in `.my-class`)
pub const NODE_CLASS_NAME: &str = "class_name";
/// CSS ID selector (e.g., `#my-id`)
pub const NODE_ID_SELECTOR: &str = "id_selector";
/// Name part of an ID selector (e.g., `my-id` in `#my-id`)
pub const NODE_ID_NAME: &str = "id_name";
/// HTML/Unity UI element tag name (e.g., `Button`, `Label`)
pub const NODE_TAG_NAME: &str = "tag_name";
/// CSS pseudo-class selector (e.g., `:hover`, `:active`)
pub const NODE_PSEUDO_CLASS_SELECTOR: &str = "pseudo_class_selector";

// At-rules
/// Generic CSS at-rule (e.g., `@import`, `@media`)
pub const NODE_AT_RULE: &str = "at_rule";
/// The literal `@import` keyword
pub const KEYWORD_AT_IMPORT: &str = "@import";
/// CSS import statement for external stylesheets
pub const NODE_IMPORT_STATEMENT: &str = "import_statement";
/// CSS charset declaration statement
pub const NODE_CHARSET_STATEMENT: &str = "charset_statement";
/// CSS keyframes animation definition
pub const NODE_KEYFRAMES_STATEMENT: &str = "keyframes_statement";
/// CSS media query statement
pub const NODE_MEDIA_STATEMENT: &str = "media_statement";
/// CSS namespace declaration statement
pub const NODE_NAMESPACE_STATEMENT: &str = "namespace_statement";
/// CSS feature query statement
pub const NODE_SUPPORTS_STATEMENT: &str = "supports_statement";

// Punctuation and operators
/// Colon separator between property and value
pub const NODE_COLON: &str = ":";
/// Semicolon terminator for declarations
pub const NODE_SEMICOLON: &str = ";";
/// Comma separator for multiple values
pub const NODE_COMMA: &str = ",";
/// Opening parenthesis for function calls
pub const NODE_OPEN_PAREN: &str = "(";
/// Closing parenthesis for function calls
pub const NODE_CLOSE_PAREN: &str = ")";

// Comments
/// CSS comment block (e.g., `/* comment */`)
pub const NODE_COMMENT: &str = "comment";

// Error and special nodes
/// Tree-sitter error node for syntax errors
pub const NODE_ERROR: &str = "ERROR";

// URI schemes (for validation)
/// File URI scheme for local file references
pub const FILE_SCHEME: &str = "file";
/// Unity project URI scheme for project-relative references
pub const PROJECT_SCHEME: &str = "project";

// USS Units
/// Pixel unit for absolute length measurements
pub const UNIT_PX: &str = "px";
/// Percentage unit for relative measurements
pub const UNIT_PERCENT: &str = "%";
/// Degree unit for angle measurements (0-360)
pub const UNIT_DEG: &str = "deg";
/// Radian unit for angle measurements (0-2π)
pub const UNIT_RAD: &str = "rad";
/// Gradian unit for angle measurements (0-400)
pub const UNIT_GRAD: &str = "grad";
/// Turn unit for angle measurements (0-1 = full rotation)
pub const UNIT_TURN: &str = "turn";
/// Second unit for time measurements
pub const UNIT_S: &str = "s";
/// Millisecond unit for time measurements
pub const UNIT_MS: &str = "ms";
