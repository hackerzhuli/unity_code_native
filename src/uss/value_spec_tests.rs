//! Tests for USS Value Specification Types
//!
//! Contains unit tests for ValueType, ValueEntry, ValueFormat, and ValueSpec.

use super::value_spec::*;
use crate::uss::parser::UssParser;
use crate::uss::definitions::UssDefinitions;

#[test]
fn test_value_format_is_match() {
    let mut parser = UssParser::new().unwrap();
    
    // Test 1: Length format with "100px"
    let content1 = "Button { width: 100px; }";
    let tree1 = parser.parse(content1, None).unwrap();
    let declaration1 = find_declaration_node(&tree1);
    
    let length_format = ValueFormat::single(ValueType::Length);
    assert!(length_format.is_match(declaration1, content1));
    
    // Test 2: Keyword format with "block"
    let content2 = "Button { display: block; }";
    let tree2 = parser.parse(content2, None).unwrap();
    let declaration2 = find_declaration_node(&tree2);
    
    let keyword_format = ValueFormat::single(ValueType::Keyword("block"));
    assert!(keyword_format.is_match(declaration2, content2));
}

#[test]
fn test_css_variables_handling() {
    let mut parser = UssParser::new().unwrap();

    // Create test formats using the correct ValueFormat constructors
    let length_format = ValueFormat::single(ValueType::Length);
    let two_length_format = ValueFormat::sequence(vec![ValueType::Length, ValueType::Length]);
    let keyword_format = ValueFormat::single(ValueType::Keyword("auto"));

    // Test case 1: Single var() should match single length format
    let content1 = "Button { width: var(--my-width); }";
    let tree1 = parser.parse(content1, None).unwrap();
    let declaration1 = find_declaration_node(&tree1);
    assert!(length_format.is_match(declaration1, content1), "var() should match single length format");

    // Test case 2: Length + var() should match two length format
    let content2 = "Button { margin: 10px var(--spacing); }";
    let tree2 = parser.parse(content2, None).unwrap();
    let declaration2 = find_declaration_node(&tree2);
    assert!(two_length_format.is_match(declaration2, content2), "length + var() should match two length format");

    // Test case 3: var() + Length should match two length format
    let content3 = "Button { margin: var(--spacing) 10px; }";
    let tree3 = parser.parse(content3, None).unwrap();
    let declaration3 = find_declaration_node(&tree3);
    assert!(two_length_format.is_match(declaration3, content3), "var() + length should match two length format");

    // Test case 4: Two var() should match two length format
    let content4 = "Button { margin: var(--top) var(--right); }";
    let tree4 = parser.parse(content4, None).unwrap();
    let declaration4 = find_declaration_node(&tree4);
    assert!(two_length_format.is_match(declaration4, content4), "two var() should match two length format");

    // Test case 5: var() should match keyword format (flexible)
    let content5 = "Button { display: var(--display-mode); }";
    let tree5 = parser.parse(content5, None).unwrap();
    let declaration5 = find_declaration_node(&tree5);
    assert!(keyword_format.is_match(declaration5, content5), "var() should match keyword format");
    
    // Test case 6: Invalid hex color split across nodes should fail
    let color_format = ValueFormat::single(ValueType::Color);
    let content6 = "Button { border-right-color: #ffaapp; }";
    let tree6 = parser.parse(content6, None).unwrap();
    let declaration6 = find_declaration_node(&tree6);
    assert!(!color_format.is_match(declaration6, content6), "Should reject invalid hex color #ffaapp");
}

