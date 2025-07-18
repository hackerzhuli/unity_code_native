//! Tests for USS diagnostics functionality
use super::diagnostics::*;
use super::parser::UssParser;
use tower_lsp::lsp_types::NumberOrString;
use url::Url;

#[test]
fn test_import_statement_validation() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with valid import statement using url() function
    let valid_url_content = r#"@import url("styles.uss");

.button {
    color: red;
}"#;
    
    let tree = parser.parse(valid_url_content, None).unwrap();
    let results = diagnostics.analyze(&tree, valid_url_content);
    
    // Should not have any errors for valid url() import
    let import_errors: Vec<_> = results.iter()
        .filter(|d| d.message.contains("import") || d.code.as_ref().map_or(false, |c| {
            if let tower_lsp::lsp_types::NumberOrString::String(s) = c {
                s.contains("import")
            } else { false }
        }))
        .collect();
    
    assert!(import_errors.is_empty(), "Valid url() import statement should not produce errors. Found: {:?}", 
        import_errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    
    // Test case with valid import statement using string
    let valid_string_content = r#"@import "styles.uss";

.button {
    color: red;
}"#;
    
    let tree = parser.parse(valid_string_content, None).unwrap();
    let results = diagnostics.analyze(&tree, valid_string_content);
    
    // Should not have any errors for valid string import
    let import_errors: Vec<_> = results.iter()
        .filter(|d| d.message.contains("import") || d.code.as_ref().map_or(false, |c| {
            if let tower_lsp::lsp_types::NumberOrString::String(s) = c {
                s.contains("import")
            } else { false }
        }))
        .collect();
    
    assert!(import_errors.is_empty(), "Valid string import statement should not produce errors. Found: {:?}", 
        import_errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    
    // Test case with import statement missing .uss extension
    let css_import_content = r#"@import "styles.css";

.button {
    color: red;
}"#;
    
    let tree = parser.parse(css_import_content, None).unwrap();
    let results = diagnostics.analyze(&tree, css_import_content);
    
    // Should detect missing .uss extension warning
    let uss_extension_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(NumberOrString::String("missing-uss-extension".to_string())))
        .collect();
    
    assert!(!uss_extension_warnings.is_empty(), "Should detect missing .uss extension");
    
    // Test case with empty import path
    let empty_import_content = r#"@import "";

.button {
    color: red;
}"#;
    
    let tree = parser.parse(empty_import_content, None).unwrap();
    let results = diagnostics.analyze(&tree, empty_import_content);
    
    // Should detect some error for empty import path (UssValue validation handles this)
    println!("Empty import diagnostics: {:?}", results.iter().map(|d| (&d.code, &d.message)).collect::<Vec<_>>());
    
    // UssValue validation should produce an error for empty strings
     assert!(!results.is_empty() && results.iter().any(|d| 
         d.severity == Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR)
     ), "Should detect error for empty import path");
}

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
    
    // Should not have any errors for valid rgb color
    assert!(results.is_empty(), "Valid rgb color should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_valid_rgba_color() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-color: rgba(255,255,255,1); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should not have any errors for valid rgba color
    assert!(results.is_empty(), "Valid rgba color should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());
}



#[test]
fn test_valid_named_colors() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test multiple named colors
    let test_cases = vec![
        "Button { color: red; }",
        "Button { color: blue; }",
        "Button { color: green; }",
        "Button { color: white; }",
        "Button { color: black; }",
        "Button { color: transparent; }",
    ];
    
    for content in test_cases {
        let tree = parser.parse(content, None).unwrap();
        let results = diagnostics.analyze(&tree, content);
        
        // Should not have any errors for valid named colors
        assert!(results.is_empty(), "Valid named color should not produce any errors for '{}'. Found: {:?}", 
            content, results.iter().map(|e| &e.message).collect::<Vec<_>>());
    }
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
    
    // Should detect some error for invalid hex color (too long - 10 characters)
    assert!(!results.is_empty(), "Should detect some error for invalid hex color #ffffaabbcc (too long). Found {} total diagnostics", results.len());
}

