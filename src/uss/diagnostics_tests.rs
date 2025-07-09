//! Tests for USS diagnostics functionality

use crate::uss::tree_printer::print_tree_to_stdout;

use super::diagnostics::*;
use super::parser::UssParser;
use tower_lsp::lsp_types::NumberOrString;

#[test]
fn test_precise_syntax_error_range() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with syntax error - a simple invalid token on line 4
    let content = ".valid-rule {\n    background-color: red;\n}\n\na;\n\n.another-valid-rule {\n    color: blue;\n}";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Debug: Check diagnostic details
    // Uncomment the following lines for debugging:
    // for (i, diagnostic) in results.iter().enumerate() {
    //     println!("  {}: Line {}:{}-{}:{} - {} - {}", 
    //         i, 
    //         diagnostic.range.start.line, diagnostic.range.start.character,
    //         diagnostic.range.end.line, diagnostic.range.end.character,
    //         diagnostic.code.as_ref().map(|c| match c {
    //             tower_lsp::lsp_types::NumberOrString::String(s) => s.as_str(),
    //             tower_lsp::lsp_types::NumberOrString::Number(_) => "number",
    //         }).unwrap_or("no-code"),
    //         diagnostic.message
    //     );
    // }
    
    // Should have at least one syntax error
    let syntax_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("syntax-error".to_string())))
        .collect();
    
    assert!(!syntax_errors.is_empty(), "Should detect syntax error");
    
    // Verify that we have syntax errors to test
    
    // Test that errors don't span the entire file (which was the original problem)
    // Each error should be limited to a reasonable range
    for error in &syntax_errors {
        let line_span = error.range.end.line - error.range.start.line;
        let char_span = if error.range.start.line == error.range.end.line {
            error.range.end.character - error.range.start.character
        } else {
            100 // If it spans multiple lines, we'll check line span instead
        };
        
        // Error should not span more than 1 line or more than 50 characters on a single line
        assert!(line_span <= 1 && char_span <= 50, 
            "Error range too large: spans {} lines and {} chars. Range: {}:{}-{}:{}",
            line_span, char_span, 
            error.range.start.line, error.range.start.character,
            error.range.end.line, error.range.end.character);
    }
    
    // Check that at least one error is on line 4 (where "a;" is located)
    let has_error_on_line_4 = syntax_errors.iter().any(|error| {
        error.range.start.line == 4 // Line 4 is where "a;" is located
    });
    
    assert!(has_error_on_line_4, "Should have at least one error on line 4 where 'a;' is located");
    
    // At least one error should be small and precise
    let has_small_error = syntax_errors.iter().any(|error| {
        let line_span = error.range.end.line - error.range.start.line;
        let char_span = if error.range.start.line == error.range.end.line {
            error.range.end.character - error.range.start.character
        } else {
            100
        };
        line_span == 0 && char_span <= 10 // Small, precise error
    });
    
    assert!(has_small_error, "Should have at least one small, precise error range");
}

#[test]
fn test_missing_semicolon_detection() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with missing semicolon after background-color: red
    let content = r#"@import url("a.css");

a {
    background-color: red
    border-radius:10px;
}"#;
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Check if missing semicolon is detected
    let missing_semicolon_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("missing-semicolon".to_string())))
        .collect();
    
    assert!(!missing_semicolon_errors.is_empty(), "Should detect missing semicolon");
    
    // Verify the error is reported at the correct location (before 'border-radius')
    let has_border_radius_error = missing_semicolon_errors.iter().any(|error| {
        error.message.contains("border-radius")
    });
    
    assert!(has_border_radius_error, "Should detect missing semicolon before 'border-radius' property");
}

#[test]
fn test_nested_rule_missing_semicolon() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with missing semicolon before nested rule
    let content = r#"@import url("a.css");

a {
    background-color: red;
    border-radius:10px
    c{
        
    }
}"#;
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    
    
    // Should detect missing semicolon before nested rule, not pseudo-class error
    let missing_semicolon_errors: Vec<_> = results.iter()
        .filter(|e| e.message.contains("Missing semicolon after property"))
        .collect();
    
    assert!(!missing_semicolon_errors.is_empty(), "Should detect missing semicolon before nested rule");
    
    // Verify the specific error message
    let semicolon_error = &missing_semicolon_errors[0];
    assert!(semicolon_error.message.contains("border-radius"), "Should identify the correct property name");
    assert_eq!(semicolon_error.code, Some(NumberOrString::String("missing-semicolon".to_string())));
}

#[test]
fn test_multiline_error_limitation() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with a syntax error that might span multiple lines
    let content = r#"
.rule {
    background-color: red
    /* missing semicolon */
    color: blue;
}
"#;
    
    let tree = parser.parse(content, None);
    if let Some(tree) = tree {
        let results = diagnostics.analyze(&tree, content);
        
        // Check that any syntax errors don't span too many lines
        for diagnostic in results {
            if diagnostic.code == Some(tower_lsp::lsp_types::NumberOrString::String("syntax-error".to_string())) {
                let line_span = diagnostic.range.end.line - diagnostic.range.start.line;
                assert!(line_span <= 1, "Syntax error should not span more than 1 line, but spans {} lines", line_span);
            }
        }
    }
}