#[test]
fn test_valid_rgb_rgba_colors() {
    let mut parser = UssParser::new().unwrap();
    let color_format = ValueFormat::single(ValueType::Color);
    
    // Test case 1: Valid rgb() function
    let content1 = "Button { color: rgb(255, 128, 0); }";
    let tree1 = parser.parse(content1, None).unwrap();
    let declaration1 = find_declaration_node(&tree1);
    assert!(color_format.is_match(declaration1, content1), "Should accept valid rgb() function");
    
    // Test case 2: Valid rgba() function
    let content2 = "Button { color: rgba(255, 128, 0, 0.5); }";
    let tree2 = parser.parse(content2, None).unwrap();
    let declaration2 = find_declaration_node(&tree2);
    assert!(color_format.is_match(declaration2, content2), "Should accept valid rgba() function");
    
    // Test case 3: Valid hsl() function
    let content3 = "Button { color: hsl(120, 100%, 50%); }";
    let tree3 = parser.parse(content3, None).unwrap();
    let declaration3 = find_declaration_node(&tree3);
    assert!(color_format.is_match(declaration3, content3), "Should accept valid hsl() function");
    
    // Test case 4: Valid hsla() function
    let content4 = "Button { color: hsla(120, 100%, 50%, 0.8); }";
    let tree4 = parser.parse(content4, None).unwrap();
    let declaration4 = find_declaration_node(&tree4);
    assert!(color_format.is_match(declaration4, content4), "Should accept valid hsla() function");
    
    // Test case 5: Valid hex color
    let content5 = "Button { color: #ff8000; }";
    let tree5 = parser.parse(content5, None).unwrap();
    let declaration5 = find_declaration_node(&tree5);
    assert!(color_format.is_match(declaration5, content5), "Should accept valid hex color");
    
    // Test case 6: Valid named color
    let content6 = "Button { color: red; }";
    let tree6 = parser.parse(content6, None).unwrap();
    let declaration6 = find_declaration_node(&tree6);
    assert!(color_format.is_match(declaration6, content6), "Should accept valid named color");
    
    // Test case 7: Invalid color function (not a color function)
    let content7 = "Button { color: calc(100px + 50px); }";
    let tree7 = parser.parse(content7, None).unwrap();
    let declaration7 = find_declaration_node(&tree7);
    assert!(!color_format.is_match(declaration7, content7), "Should reject non-color function");
}

#[test]
fn test_named_color_keywords() {
    let mut parser = UssParser::new().unwrap();
    let color_format = ValueFormat::single(ValueType::Color);
    
    // Test case 1: aliceblue - the specific color mentioned in the issue
    let content1 = "Button { border-right-color: aliceblue; }";
    let tree1 = parser.parse(content1, None).unwrap();
    let declaration1 = find_declaration_node(&tree1);
    assert!(color_format.is_match(declaration1, content1), "Should accept 'aliceblue' as valid color");
    
    // Test case 2: Other common named colors
    let content2 = "Button { color: cornflowerblue; }";
    let tree2 = parser.parse(content2, None).unwrap();
    let declaration2 = find_declaration_node(&tree2);
    assert!(color_format.is_match(declaration2, content2), "Should accept 'cornflowerblue' as valid color");
    
    // Test case 3: transparent keyword
    let content3 = "Button { background-color: transparent; }";
    let tree3 = parser.parse(content3, None).unwrap();
    let declaration3 = find_declaration_node(&tree3);
    assert!(color_format.is_match(declaration3, content3), "Should accept 'transparent' as valid color");
    
    // Test case 4: Invalid color name
    let content4 = "Button { color: notarealcolor; }";
    let tree4 = parser.parse(content4, None).unwrap();
    let declaration4 = find_declaration_node(&tree4);
    assert!(!color_format.is_match(declaration4, content4), "Should reject invalid color name");
    
    // Test case 5: Basic colors that should still work
    let content5 = "Button { color: white; }";
    let tree5 = parser.parse(content5, None).unwrap();
    let declaration5 = find_declaration_node(&tree5);
    assert!(color_format.is_match(declaration5, content5), "Should accept 'white' as valid color");
}