// TODO: Implement proper type-based validation for hex colors
// This test will be re-enabled once we have proper property type validation
#[test]
fn test_invalid_hex_color_invalid_chars() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { border-right-color: #ffaapp; }";
    
    let tree = parser.parse(content, None).unwrap();
    
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect some error for invalid hex color (contains 'p' which is not a hex digit)
    assert!(!results.is_empty(), "Should detect some error for invalid hex color #ffaapp (invalid characters). Found {} total diagnostics", results.len());
}

#[test]
fn test_valid_url_function() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: url(\"path/to/image.png\"); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should not have any errors for valid url function
    assert!(results.is_empty(), "Valid url() function should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_valid_resource_function() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: resource(\"UI/Textures/button-bg\"); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should not have any errors for valid resource function
    assert!(results.is_empty(), "Valid resource() function should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_comments_in_declaration() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with comment after colon
    let content_after_colon = r#"Button { 
    color: /* comment */ red;
}"#;
    
    let tree = parser.parse(content_after_colon, None).unwrap();
    let results = diagnostics.analyze(&tree, content_after_colon);
    
    // Should not have any errors for valid declaration with comment
    assert!(results.is_empty(), "Valid declaration with comment after colon should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());
    
    // Test case with comment between values
    let content_between_values = r#"Button { 
    margin: 10px /* comment */ 20px;
}"#;
    
    let tree = parser.parse(content_between_values, None).unwrap();
    let results = diagnostics.analyze(&tree, content_between_values);
    
    // Should not have any errors for valid declaration with comment between values
    assert!(results.is_empty(), "Valid declaration with comment between values should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());

    // Test case with comment before semicolon
    let content_before_semicolon = r#"Button { 
    color: red /* comment */;
}"#;
    
    let tree = parser.parse(content_before_semicolon, None).unwrap();
    let results = diagnostics.analyze(&tree, content_before_semicolon);
    
    // Should not have any errors for valid declaration with comment before semicolon
    assert!(results.is_empty(), "Valid declaration with comment before semicolon should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());

    // Test case with comment before colon
    let content_before_semicolon = r#"Button { 
    color/* comment */: red;
}"#;
    
    let tree = parser.parse(content_before_semicolon, None).unwrap();
    let results = diagnostics.analyze(&tree, content_before_semicolon);
    
    assert!(results.is_empty(), "Valid declaration with comment before semicolon should not produce any errors. Found: {:?}",
        results.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_url_function_missing_argument() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: url(); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect missing argument error
    let missing_arg_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("url-invalid-argument-count".to_string())))
        .collect();
    
    assert!(!missing_arg_errors.is_empty(), "Should detect missing argument in url() function");
}

#[test]
fn test_resource_function_missing_argument() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: resource(); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect missing argument error
    let missing_arg_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-value".to_string())))
        .collect();
    
    assert!(!missing_arg_errors.is_empty(), "Should detect missing argument in resource() function");
}

#[test]
fn test_url_function_too_many_arguments() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: url(\"path1.png\", \"path2.png\"); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect too many arguments error
    let arg_count_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("url-invalid-argument-count".to_string())))
        .collect();
    
    assert!(!arg_count_errors.is_empty(), "Should detect too many arguments in url() function");
}

#[test]
fn test_resource_function_too_many_arguments() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: resource(\"path1\", \"path2\"); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect too many arguments error
    let arg_count_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-value".to_string())))
        .collect();
    
    assert!(!arg_count_errors.is_empty(), "Should detect too many arguments in resource() function");
}

#[test]
fn test_url_function_invalid_argument_type() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: url(123); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect invalid argument type error
    let type_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(NumberOrString::String("url-invalid-argument-type".to_string())))
        .collect();
    
    assert!(!type_errors.is_empty(), "Should detect invalid argument type in url() function");
}

