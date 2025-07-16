use tower_lsp::lsp_types::{Position, Range};

use crate::uss::{formatter::UssFormatter, parser::UssParser};

fn create_parser() -> UssParser {
    UssParser::new().expect("Error creating USS parser")
}

#[test]
fn test_format_simple_css() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = ".test{color:red;background:blue;}";
    let tree = parser.parse(content, None).unwrap();

    let result = formatter.format_document(content, &tree);
    assert!(result.is_ok());

    let edits = result.unwrap();
    assert!(!edits.is_empty());
}

#[test]
fn test_skip_formatting_with_errors() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = ".test{color:red";
    let tree = parser.parse(content, None).unwrap();

    let result = formatter.format_document(content, &tree);
    assert!(result.is_ok());

    let edits = result.unwrap();
    assert!(edits.is_empty()); // Should skip formatting due to errors
}

#[test]
fn test_find_actual_format_range_mixed_content_line() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = "/* comment */ .class1 { color: red; } /* another comment */";
    let tree = parser.parse(content, None).unwrap();

    // Request range that includes the rule but has other content on same line
    let requested_range = Range {
        start: Position {
            line: 0,
            character: 14,
        }, // Start at .class1
        end: Position {
            line: 0,
            character: 37,
        }, // End after closing brace
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    assert!(result.is_none()); // Should reject due to mixed content on line
}

#[test]
fn test_find_actual_format_range_clean_line_boundaries() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = "\n.class1 {\n  color: red;\n}\n\n.class2 {\n  background: blue;\n}\n";
    let tree = parser.parse(content, None).unwrap();

    // Request range for first rule with clean line boundaries
    let requested_range = Range {
        start: Position {
            line: 1,
            character: 0,
        },
        end: Position {
            line: 3,
            character: 1,
        },
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    assert!(result.is_some());

    let actual_range = result.unwrap();
    assert_eq!(
        actual_range.start,
        Position {
            line: 1,
            character: 0
        }
    );
    assert_eq!(
        actual_range.end,
        Position {
            line: 3,
            character: 1
        }
    );
}

#[test]
fn test_find_actual_format_range_empty_selection() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = ".class1 { color: red; }\n.class2 { background: blue; }";
    let tree = parser.parse(content, None).unwrap();

    // Request range that doesn't contain any complete rules
    let requested_range = Range {
        start: Position {
            line: 0,
            character: 23,
        }, // Between rules
        end: Position {
            line: 1,
            character: 0,
        },
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    assert!(result.is_none()); // Should return None for empty selection
}

#[test]
fn test_find_actual_format_range_single_rule_subset() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content =
        ".class1 { color: red; }\n.class2 { background: blue; }\n.class3 { margin: 10px; }";
    let tree = parser.parse(content, None).unwrap();

    // Request range that covers only the middle rule completely
    let requested_range = Range {
        start: Position {
            line: 1,
            character: 0,
        },
        end: Position {
            line: 1,
            character: 29,
        },
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    assert!(result.is_some());

    let actual_range = result.unwrap();
    assert_eq!(
        actual_range.start,
        Position {
            line: 1,
            character: 0
        }
    );
    assert_eq!(
        actual_range.end,
        Position {
            line: 1,
            character: 29
        }
    );
}

#[test]
fn test_find_actual_format_range_start_middle_extract_complete_rules() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = "/* comment */ .class1 { color: red; }\n.class2 { background: blue; }\n.class3 { margin: 10px; }";
    let tree = parser.parse(content, None).unwrap();

    // Request range starting in the middle of first line but covering complete rules after
    let requested_range = Range {
        start: Position {
            line: 0,
            character: 5,
        }, // Middle of comment
        end: Position {
            line: 2,
            character: 25,
        },
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    // Should find the complete rules that start cleanly (tree-based logic)
    assert!(result.is_some());

    let actual_range = result.unwrap();
    assert_eq!(
        actual_range.start,
        Position {
            line: 1,
            character: 0
        }
    ); // Start of .class1
    assert_eq!(
        actual_range.end,
        Position {
            line: 2,
            character: 25
        }
    );
}

