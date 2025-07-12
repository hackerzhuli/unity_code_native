//! Tests for USS Value Specification Types
//!
//! Contains unit tests for ValueType, ValueEntry, ValueFormat, and ValueSpec.

use crate::uss::value_spec::{ValueFormat, ValueType};
use crate::uss::value::UssValue;
use crate::uss::definitions::UssDefinitions;
use crate::uss::constants::*;

#[test]
fn test_length_format() {
    let length_format = ValueFormat::single(ValueType::Length);
    let definitions = UssDefinitions::new();
    
    // Valid length with px unit
    let values = vec![UssValue::Numeric { 
        value: 100.0, 
        unit: Some(UNIT_PX.to_string()), 
        has_fractional: false 
    }];
    assert!(length_format.is_match(&values, &definitions));
    
    // Valid unitless 0
    let values = vec![UssValue::Numeric { 
        value: 0.0, 
        unit: None, 
        has_fractional: false 
    }];
    assert!(length_format.is_match(&values, &definitions));
    
    // Valid length with % unit
    let values = vec![UssValue::Numeric { 
        value: 50.0, 
        unit: Some(UNIT_PERCENT.to_string()), 
        has_fractional: false 
    }];
    assert!(length_format.is_match(&values, &definitions));
    
    // Invalid - unitless non-zero value should not match Length
    let values = vec![UssValue::Numeric { 
        value: 10.0, 
        unit: None, 
        has_fractional: false 
    }];
    assert!(!length_format.is_match(&values, &definitions));
    
    // Invalid - wrong type
    let values = vec![UssValue::String("not-a-length".to_string())];
    assert!(!length_format.is_match(&values, &definitions));
}

#[test]
fn test_keyword_format() {
    let keyword_format = ValueFormat::single(ValueType::Keyword("block"));
    let definitions = UssDefinitions::new();
    
    // Valid keyword
    let values = vec![UssValue::Identifier("block".to_string())];
    assert!(keyword_format.is_match(&values, &definitions));
    
    // Invalid keyword
    let values = vec![UssValue::Identifier("inline".to_string())];
    assert!(!keyword_format.is_match(&values, &definitions));
}

#[test]
fn test_color_format() {
    let color_format = ValueFormat::single(ValueType::Color);
    let definitions = UssDefinitions::new();
    
    // Valid hex color
    let values = vec![UssValue::Color(crate::uss::color::Color::new_rgb(255, 0, 0))];
    assert!(color_format.is_match(&values, &definitions));
    
    // Valid named color
    let values = vec![UssValue::Color(crate::uss::color::Color::new_rgb(255, 0, 0))];
    assert!(color_format.is_match(&values, &definitions));
    
    // Valid rgb function
    let values = vec![UssValue::Color(crate::uss::color::Color::new_rgb(255, 0, 0))];
    assert!(color_format.is_match(&values, &definitions));
    
    // Invalid - not a color
    let values = vec![UssValue::Identifier("notacolor".to_string())];
    assert!(!color_format.is_match(&values, &definitions));
}

#[test]
fn test_variable_reference() {
    let length_format = ValueFormat::single(ValueType::Length);
    let definitions = UssDefinitions::new();
    
    // Variable reference should match any format
    let values = vec![UssValue::VariableReference("my-width".to_string())];
    assert!(length_format.is_match(&values, &definitions));
}

#[test]
fn test_sequence_format() {
    let two_length_format = ValueFormat::sequence(vec![ValueType::Length, ValueType::Length]);
    let definitions = UssDefinitions::new();
    
    // Valid sequence
    let values = vec![
        UssValue::Numeric { value: 10.0, unit: Some(UNIT_PX.to_string()), has_fractional: false },
        UssValue::Numeric { value: 20.0, unit: Some(UNIT_PX.to_string()), has_fractional: false }
    ];
    assert!(two_length_format.is_match(&values, &definitions));
    
    // Invalid - wrong count
    let values = vec![
        UssValue::Numeric { value: 10.0, unit: Some(UNIT_PX.to_string()), has_fractional: false }
    ];
    assert!(!two_length_format.is_match(&values, &definitions));
}