#[test]
fn test_resource_function_invalid_argument_type() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: resource(123); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect invalid argument type error
    let type_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-value".to_string())))
        .collect();
    
    assert!(!type_errors.is_empty(), "Should detect invalid argument type in resource() function");
}

#[test]
fn test_resource_function_empty_path() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: resource(\"\"); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should detect empty resource path warning
    let empty_path_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-value".to_string())))
        .collect();
    
    assert!(!empty_path_warnings.is_empty(), "Should detect empty resource path in resource() function");
}

#[test]
fn test_url_function_with_single_quotes() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: url('path/to/image.png'); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should not have any errors for valid url function with single quotes
    assert!(results.is_empty(), "Valid url() function with single quotes should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_resource_function_with_single_quotes() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    let content = "Button { background-image: resource('UI/Textures/button-bg'); }";
    
    let tree = parser.parse(content, None).unwrap();
    let results = diagnostics.analyze(&tree, content);
    
    // Should not have any errors for valid resource function with single quotes
    assert!(results.is_empty(), "Valid resource() function with single quotes should not produce any errors. Found: {:?}", 
        results.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_variable_resolution_warning() {
    use crate::uss::variable_resolver::VariableResolver;
    
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // CSS content with variable definition that resolves to a valid value but wrong type for the property
    let content = ":root { --my-var: 10px; --my-var2: 20px; my-var100: 100px; }\nButton { color: var(--my-var) var(--my-var2) var(--my-var3); }";
    
    let tree = parser.parse(content, None).unwrap();
    
    // Create variable resolver and populate it from the parsed tree
    let mut variable_resolver = VariableResolver::new();
    variable_resolver.add_variables_from_tree(tree.root_node(), content);
    
    let (results, _url_references) = diagnostics.analyze_with_variables(&tree, content, None, Some(&variable_resolver));

    // Should generate a warning for the resolved variable value being invalid
    let warnings: Vec<_> = results.iter()
        .filter(|d| d.severity == Some(tower_lsp::lsp_types::DiagnosticSeverity::WARNING))
        .collect();
    
    assert!(!warnings.is_empty(), "Should generate a warning for invalid resolved variable value. Found {} total diagnostics", results.len());
    
    // Check that the warning message contains the expected information
    let warning = &warnings[0];
    assert!(warning.message.contains("color"), "Warning message should mention the property name 'color'");
    assert!(warning.message.contains("10px"), "Warning message should show the resolved value '10px'");
    assert!(warning.message.contains("--my-var = 10px"), "Warning message should list the resolved variable '--my-var = 10px'");
    assert!(warning.message.contains("likely invalid"), "Warning message should indicate the value is likely invalid");

    println!("{}", warning.message);
}

#[test]
fn test_url_collection() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // CSS content with import statements and url() functions
    let content = r#"@import "styles/base.uss";
@import url("styles/theme.uss");

Button {
    background-image: url("images/button.png");
    border-image: resource("UI/border");
}

.header {
    background-image: url('assets/header-bg.jpg');
}"#;
    
    let tree = parser.parse(content, None).unwrap();
    // use a more realistic url for the uss file
    let url = Url::parse("project:///Assets/UI/a.uss").unwrap();
    let (_diagnostics_result, url_references) = diagnostics.analyze_with_variables(&tree, content, Some(&url), None);

    // Check that we have the expected number of URLs (3: one string import, one url() import, two url() functions)
    // Note: String imports that are valid URLs will be collected
    let url_count = url_references.len();
    println!("Collected {} URL references", url_count);
    
    // Print collected URLs for verification
    for (i, url_ref) in url_references.iter().enumerate() {
        println!("URL {}: {} at range {:?}", i + 1, url_ref.url, url_ref.range);
    }
    
    // Should have 4 url collected
    assert_eq!(url_count, 4, "Should collect at least 2 URL references from url() functions, got {}", url_count);
}

#[test]
fn test_tag_selector_validation_with_valid_tags() {
    use std::collections::HashSet;
    
    // Create a hardcoded set of known Unity UI element names
    let mut uxml_class_names = HashSet::new();
    uxml_class_names.insert("Button".to_string());
    uxml_class_names.insert("Label".to_string());
    uxml_class_names.insert("TextField".to_string());
    uxml_class_names.insert("Slider".to_string());
    uxml_class_names.insert("Toggle".to_string());
    uxml_class_names.insert("Image".to_string());
    uxml_class_names.insert("ScrollView".to_string());
    
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with valid Unity UI elements
    let valid_content = r#"
Button {
    color: red;
}

Label {
    font-size: 14px;
}

TextField {
    background-color: white;
}

Slider {
    margin: 5px;
}
"#;
    
    let tree = parser.parse(valid_content, None).unwrap();
    let (results, _) = diagnostics.analyze_with_variables_and_classes(
        &tree,
        valid_content,
        None,
        None,
        Some(&uxml_class_names),
    );
    
    // Should not have any unknown tag warnings for valid Unity elements
    let unknown_tag_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("unknown-tag-selector".to_string())))
        .collect();
    
    assert!(unknown_tag_warnings.is_empty(), 
        "Valid Unity UI elements should not produce unknown tag warnings. Found: {:?}", 
        unknown_tag_warnings.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_tag_selector_validation_with_invalid_tags() {
    use std::collections::HashSet;
    
    // Create a hardcoded set of known Unity UI element names
    let mut uxml_class_names = HashSet::new();
    uxml_class_names.insert("Button".to_string());
    uxml_class_names.insert("Label".to_string());
    uxml_class_names.insert("TextField".to_string());
    uxml_class_names.insert("Slider".to_string());
    uxml_class_names.insert("Toggle".to_string());
    uxml_class_names.insert("Image".to_string());
    uxml_class_names.insert("ScrollView".to_string());
    
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with invalid/unknown tag selectors
    let invalid_content = r#"
/* Valid Unity UI elements */
Button {
    color: red;
}

Label {
    font-size: 14px;
}

/* Invalid/unknown tag selectors */
UnknownElement {
    background-color: blue;
}

CustomWidget {
    margin: 10px;
}

MyCustomControl {
    padding: 5px;
}
"#;
    
    let tree = parser.parse(invalid_content, None).unwrap();
    let (results, _) = diagnostics.analyze_with_variables_and_classes(
        &tree,
        invalid_content,
        None,
        None,
        Some(&uxml_class_names),
    );
    
    // Should detect unknown tag warnings for invalid elements
    let unknown_tag_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("unknown-tag-selector".to_string())))
        .collect();
    
    println!("Found {} unknown tag warnings", unknown_tag_warnings.len());
    for warning in &unknown_tag_warnings {
        println!("Warning: {}", warning.message);
    }
    
    // Should have exactly 3 unknown tag warnings (UnknownElement, CustomWidget, MyCustomControl)
    assert_eq!(unknown_tag_warnings.len(), 3, 
        "Should detect exactly 3 unknown tag selectors. Found: {:?}", 
        unknown_tag_warnings.iter().map(|e| &e.message).collect::<Vec<_>>());
    
    // Verify specific unknown tags are detected
    let warning_messages: Vec<String> = unknown_tag_warnings.iter()
        .map(|w| w.message.clone())
        .collect();
    
    assert!(warning_messages.iter().any(|msg| msg.contains("UnknownElement")), 
        "Should detect UnknownElement as unknown tag");
    assert!(warning_messages.iter().any(|msg| msg.contains("CustomWidget")), 
        "Should detect CustomWidget as unknown tag");
    assert!(warning_messages.iter().any(|msg| msg.contains("MyCustomControl")), 
        "Should detect MyCustomControl as unknown tag");
}

