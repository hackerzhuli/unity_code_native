use crate::uss::{parser::UssParser, value::UssValue, variable_resolver::{VariableResolutionStatus, VariableResolver}};

fn create_test_tree(content: &str) -> Option<tree_sitter::Tree> {
    let mut parser = UssParser::new().unwrap();
    parser.parse(content, None)
}

#[test]
fn test_variable_extraction() {
    let content = r#":root {
    --primary-color: #ff0000;
    --secondary-color: #00ff00;
    --margin: 10px;
}"#;
    
    let tree = create_test_tree(content).unwrap();
    let mut resolver = VariableResolver::new();
    resolver.extract_and_resolve(tree.root_node(), content);
    
    let variables = resolver.get_variables();
    assert_eq!(variables.len(), 3);
    assert!(variables.contains_key("primary-color"));
    assert!(variables.contains_key("secondary-color"));
    assert!(variables.contains_key("margin"));
}

#[test]
fn test_variable_resolution_simple() {
    let content = r#"
            :root {
                --primary-color: #ff0000;
                --text-color: var(--primary-color);
            }
        "#;
    
    let tree = create_test_tree(content).unwrap();
    let mut resolver = VariableResolver::new();
    resolver.extract_and_resolve(tree.root_node(), content);
    
    let variables = resolver.get_variables();
    // Check that we have the expected variables
    assert_eq!(variables.len(), 2);
    
    let primary_var = resolver.get_variable("primary-color").unwrap();
    assert!(matches!(primary_var.status, VariableResolutionStatus::Resolved(_)));
    
    let text_var = resolver.get_variable("text-color").unwrap();
    assert!(matches!(text_var.status, VariableResolutionStatus::Resolved(_)));
    
    // Check that the resolved value is correct
    if let VariableResolutionStatus::Resolved(values) = &text_var.status {
        assert_eq!(values.len(), 1);
        assert!(matches!(values[0], UssValue::Color(_)));
    }
}

#[test]
fn test_variable_resolution_circular() {
    let content = r#"
            :root {
                --color-a: var(--color-b);
                --color-b: var(--color-a);
            }
        "#;
    
    let tree = create_test_tree(content).unwrap();
    let mut resolver = VariableResolver::new();
    resolver.extract_and_resolve(tree.root_node(), content);
    
    let color_a = resolver.get_variable("color-a").unwrap();
    let color_b = resolver.get_variable("color-b").unwrap();
    
    // Both should be unresolved due to circular dependency
    assert!(matches!(color_a.status, VariableResolutionStatus::Unresolved));
    assert!(matches!(color_b.status, VariableResolutionStatus::Unresolved));
}

#[test]
fn test_variable_resolution_ambiguous() {
    let content = r#"
            .class1 {
                --primary-color: #ff0000;
            }
            .class2 {
                --primary-color: #00ff00;
            }
        "#;
    
    let tree = create_test_tree(content).unwrap();
    let mut resolver = VariableResolver::new();
    resolver.extract_and_resolve(tree.root_node(), content);
    
    let primary_var = resolver.get_variable("primary-color").unwrap();
    assert!(matches!(primary_var.status, VariableResolutionStatus::Ambiguous));
}

#[test]
fn test_variable_invalidation() {
    let content = r#"
            :root {
                --primary-color: #ff0000;
                --text-color: var(--primary-color);
            }
        "#;
    
    let tree = create_test_tree(content).unwrap();
    let mut resolver = VariableResolver::new();
    resolver.extract_and_resolve(tree.root_node(), content);
    
    // Initially both should be resolved
    let primary_var = resolver.get_variable("primary-color").unwrap();
    assert!(matches!(primary_var.status, VariableResolutionStatus::Resolved(_)));
    
    let text_var = resolver.get_variable("text-color").unwrap();
    assert!(matches!(text_var.status, VariableResolutionStatus::Resolved(_)));
    
    // Invalidate primary-color
    resolver.invalidate_variable("primary-color");
    
    // primary-color should be unresolved, text-color should also be unresolved
    let primary_var = resolver.get_variable("primary-color").unwrap();
    assert!(matches!(primary_var.status, VariableResolutionStatus::Unresolved));
    
    let text_var = resolver.get_variable("text-color").unwrap();
    assert!(matches!(text_var.status, VariableResolutionStatus::Unresolved));
}

#[test]
fn test_complex_variable_dependencies() {
    let content = r#"
            .test-var2 {
                --something-even-bigger: var(--something) row column var(--something-else);
            }

            .test-var {
                --something: 1px 2 #aabbcc;
                --something-else: var(--something) 1px;
            }
        "#;
    
    let tree = create_test_tree(content).unwrap();
    let mut resolver = VariableResolver::new();
    resolver.extract_and_resolve(tree.root_node(), content);
    
    let variables = resolver.get_variables();
    assert_eq!(variables.len(), 3);
    
    // Check that --something is resolved (no dependencies)
    let something_var = resolver.get_variable("something").unwrap();
    assert!(matches!(something_var.status, VariableResolutionStatus::Resolved(_)));
    if let VariableResolutionStatus::Resolved(values) = &something_var.status {
        assert_eq!(values.len(), 3); // 1px, 2, #aabbcc
    }
    
    // Check that --something-else is resolved (depends on --something)
    let something_else_var = resolver.get_variable("something-else").unwrap();
    assert!(matches!(something_else_var.status, VariableResolutionStatus::Resolved(_)));
    if let VariableResolutionStatus::Resolved(values) = &something_else_var.status {
        assert_eq!(values.len(), 4); // 1px, 2, #aabbcc (from var(--something)), 1px
    }
    
    // Check that --something-even-bigger is resolved (depends on both --something and --something-else)
      let something_bigger_var = resolver.get_variable("something-even-bigger").unwrap();
      assert!(matches!(something_bigger_var.status, VariableResolutionStatus::Resolved(_)));
      if let VariableResolutionStatus::Resolved(values) = &something_bigger_var.status {
          // Should contain: 1px, 2, #aabbcc (from var(--something)), "row", "column", 1px, 2, #aabbcc, 1px (from var(--something-else))
          // Expected: [1px, 2, #aabbcc] + ["row"] + ["column"] + [1px, 2, #aabbcc, 1px] = 9 values total
          assert_eq!(values.len(), 9);
          
          // Check exact values in order with equality assertions
          assert_eq!(values[0], UssValue::Numeric { value: 1.0, unit: Some("px".to_string()), has_fractional: false });
          assert_eq!(values[1], UssValue::Numeric { value: 2.0, unit: None, has_fractional: false });
          assert_eq!(values[2], UssValue::Color("#aabbcc".to_string()));
          assert_eq!(values[3], UssValue::Identifier("row".to_string()));
          assert_eq!(values[4], UssValue::Identifier("column".to_string()));
          assert_eq!(values[5], UssValue::Numeric { value: 1.0, unit: Some("px".to_string()), has_fractional: false });
          assert_eq!(values[6], UssValue::Numeric { value: 2.0, unit: None, has_fractional: false });
          assert_eq!(values[7], UssValue::Color("#aabbcc".to_string()));
          assert_eq!(values[8], UssValue::Numeric { value: 1.0, unit: Some("px".to_string()), has_fractional: false });
      }
}