#[test]
fn test_invalid_unit_detection() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with invalid unit 'emeea'
    let content = r#"Button { 
     border-radius: 1emeea; 
 }"#;
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Debug: Print all diagnostics to see what we get
    println!("\nDiagnostics found:");
    for (i, diagnostic) in results.iter().enumerate() {
        println!("  {}: Line {}:{}-{}:{} - {} - {}", 
            i, 
            diagnostic.range.start.line, diagnostic.range.start.character,
            diagnostic.range.end.line, diagnostic.range.end.character,
            diagnostic.code.as_ref().map(|c| match c {
                tower_lsp::lsp_types::NumberOrString::String(s) => s.as_str(),
                tower_lsp::lsp_types::NumberOrString::Number(_) => "number",
            }).unwrap_or("no-code"),
            diagnostic.message
        );
    }
    
    // Check if invalid unit is detected
    let invalid_unit_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-unit".to_string())))
        .collect();
    
    // Check for any error that mentions the invalid unit 'emeea'
    let unit_related_errors: Vec<_> = results.iter()
        .filter(|d| d.message.contains("emeea"))
        .collect();
    
    println!("\nInvalid unit errors: {}", invalid_unit_errors.len());
    println!("Unit-related errors: {}", unit_related_errors.len());
    
    // The test should detect the invalid unit 'emeea'
    assert!(!invalid_unit_errors.is_empty() || !unit_related_errors.is_empty(), 
        "Should detect invalid unit 'emeea' in border-radius property. Found {} diagnostics total.", 
        results.len());
    
    // If we found invalid unit errors, verify they mention the correct unit
    if !invalid_unit_errors.is_empty() {
        let has_emeea_error = invalid_unit_errors.iter().any(|error| {
            error.message.contains("emeea")
        });
        assert!(has_emeea_error, "Should specifically identify 'emeea' as invalid unit");
    }
}

#[test]
fn test_valid_rgb_color() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { color: rgb(255,255,255); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should not have any color-related errors for valid rgb
    let color_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-color".to_string())))
        .collect();
    
    assert!(color_errors.is_empty(), "Valid rgb color should not produce errors. Found: {:?}", 
        color_errors.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_valid_rgba_color() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-color: rgba(255,255,255,1); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should not have any color-related errors for valid rgba
    let color_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-color".to_string())))
        .collect();
    
    assert!(color_errors.is_empty(), "Valid rgba color should not produce errors. Found: {:?}", 
        color_errors.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_valid_hex_color() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { -unity-background-image-tint-color: #ffffff; }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should not have any color-related errors for valid hex color
    let color_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-color".to_string())))
        .collect();
    
    assert!(color_errors.is_empty(), "Valid hex color #ffffff should not produce errors. Found: {:?}", 
        color_errors.iter().map(|e| &e.message).collect::<Vec<_>>());
}

// TODO: Implement proper type-based validation for hex colors
// This test will be re-enabled once we have proper property type validation
#[test]
fn test_invalid_hex_color_too_long() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { border-left-color: #ffffaabbcc; }";
    
    let tree = parser.parse(content, None).unwrap();
    

    
    let results = diagnostics.analyze(&tree, content);
    
    // Debug: Print all diagnostics
    println!("\nDiagnostics for #ffffaabbcc:");
    for (i, diagnostic) in results.iter().enumerate() {
        println!("  {}: {} - {}", i, 
            diagnostic.code.as_ref().map(|c| match c {
                tower_lsp::lsp_types::NumberOrString::String(s) => s.as_str(),
                tower_lsp::lsp_types::NumberOrString::Number(_) => "number",
            }).unwrap_or("no-code"),
            diagnostic.message
        );
    }
    
    // Should detect invalid hex color (too long - 10 characters)
    let color_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-color".to_string())))
        .collect();
    
    assert!(!color_errors.is_empty(), "Should detect invalid hex color #ffffaabbcc (too long). Found {} total diagnostics", results.len());
    
    let has_hex_error = color_errors.iter().any(|error| {
        error.message.contains("#ffffaabbcc")
    });
    assert!(has_hex_error, "Should specifically identify #ffffaabbcc as invalid hex color");
}

// TODO: Implement proper type-based validation for hex colors
// This test will be re-enabled once we have proper property type validation
#[test]
fn test_invalid_hex_color_invalid_chars() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { border-right-color: #ffaapp; }";
    
    let tree = parser.parse(content, None).unwrap();

    print_tree_to_stdout(tree.root_node(), content);
    
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect invalid hex color (contains 'p' which is not a hex digit)
    let color_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-color".to_string())))
        .collect();
    
    assert!(!color_errors.is_empty(), "Should detect invalid hex color #ffaaeepp (invalid characters). Found {} total diagnostics", results.len());
    
    let has_hex_error = color_errors.iter().any(|error| {
        error.message.contains("#ffaaeepp")
    });
    assert!(has_hex_error, "Should specifically identify #ffaaeepp as invalid hex color");
}