#[test]
fn test_tag_selector_validation_without_schema() {
    
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with tag selectors but no schema provided
    let content = r#"
Button {
    color: red;
}

UnknownElement {
    background-color: blue;
}
"#;
    
    let tree = parser.parse(content, None).unwrap();
    
    // Test without providing UXML class names (None)
    let (results, _) = diagnostics.analyze_with_variables_and_classes(
        &tree,
        content,
        None,
        None,
        None, // No class names provided
    );
    
    // Should not have any unknown tag warnings when no schema is provided
    let unknown_tag_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("unknown-tag-selector".to_string())))
        .collect();
    
    assert!(unknown_tag_warnings.is_empty(), 
        "Should not produce unknown tag warnings when no schema is provided. Found: {:?}", 
        unknown_tag_warnings.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_tag_selector_validation_case_sensitivity() {
    use std::collections::HashSet;
    
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Create a small set of known class names for testing
    let mut uxml_class_names = HashSet::new();
    uxml_class_names.insert("Button".to_string());
    uxml_class_names.insert("Label".to_string());
    // Note: lowercase versions are intentionally not included
    
    // Test case with case-sensitive tag selectors
    let case_content = r#"
/* Correct case - should be valid */
Button {
    color: red;
}

Label {
    font-size: 14px;
}

/* Wrong case - should be invalid */
button {
    background-color: blue;
}

label {
    margin: 10px;
}

BUTTON {
    padding: 5px;
}
"#;
    
    let tree = parser.parse(case_content, None).unwrap();
    let (results, _) = diagnostics.analyze_with_variables_and_classes(
        &tree,
        case_content,
        None,
        None,
        Some(&uxml_class_names),
    );
    
    // Should detect unknown tag warnings for incorrect case
    let unknown_tag_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("unknown-tag-selector".to_string())))
        .collect();
    
    println!("Found {} case-sensitivity warnings", unknown_tag_warnings.len());
    for warning in &unknown_tag_warnings {
        println!("Warning: {}", warning.message);
    }
    
    // Should have exactly 3 unknown tag warnings (button, label, BUTTON)
    assert_eq!(unknown_tag_warnings.len(), 3, 
        "Should detect exactly 3 case-sensitive unknown tag selectors. Found: {:?}", 
        unknown_tag_warnings.iter().map(|e| &e.message).collect::<Vec<_>>());
    
    // Verify specific case-incorrect tags are detected
    let warning_messages: Vec<String> = unknown_tag_warnings.iter()
        .map(|w| w.message.clone())
        .collect();
    
    assert!(warning_messages.iter().any(|msg| msg.contains("button")), 
        "Should detect lowercase 'button' as unknown tag");
    assert!(warning_messages.iter().any(|msg| msg.contains("label")), 
        "Should detect lowercase 'label' as unknown tag");
    assert!(warning_messages.iter().any(|msg| msg.contains("BUTTON")), 
        "Should detect uppercase 'BUTTON' as unknown tag");
}