#[test]
fn test_integer_format() {
    let integer_format = ValueFormat::single(ValueType::Integer);
    let definitions = UssDefinitions::new();
    
    // Valid integer
    let values = vec![UssValue::Numeric { 
        value: 42.0, 
        unit: None, 
        has_fractional: false 
    }];
    assert!(integer_format.is_match(&values, &definitions));
    
    // Invalid - has fractional part
    let values = vec![UssValue::Numeric { 
        value: 42.5, 
        unit: None, 
        has_fractional: true 
    }];
    assert!(!integer_format.is_match(&values, &definitions));
}

#[test]
fn test_time_format() {
    let time_format = ValueFormat::single(ValueType::Time);
    let definitions = UssDefinitions::new();
    
    // Valid time with seconds
    let values = vec![UssValue::Numeric { 
        value: 2.0, 
        unit: Some(UNIT_S.to_string()), 
        has_fractional: false 
    }];
    assert!(time_format.is_match(&values, &definitions));
    
    // Valid time with milliseconds
    let values = vec![UssValue::Numeric { 
        value: 500.0, 
        unit: Some(UNIT_MS.to_string()), 
        has_fractional: false 
    }];
    assert!(time_format.is_match(&values, &definitions));
    
    // Invalid - wrong unit
    let values = vec![UssValue::Numeric { 
        value: 2.0, 
        unit: Some(UNIT_PX.to_string()), 
        has_fractional: false 
    }];
    assert!(!time_format.is_match(&values, &definitions));
}

#[test]
fn test_angle_format() {
    let angle_format = ValueFormat::single(ValueType::Angle);
    let definitions = UssDefinitions::new();
    
    // Valid angle with degrees
    let values = vec![UssValue::Numeric { 
        value: 45.0, 
        unit: Some(UNIT_DEG.to_string()), 
        has_fractional: false 
    }];
    assert!(angle_format.is_match(&values, &definitions));
    
    // Valid angle with radians
    let values = vec![UssValue::Numeric { 
        value: 1.57, 
        unit: Some(UNIT_RAD.to_string()), 
        has_fractional: true 
    }];
    assert!(angle_format.is_match(&values, &definitions));
    
    // Invalid - wrong unit
    let values = vec![UssValue::Numeric { 
        value: 45.0, 
        unit: Some(UNIT_PX.to_string()), 
        has_fractional: false 
    }];
    assert!(!angle_format.is_match(&values, &definitions));
}

#[test]
fn test_asset_format() {
    let asset_format = ValueFormat::single(ValueType::Asset);
    let definitions = UssDefinitions::new();
    
    // Valid asset URL
    let values = vec![UssValue::Url(url::Url::parse("file:///path/to/image.png").unwrap())];
    assert!(asset_format.is_match(&values, &definitions));
    
    // Valid resource
    let values = vec![UssValue::Resource(url::Url::parse("resource://MyTexture").unwrap())];
    assert!(asset_format.is_match(&values, &definitions));
    
    // Invalid - not an asset
    let values = vec![UssValue::String("not-an-asset".to_string())];
    assert!(!asset_format.is_match(&values, &definitions));
}

#[test]
fn test_string_format() {
    let string_format = ValueFormat::single(ValueType::String);
    let definitions = UssDefinitions::new();
    
    // Valid string
    let values = vec![UssValue::String("hello world".to_string())];
    assert!(string_format.is_match(&values, &definitions));
    
    // Invalid - not a string
    let values = vec![UssValue::Numeric { 
        value: 42.0, 
        unit: None, 
        has_fractional: false 
    }];
    assert!(!string_format.is_match(&values, &definitions));
}

#[test]
fn test_property_name_format() {
    let property_format = ValueFormat::single(ValueType::PropertyName);
    let definitions = UssDefinitions::new();
    
    // Valid property name
    let values = vec![UssValue::Identifier("width".to_string())];
    assert!(property_format.is_match(&values, &definitions));
    
    // Invalid - not an identifier
    let values = vec![UssValue::String("not-a-property".to_string())];
    assert!(!property_format.is_match(&values, &definitions));
}