#[test]
fn test_find_actual_format_range_middle_of_first_rule_extract_others() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = ".class1 {\n  color: red;\n  background: blue;\n}\n.class2 { margin: 10px; }\n.class3 { padding: 5px; }";
    let tree = parser.parse(content, None).unwrap();

    // Request range starting in the middle of first rule's declaration block
    let requested_range = Range {
        start: Position {
            line: 1,
            character: 5,
        }, // Middle of "color: red;"
        end: Position {
            line: 5,
            character: 25,
        },
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    assert!(result.is_some());

    let actual_range = result.unwrap();
    // Should extract the two complete rules that start cleanly
    assert_eq!(
        actual_range.start,
        Position {
            line: 4,
            character: 0
        }
    );
    assert_eq!(
        actual_range.end,
        Position {
            line: 5,
            character: 25
        }
    );
}

#[test]
fn test_find_actual_format_range_end_middle_extract_complete_rules() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = ".class1 { color: red; }\n.class2 { background: blue; }\n.class3 {\n  margin: 10px;\n  padding: 5px;\n}";
    let tree = parser.parse(content, None).unwrap();

    // Request range ending in the middle of last rule's declaration block
    let requested_range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 4,
            character: 10,
        }, // Middle of "padding: 5px;"
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    assert!(result.is_some());

    let actual_range = result.unwrap();
    // Should extract the two complete rules that end cleanly
    assert_eq!(
        actual_range.start,
        Position {
            line: 0,
            character: 0
        }
    );
    assert_eq!(
        actual_range.end,
        Position {
            line: 1,
            character: 29
        }
    );
}

#[test]
fn test_find_actual_format_range_middle_selector_extract_inner_rules() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = ".very-long-class-name { color: red; }\n.class2 { background: blue; }\n.class3 { margin: 10px; }\n.another-long-name { padding: 5px; }";
    let tree = parser.parse(content, None).unwrap();

    // Request range starting in the middle of first selector and ending in middle of last
    let requested_range = Range {
        start: Position {
            line: 0,
            character: 10,
        }, // Middle of first selector
        end: Position {
            line: 3,
            character: 15,
        }, // Middle of last selector
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    assert!(result.is_some());

    let actual_range = result.unwrap();
    // Should extract the two complete middle rules
    assert_eq!(
        actual_range.start,
        Position {
            line: 1,
            character: 0
        }
    );
    assert_eq!(
        actual_range.end,
        Position {
            line: 2,
            character: 25
        }
    );
}

#[test]
fn test_find_actual_format_range_across_multiple_rules_partial() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content =
        ".class1 { color: red; }\n.class2 { background: blue; }\n.class3 { margin: 10px; }";
    let tree = parser.parse(content, None).unwrap();

    // Request range that starts in middle of first rule and ends in middle of last rule
    let requested_range = Range {
        start: Position {
            line: 0,
            character: 10,
        }, // Middle of first rule
        end: Position {
            line: 2,
            character: 15,
        }, // Middle of last rule
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    // Should only include the complete middle rule
    assert!(result.is_some());

    let actual_range = result.unwrap();
    assert_eq!(
        actual_range.start,
        Position {
            line: 1,
            character: 0
        }
    );
    assert_eq!(
        actual_range.end,
        Position {
            line: 1,
            character: 29
        }
    );
}

#[test]
fn test_find_actual_format_range_multiline_rule_with_complete_rules_inside() {
    let formatter = UssFormatter::new();
    let mut parser = create_parser();

    let content = ".outer {\n  color: red;\n}\n.class1 { background: blue; }\n.class2 { margin: 10px; }\n.final {\n  padding: 5px;\n}";
    let tree = parser.parse(content, None).unwrap();

    // Request range starting in the middle of first multiline rule and ending in middle of last
    let requested_range = Range {
        start: Position {
            line: 1,
            character: 5,
        }, // Middle of first rule's content
        end: Position {
            line: 6,
            character: 10,
        }, // Middle of last rule's content
    };

    let result = formatter
        .find_actual_format_range(content, &tree, requested_range)
        .unwrap();
    assert!(result.is_some());

    let actual_range = result.unwrap();
    // Should extract the two complete rules in the middle
    assert_eq!(
        actual_range.start,
        Position {
            line: 3,
            character: 0
        }
    );
    assert_eq!(
        actual_range.end,
        Position {
            line: 4,
            character: 25
        }
    );
}