#[test]
fn test_duplicate_property_detection() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with duplicate valid properties
    let duplicate_content = r#"
.button {
    color: red;
    background-color: blue;
    color: green;  /* duplicate */
    margin: 10px;
    background-color: yellow;  /* duplicate */
    padding: 5px;
}
"#;
    
    let tree = parser.parse(duplicate_content, None).unwrap();
    let results = diagnostics.analyze(&tree, duplicate_content);
    
    // Should detect duplicate property warnings
    let duplicate_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("duplicate-property".to_string())))
        .collect();
    
    // Should have exactly 4 duplicate warnings (2 for color + 2 for background-color)
    assert_eq!(duplicate_warnings.len(), 4, 
        "Should detect exactly 4 duplicate property warnings. Found: {:?}", 
        duplicate_warnings.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_duplicate_property_with_variables() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with duplicate CSS custom properties (variables)
    let variable_content = r#"
.button {
    --main-color: red;
    color: blue;
    --main-color: green;  /* duplicate variable */
    --secondary-color: yellow;
    --secondary-color: purple;  /* duplicate variable */
}
"#;
    
    let tree = parser.parse(variable_content, None).unwrap();
    let results = diagnostics.analyze(&tree, variable_content);
    
    // Should detect duplicate variable warnings
    let duplicate_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("duplicate-property".to_string())))
        .collect();
    
    println!("Found {} duplicate variable warnings", duplicate_warnings.len());
    for warning in &duplicate_warnings {
        println!("Warning: {}", warning.message);
    }
    
    // Should have exactly 4 duplicate warnings (2 for --main-color + 2 for --secondary-color)
    assert_eq!(duplicate_warnings.len(), 4, 
        "Should detect exactly 4 duplicate variable warnings. Found: {:?}", 
        duplicate_warnings.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_duplicate_property_ignores_invalid_properties() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with duplicate invalid properties (should be ignored)
    let invalid_content = r#"
.button {
    color: red;
    invalid-property: value1;
    invalid-property: value2;  /* duplicate but invalid */
    background-color: blue;
    another-invalid: test1;
    another-invalid: test2;  /* duplicate but invalid */
}
"#;
    
    let tree = parser.parse(invalid_content, None).unwrap();
    let results = diagnostics.analyze(&tree, invalid_content);
    
    // Should NOT detect duplicate warnings for invalid properties
    let duplicate_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("duplicate-property".to_string())))
        .collect();
    
    println!("Found {} duplicate warnings for invalid properties", duplicate_warnings.len());
    for warning in &duplicate_warnings {
        println!("Warning: {}", warning.message);
    }
    
    // Should have no duplicate warnings since invalid properties are ignored
    assert_eq!(duplicate_warnings.len(), 0, 
        "Should not detect duplicate warnings for invalid properties. Found: {:?}", 
        duplicate_warnings.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_no_duplicate_property_warnings_for_single_occurrence() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with no duplicate properties
    let single_content = r#"
.button {
    color: red;
    background-color: blue;
    margin: 10px;
    padding: 5px;
    font-size: 14px;
}
"#;
    
    let tree = parser.parse(single_content, None).unwrap();
    let results = diagnostics.analyze(&tree, single_content);
    
    // Should NOT detect any duplicate property warnings
    let duplicate_warnings: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("duplicate-property".to_string())))
        .collect();
    
    assert_eq!(duplicate_warnings.len(), 0, 
        "Should not detect duplicate warnings when properties appear only once. Found: {:?}", 
        duplicate_warnings.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_comma_separated_values_valid() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with valid comma-separated transition values
    let valid_content = r#"
.element {
    transition: opacity 0.3s ease-in-out, transform 0.5s linear;
    transition-property: opacity, transform, color;
    transition-duration: 0.3s, 0.5s, 1s;
}
"#;
    
    let tree = parser.parse(valid_content, None).unwrap();
    let results = diagnostics.analyze(&tree, valid_content);
    
    // Should not have any errors for valid comma-separated values
    let value_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-property-value".to_string())))
        .collect();
    
    assert!(value_errors.is_empty(), 
        "Valid comma-separated values should not produce errors. Found: {:?}", 
        value_errors.iter().map(|e| &e.message).collect::<Vec<_>>());
}