#[test]
fn test_value_format_invalid_cases() {
    let mut parser = UssParser::new().unwrap();
    
    // Test 1: Invalid unit for length (should reject "em")
    let content1 = "Button { width: 100em; }";
    let tree1 = parser.parse(content1, None).unwrap();
    let declaration1 = find_declaration_node(&tree1);
    let length_format = ValueFormat::single(ValueType::Length);
    assert!(!length_format.is_match(declaration1, content1));
    
    // Test 2: Wrong keyword (expecting "block" but got "inline")
    let content2 = "Button { display: inline; }";
    let tree2 = parser.parse(content2, None).unwrap();
    let declaration2 = find_declaration_node(&tree2);
    let block_keyword_format = ValueFormat::single(ValueType::Keyword("block"));
    assert!(!block_keyword_format.is_match(declaration2, content2));
    
    // Test 3: Type mismatch (expecting length but got string)
    let content3 = "Button { width: auto; }";
    let tree3 = parser.parse(content3, None).unwrap();
    let declaration3 = find_declaration_node(&tree3);
    let length_format2 = ValueFormat::single(ValueType::Length);
    assert!(!length_format2.is_match(declaration3, content3));
    
    // Test 4: Count mismatch (expecting 1 value but got 2)
    let content4 = "Button { width: 100px 200px; }";
    let tree4 = parser.parse(content4, None).unwrap();
    let declaration4 = find_declaration_node(&tree4);
    let single_length_format = ValueFormat::single(ValueType::Length);
    assert!(!single_length_format.is_match(declaration4, content4));
    
    // Test 5: Invalid time unit (should reject "sec")
    let content5 = "Button { transition-duration: 1sec; }";
    let tree5 = parser.parse(content5, None).unwrap();
    let declaration5 = find_declaration_node(&tree5);
    let time_format = ValueFormat::single(ValueType::Time);
    assert!(!time_format.is_match(declaration5, content5));
    
    // Test 6: Invalid angle unit (should reject "degrees")
    let content6 = "Button { rotate: 45degrees; }";
    let tree6 = parser.parse(content6, None).unwrap();
    let declaration6 = find_declaration_node(&tree6);
    let angle_format = ValueFormat::single(ValueType::Angle);
    assert!(!angle_format.is_match(declaration6, content6));
    
    // Test 7: Invalid integer (float when expecting integer)
    let content7 = "Button { z-index: 1.5; }";
    let tree7 = parser.parse(content7, None).unwrap();
    let declaration7 = find_declaration_node(&tree7);
    let integer_format = ValueFormat::single(ValueType::Integer);
    assert!(!integer_format.is_match(declaration7, content7));
    
    // Test 8: Invalid color format
    let content8 = "Button { color: notacolor; }";
    let tree8 = parser.parse(content8, None).unwrap();
    let declaration8 = find_declaration_node(&tree8);
    let color_format = ValueFormat::single(ValueType::Color);
    assert!(!color_format.is_match(declaration8, content8));
    
    // Test 9: Wrong asset function (expecting url/resource but got other)
    let content9 = "Button { background-image: calc(100px); }";
    let tree9 = parser.parse(content9, None).unwrap();
    let declaration9 = find_declaration_node(&tree9);
    let asset_format = ValueFormat::single(ValueType::Asset);
    assert!(!asset_format.is_match(declaration9, content9));
    
    // Test 10: Invalid property name format (contains invalid characters)
    let content10 = "Button { animation-property: width@height; }";
    let tree10 = parser.parse(content10, None).unwrap();
    let declaration10 = find_declaration_node(&tree10);
    let property_name_format = ValueFormat::single(ValueType::PropertyName);
    assert!(!property_name_format.is_match(declaration10, content10));
    
    // Test 11: Too few values (expecting 2 but got 1)
    let content11 = "Button { margin: 10px; }";
    let tree11 = parser.parse(content11, None).unwrap();
    let declaration11 = find_declaration_node(&tree11);
    let two_length_format = ValueFormat::sequence(vec![ValueType::Length, ValueType::Length]);
    assert!(!two_length_format.is_match(declaration11, content11));
    
    // Test 12: Too many values (expecting 1 but got 3)
    let content12 = "Button { width: 10px 20px 30px; }";
    let tree12 = parser.parse(content12, None).unwrap();
    let declaration12 = find_declaration_node(&tree12);
    let single_length_format2 = ValueFormat::single(ValueType::Length);
    assert!(!single_length_format2.is_match(declaration12, content12));
}

fn find_declaration_node(tree: &tree_sitter::Tree) -> tree_sitter::Node {
    let root = tree.root_node();
    let rule_set = root.child(0).unwrap(); // rule_set
    let block = rule_set.child(1).unwrap(); // block
    
    for i in 0..block.child_count() {
        if let Some(child) = block.child(i) {
            if child.kind() == "declaration" {
                return child;
            }
        }
    }
    
    panic!("Should find declaration node");
}