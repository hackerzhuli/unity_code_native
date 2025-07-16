//! USS and Tree-sitter CSS Node Kind Constants
//!
//! This module contains all the constant for uss language and tree-sitter node kinds
//! used throughout the USS language support. Centralizing these constants helps
//! prevent mistakes and makes the code more maintainable.
//! 
//! Tree-sitter node kinds for USS/CSS syntax tree
//! 
//! Note that the parse is CSS parser, so there are node kinds that is not supported in USS

// Basic structural nodes
/// Root node of the USS syntax tree
pub const NODE_STYLESHEET: &str = "stylesheet";
/// A USS rule containing selectors and a declaration block
pub const NODE_RULE_SET: &str = "rule_set";
/// A block of declarations enclosed in curly braces
pub const NODE_BLOCK: &str = "block";
/// A single property-value pair (e.g., `color: red;`)
pub const NODE_DECLARATION: &str = "declaration";
/// Container for one or more selectors
pub const NODE_SELECTORS: &str = "selectors";

// Property and value nodes
/// USS property name (e.g., `color`, `-unity-font`)
pub const NODE_PROPERTY_NAME: &str = "property_name";

/// Incomplete property name in error node when user is still typing the property name before typing colon, or a partial pseudo class
pub const NODE_ATTRIBUTE_NAME: &'static str = "attribute_name";

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
/// USS unit identifier (e.g., `px`, `%`, `em`)
pub const NODE_UNIT: &str = "unit";

// Function calls
/// USS function call (e.g., `rgb()`, `url()`, `resource()`)
pub const NODE_CALL_EXPRESSION: &str = "call_expression";
/// Name of a USS function (e.g., `rgb`, `url`, `resource`)
pub const NODE_FUNCTION_NAME: &str = "function_name";
/// Arguments passed to a USS function
pub const NODE_ARGUMENTS: &str = "arguments";

// Selector types
/// USS class selector (e.g., `.my-class`)
pub const NODE_CLASS_SELECTOR: &str = "class_selector";
/// Descendant selector, eg. `.a .b c`
pub const NODE_DESCENDANT_SELECTOR:&str= "descendant_selector";
/// Name part of a class selector or a pseudo class selector (e.g., `my-class` in `.my-class` or `hover` in `.my-class:hover`)
pub const NODE_CLASS_NAME: &str = "class_name";
/// USS ID selector (e.g., `#my-id`)
pub const NODE_ID_SELECTOR: &str = "id_selector";
/// Name part of an ID selector (e.g., `my-id` in `#my-id`)
pub const NODE_ID_NAME: &str = "id_name";
/// HTML/Unity UI element tag name (e.g., `Button`, `Label`)
pub const NODE_TAG_NAME: &str = "tag_name";
/// USS pseudo-class selector (e.g., `.my-class:hover`, `.my-class:active`), note this is the full selector including everything and the pseudo class part as the last child
pub const NODE_PSEUDO_CLASS_SELECTOR: &str = "pseudo_class_selector";

// At-rules
/// Generic CSS at-rule
/// The keyword part of the at rule (it may not be an existing keyword)
pub const NODE_AT_KEYWORD: &str = "at_keyword";
pub const NODE_AT_RULE: &str = "at_rule";
/// The literal `@import` keyword node, note this is not the import statement just the keyword part
pub const NODE_IMPORT: &str = "@import";
/// USS import statement for external stylesheets
pub const NODE_IMPORT_STATEMENT: &str = "import_statement";
/// CSS charset declaration statement (not supported in USS)
pub const NODE_CHARSET_STATEMENT: &str = "charset_statement";
/// CSS keyframes animation definition (not supported in USS)
pub const NODE_KEYFRAMES_STATEMENT: &str = "keyframes_statement";
/// CSS media query statement (not supported in USS)
pub const NODE_MEDIA_STATEMENT: &str = "media_statement";
/// CSS namespace declaration statement (not supported in USS)
pub const NODE_NAMESPACE_STATEMENT: &str = "namespace_statement";
/// CSS feature query statement (not supported in USS)
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
/// USS comment block (e.g., `/* comment */`)
pub const NODE_COMMENT: &str = "comment";

// Error and special nodes
/// Tree-sitter error node for syntax errors
pub const NODE_ERROR: &str = "ERROR";

// URI schemes (for validation)
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