#[test]
fn test_comma_separated_values_invalid_segments() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with some invalid segments in comma-separated values
    let invalid_content = r#"
.element {
    transition: opacity 0.3s ease-in-out, invalid-value, transform 0.5s linear;
    transition-duration: 0.3s, invalid-time, 1s;
}
"#;
    
    let tree = parser.parse(invalid_content, None).unwrap();
    let results = diagnostics.analyze(&tree, invalid_content);
    
    // Should detect errors for invalid segments
    let value_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-property-value".to_string())))
        .collect();
    
    assert!(!value_errors.is_empty(), 
        "Should detect errors for invalid comma-separated value segments");
    
    // Check that error messages mention the specific invalid values
    let error_messages: Vec<String> = value_errors.iter()
        .map(|e| e.message.clone())
        .collect();
    
    let has_invalid_value_error = error_messages.iter()
        .any(|msg| msg.contains("invalid-value"));
    let has_invalid_time_error = error_messages.iter()
        .any(|msg| msg.contains("invalid-time"));
    
    assert!(has_invalid_value_error || has_invalid_time_error, 
        "Should detect specific invalid values in comma-separated segments. Found: {:?}", 
        error_messages);
}

#[test]
fn test_comma_not_allowed_property() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with comma in property that doesn't support multiple values
    let comma_content = r#"
