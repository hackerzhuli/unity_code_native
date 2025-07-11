//! Tests for USS Value Specification Types
//!
//! Contains unit tests for ValueType, ValueEntry, ValueFormat, and ValueSpec.

use crate::uss::value_spec::{ValueFormat, ValueType};
use crate::uss::value::UssValue;

#[test]
fn test_length_format() {
    let length_format = ValueFormat::single(ValueType::Length);
    
    // Valid length with px unit
    let values = vec![UssValue::Numeric { 
        value: 100.0, 
        unit: Some("px".to_string()), 
        has_fractional: false 
    }];
    assert!(length_format.is_match(&values));
    
    // Valid unitless 0
    let values = vec![UssValue::Numeric { 
        value: 0.0, 
        unit: None, 
        has_fractional: false 
    }];
    assert!(length_format.is_match(&values));
    
    // Valid length with % unit
    let values = vec![UssValue::Numeric { 
        value: 50.0, 
        unit: Some("%".to_string()), 
        has_fractional: false 
    }];
    assert!(length_format.is_match(&values));
    
    // Invalid - unitless non-zero value should not match Length
    let values = vec![UssValue::Numeric { 
        value: 10.0, 
        unit: None, 
        has_fractional: false 
    }];
    assert!(!length_format.is_match(&values));
    
    // Invalid - wrong type
    let values = vec![UssValue::String("not-a-length".to_string())];
    assert!(!length_format.is_match(&values));
}

#[test]
fn test_keyword_format() {
    let keyword_format = ValueFormat::single(ValueType::Keyword("block"));
    
    // Valid keyword
    let values = vec![UssValue::Identifier("block".to_string())];
    assert!(keyword_format.is_match(&values));
    
    // Invalid keyword
    let values = vec![UssValue::Identifier("inline".to_string())];
    assert!(!keyword_format.is_match(&values));
}

#[test]
fn test_color_format() {
    let color_format = ValueFormat::single(ValueType::Color);
    
    // Valid hex color
    let values = vec![UssValue::Color("#ff0000".to_string())];
    assert!(color_format.is_match(&values));
    
    // Valid named color
    let values = vec![UssValue::Color("red".to_string())];
    assert!(color_format.is_match(&values));
    
    // Valid rgb function
    let values = vec![UssValue::Color("rgb(255, 0, 0)".to_string())];
    assert!(color_format.is_match(&values));
    
    // Invalid - not a color
    let values = vec![UssValue::Identifier("notacolor".to_string())];
    assert!(!color_format.is_match(&values));
}

#[test]
fn test_variable_reference() {
    let length_format = ValueFormat::single(ValueType::Length);
    
    // Variable reference should match any format
    let values = vec![UssValue::VariableReference("my-width".to_string())];
    assert!(length_format.is_match(&values));
}

#[test]
fn test_sequence_format() {
    let two_length_format = ValueFormat::sequence(vec![ValueType::Length, ValueType::Length]);
    
    // Valid sequence
    let values = vec![
        UssValue::Numeric { value: 10.0, unit: Some("px".to_string()), has_fractional: false },
        UssValue::Numeric { value: 20.0, unit: Some("px".to_string()), has_fractional: false }
    ];
    assert!(two_length_format.is_match(&values));
    
    // Invalid - wrong count
    let values = vec![
        UssValue::Numeric { value: 10.0, unit: Some("px".to_string()), has_fractional: false }
    ];
    assert!(!two_length_format.is_match(&values));
}

#[test]
fn test_integer_format() {
    let integer_format = ValueFormat::single(ValueType::Integer);
    
    // Valid integer
    let values = vec![UssValue::Numeric { 
        value: 42.0, 
        unit: None, 
        has_fractional: false 
    }];
    assert!(integer_format.is_match(&values));
    
    // Invalid - has fractional part
    let values = vec![UssValue::Numeric { 
        value: 42.5, 
        unit: None, 
        has_fractional: true 
    }];
    assert!(!integer_format.is_match(&values));
}

#[test]
fn test_time_format() {
    let time_format = ValueFormat::single(ValueType::Time);
    
    // Valid time with seconds
    let values = vec![UssValue::Numeric { 
        value: 2.0, 
        unit: Some("s".to_string()), 
        has_fractional: false 
    }];
    assert!(time_format.is_match(&values));
    
    // Valid time with milliseconds
    let values = vec![UssValue::Numeric { 
        value: 500.0, 
        unit: Some("ms".to_string()), 
        has_fractional: false 
    }];
    assert!(time_format.is_match(&values));
    
    // Invalid - wrong unit
    let values = vec![UssValue::Numeric { 
        value: 2.0, 
        unit: Some("px".to_string()), 
        has_fractional: false 
    }];
    assert!(!time_format.is_match(&values));
}

#[test]
fn test_angle_format() {
    let angle_format = ValueFormat::single(ValueType::Angle);
    
    // Valid angle with degrees
    let values = vec![UssValue::Numeric { 
        value: 45.0, 
        unit: Some("deg".to_string()), 
        has_fractional: false 
    }];
    assert!(angle_format.is_match(&values));
    
    // Valid angle with radians
    let values = vec![UssValue::Numeric { 
        value: 1.57, 
        unit: Some("rad".to_string()), 
        has_fractional: true 
    }];
    assert!(angle_format.is_match(&values));
    
    // Invalid - wrong unit
    let values = vec![UssValue::Numeric { 
        value: 45.0, 
        unit: Some("px".to_string()), 
        has_fractional: false 
    }];
    assert!(!angle_format.is_match(&values));
}

#[test]
fn test_asset_format() {
    let asset_format = ValueFormat::single(ValueType::Asset);
    
    // Valid asset URL
    let values = vec![UssValue::Asset("url(\"path/to/image.png\")".to_string())];
    assert!(asset_format.is_match(&values));
    
    // Valid resource
    let values = vec![UssValue::Asset("resource(\"MyTexture\")".to_string())];
    assert!(asset_format.is_match(&values));
    
    // Invalid - not an asset
    let values = vec![UssValue::String("not-an-asset".to_string())];
    assert!(!asset_format.is_match(&values));
}

#[test]
fn test_string_format() {
    let string_format = ValueFormat::single(ValueType::String);
    
    // Valid string
    let values = vec![UssValue::String("hello world".to_string())];
    assert!(string_format.is_match(&values));
    
    // Invalid - not a string
    let values = vec![UssValue::Numeric { 
        value: 42.0, 
        unit: None, 
        has_fractional: false 
    }];
    assert!(!string_format.is_match(&values));
}

#[test]
fn test_property_name_format() {
    let property_format = ValueFormat::single(ValueType::PropertyName);
    
    // Valid property name
    let values = vec![UssValue::Identifier("width".to_string())];
    assert!(property_format.is_match(&values));
    
    // Invalid - not an identifier
    let values = vec![UssValue::String("not-a-property".to_string())];
    assert!(!property_format.is_match(&values));
}