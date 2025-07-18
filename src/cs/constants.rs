//! Tree-sitter node kind constants for C# parsing
//!
//! This module contains string constants for all tree-sitter node kinds used
//! when parsing C# source code. These constants help maintain consistency
//! and provide better IDE support with autocompletion and refactoring.

/// Tree-sitter node kind for namespace declarations
/// 
/// Example: `namespace MyNamespace { ... }`
pub const NAMESPACE_DECLARATION: &str = "namespace_declaration";

/// Tree-sitter node kind for class declarations
/// 
/// Example: `public class MyClass { ... }`
pub const CLASS_DECLARATION: &str = "class_declaration";

/// Tree-sitter node kind for interface declarations
/// 
/// Example: `public interface IMyInterface { ... }`
pub const INTERFACE_DECLARATION: &str = "interface_declaration";

/// Tree-sitter node kind for struct declarations
/// 
/// Example: `public struct MyStruct { ... }`
pub const STRUCT_DECLARATION: &str = "struct_declaration";

/// Tree-sitter node kind for enum declarations
/// 
/// Example: `public enum MyEnum { ... }`
pub const ENUM_DECLARATION: &str = "enum_declaration";

/// Tree-sitter node kind for method declarations
/// 
/// Example: `public void MyMethod() { ... }`
pub const METHOD_DECLARATION: &str = "method_declaration";

/// Tree-sitter node kind for property declarations
/// 
/// Example: `public string MyProperty { get; set; }`
pub const PROPERTY_DECLARATION: &str = "property_declaration";

/// Tree-sitter node kind for field declarations
/// 
/// Example: `private int myField;`
pub const FIELD_DECLARATION: &str = "field_declaration";

/// Tree-sitter node kind for event declarations
/// 
/// Example: `public event Action MyEvent;`
pub const EVENT_DECLARATION: &str = "event_declaration";

/// Tree-sitter node kind for declaration lists
pub const DECLARATION_LIST: &str = "declaration_list";

/// Tree-sitter node kind for modifier keywords
/// 
/// Example: `public`, `private`, `static`, etc.
pub const MODIFIER: &str = "modifier";

/// Tree-sitter node kind for using directives
/// 
/// Example: `using System;`
pub const USING_DIRECTIVE: &str = "using_directive";

/// Tree-sitter node kind for qualified names
/// 
/// Example: `System.Collections.Generic`
pub const QUALIFIED_NAME: &str = "qualified_name";

/// Tree-sitter node kind for simple identifiers
/// 
/// Example: `MyClass`, `myVariable`
pub const IDENTIFIER: &str = "identifier";

/// Tree-sitter node kind for comments
/// 
/// Example: `// comment` or `/* comment */`
pub const COMMENT: &str = "comment";

/// Tree-sitter node kind for whitespace
/// 
/// Includes spaces, tabs, newlines, etc.
pub const WHITESPACE: &str = "whitespace";

/// Tree-sitter field name for accessing the name of a declaration
/// 
/// Used with `node.child_by_field_name(NAME_FIELD)`
pub const NAME_FIELD: &str = "name";

/// Tree-sitter field name for accessing the body of a declaration
/// 
/// Used with `node.child_by_field_name(BODY_FIELD)`
pub const BODY_FIELD: &str = "body";

/// Public access modifier
pub const PUBLIC_MODIFIER: &str = "public";

/// File extension for C# project files
pub const CSPROJ_EXTENSION: &str = "csproj";

/// File extension for Unity assembly definition files
pub const ASMDEF_EXTENSION: &str = "asmdef";

/// Prefix for XML documentation comments
pub const XML_DOC_COMMENT_PREFIX: &str = "///";

// Additional Tree-sitter node kinds for C# parsing

/// Tree-sitter node kind for constructor declarations
/// 
/// Example: `public MyClass() { ... }`
pub const CONSTRUCTOR_DECLARATION: &str = "constructor_declaration";