.element {
    color: red, blue;
    background-color: green, yellow;
    font-size: 12px, 14px;
}
"#;
    
    let tree = parser.parse(comma_content, None).unwrap();
    let results = diagnostics.analyze(&tree, comma_content);
    
    // Should detect "unexpected-comma" errors
    let comma_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("unexpected-comma".to_string())))
        .collect();
    
    assert!(!comma_errors.is_empty(), 
        "Should detect unexpected comma errors for properties that don't support multiple values");
    
    // Should have exactly 3 comma errors (one for each property)
    assert_eq!(comma_errors.len(), 3, 
        "Should detect exactly 3 unexpected comma errors. Found: {:?}", 
        comma_errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    
    // Check that error messages mention the property names
    let error_messages: Vec<String> = comma_errors.iter()
        .map(|e| e.message.clone())
        .collect();
    
    assert!(error_messages.iter().any(|msg| msg.contains("color")), 
        "Should mention 'color' property in error message");
    assert!(error_messages.iter().any(|msg| msg.contains("background-color")), 
        "Should mention 'background-color' property in error message");
    assert!(error_messages.iter().any(|msg| msg.contains("font-size")), 
        "Should mention 'font-size' property in error message");
}

#[test]
fn test_empty_comma_segments() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with empty segments between commas
    let empty_content = r#"
.element {
    transition: opacity 0.3s, , transform 0.5s;
    transition-property: opacity, , color;
}
"#;
    
    let tree = parser.parse(empty_content, None).unwrap();
    let results = diagnostics.analyze(&tree, empty_content);
    
    // The parser should handle empty segments gracefully
    // We mainly want to ensure no crashes occur
    println!("Diagnostics for empty segments: {:?}", 
        results.iter().map(|d| (&d.code, &d.message)).collect::<Vec<_>>());
    
    // This test mainly ensures the code doesn't panic with empty segments
    // The specific behavior for empty segments may vary
}

#[test]
fn test_mixed_valid_invalid_comma_separated() {
    let diagnostics = UssDiagnostics::new();
    let mut parser = UssParser::new().unwrap();
    
    // Test case with mix of valid and invalid comma-separated values
    let mixed_content = r#"
.element {
    transition: opacity 0.3s ease-in-out, background-color invalid-duration, color 1s linear;
}
"#;
    
    let tree = parser.parse(mixed_content, None).unwrap();
    let results = diagnostics.analyze(&tree, mixed_content);
    
    // Should detect error only for the invalid segment
    let value_errors: Vec<_> = results.iter()
        .filter(|d| d.code == Some(tower_lsp::lsp_types::NumberOrString::String("invalid-property-value".to_string())))
        .collect();
    
    // Should have exactly one error for the invalid segment
    assert_eq!(value_errors.len(), 1, 
        "Should detect exactly one error for the invalid segment. Found: {:?}", 
        value_errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    
    // The error should mention the invalid duration
    let error_message = &value_errors[0].message;
    assert!(error_message.contains("transform") || error_message.contains("invalid-duration"), 
        "Error message should reference the invalid segment: {}", error_message);
}