/// Tree-sitter node kind for destructor declarations
/// 
/// Example: `~MyClass() { ... }`
pub const DESTRUCTOR_DECLARATION: &str = "destructor_declaration";

/// Tree-sitter node kind for variable declarations
/// 
/// Example: `int x, y;`
pub const VARIABLE_DECLARATION: &str = "variable_declaration";

/// Tree-sitter node kind for variable declarators
/// 
/// Example: `x = 5` in `int x = 5;`
pub const VARIABLE_DECLARATOR: &str = "variable_declarator";

/// Tree-sitter node kind for parameter declarations
/// 
/// Example: `int value` in `void Method(int value)`
pub const PARAMETER: &str = "parameter";

/// Tree-sitter node kind for parameter modifiers
/// 
/// Example: `ref`, `in`, `out` in method parameters
pub const PARAMETER_MODIFIER: &str = "parameter_modifier";

// Additional Tree-sitter field names

/// Tree-sitter field name for accessing the type of a declaration
/// 
/// Used with `node.child_by_field_name(TYPE_FIELD)`
pub const TYPE_FIELD: &str = "type";

/// Tree-sitter field name for accessing type parameters
/// 
/// Used with `node.child_by_field_name(TYPE_PARAMETERS_FIELD)`
pub const TYPE_PARAMETERS_FIELD: &str = "type_parameters";

/// Tree-sitter field name for accessing parameters
/// 
/// Used with `node.child_by_field_name(PARAMETERS_FIELD)`
pub const PARAMETERS_FIELD: &str = "parameters";

// C# System type constants

/// Fully qualified name for System.Int32
pub const SYSTEM_INT32_TYPE: &str = "System.Int32";

/// Fully qualified name for System.String
pub const SYSTEM_STRING_TYPE: &str = "System.String";

/// Fully qualified name for System.Boolean
pub const SYSTEM_BOOLEAN_TYPE: &str = "System.Boolean";

/// Fully qualified name for System.Double
pub const SYSTEM_DOUBLE_TYPE: &str = "System.Double";

/// Fully qualified name for System.Single
pub const SYSTEM_SINGLE_TYPE: &str = "System.Single";

/// Fully qualified name for System.Int64
pub const SYSTEM_INT64_TYPE: &str = "System.Int64";

/// Fully qualified name for System.Int16
pub const SYSTEM_INT16_TYPE: &str = "System.Int16";

/// Fully qualified name for System.Byte
pub const SYSTEM_BYTE_TYPE: &str = "System.Byte";

/// Fully qualified name for System.Object
pub const SYSTEM_OBJECT_TYPE: &str = "System.Object";

// C# primitive type aliases

/// C# primitive type alias for System.Int32
pub const INT_TYPE: &str = "int";

/// C# primitive type alias for System.String
pub const STRING_TYPE: &str = "string";

/// C# primitive type alias for System.Boolean
pub const BOOL_TYPE: &str = "bool";

/// C# primitive type alias for System.Double
pub const DOUBLE_TYPE: &str = "double";

/// C# primitive type alias for System.Single
pub const FLOAT_TYPE: &str = "float";

/// C# primitive type alias for System.Int64
pub const LONG_TYPE: &str = "long";

/// C# primitive type alias for System.Int16
pub const SHORT_TYPE: &str = "short";

/// C# primitive type alias for System.Byte
pub const BYTE_TYPE: &str = "byte";

/// C# primitive type alias for System.Object
pub const OBJECT_TYPE: &str = "object";

// Parameter modifier constants

/// Parameter modifier for reference parameters
pub const REF_MODIFIER_WITH_SPACE: &str = "ref ";

/// Parameter modifier for input parameters
pub const IN_MODIFIER_WITH_SPACE: &str = "in ";

/// Parameter modifier for output parameters
pub const OUT_MODIFIER_WITH_SPACE: &str = "out ";