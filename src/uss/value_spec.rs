//! USS Value Specification Types
//!
//! Contains types for defining and validating USS property values,
//! including ValueType, ValueEntry, ValueFormat, and ValueSpec.

use crate::uss::definitions::UssDefinitions;
use crate::uss::value::UssValue;
use crate::uss::constants::*;

/// Basic value type that a property accepts
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueType {
    /// Length values (px, %)
    /// 
    /// note: value 0 can be intepreted as length even without unit
    Length,
    /// Numeric values (unitless numbers)(can be integer or float)
    Number,
    /// Integer values (unitless numbers, must be integer)
    Integer,
    String,
    Time,
    /// Color values (hex, named colors, color functions)
    Color,
    /// Angle values (units are deg, rad, grad, turn)
    Angle,
    /// Keyword values from a predefined list (a USS Keyword)
    Keyword(&'static str),
    /// Asset references (url(), resource())
    Asset,
    /// property names in animation related property
    PropertyName
}

/// one value entry of property
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueEntry {
    /// All valid value types for this entry
    pub options: Vec<ValueType>
}



impl ValueEntry {
    pub fn new(types: Vec<ValueType>) -> Self {
        Self {
            options: types,
        }
    }

    pub fn keywords(keywords: &[&'static str]) -> Self {
        Self {
            options: keywords.iter().map(|&k| ValueType::Keyword(k)).collect(),
        }
    }

    fn is_keyword_only(&self) -> bool {
        return self.options.iter().all(|vt| matches!(vt, ValueType::Keyword(_)));
    }
    
    fn is_color_only(&self) -> bool {
        return self.options.iter().all(|vt| matches!(vt, ValueType::Color) || matches!(vt, ValueType::Keyword("initial")));
    }
}

/// Specific value format with exact type and count requirements
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueFormat {
    /// this format should have these entries in this order
    pub entries: Vec<ValueEntry>,
}

impl ValueFormat {
    /// Create a ValueFormat for a single value type
    pub fn single(value_type: ValueType) -> Self {
        Self {
            entries: vec![ValueEntry {
                options: vec![value_type],
            }],
        }
    }

    /// Create a ValueFormat that accepts one of multiple value types
    pub fn one_of(value_types: Vec<ValueType>) -> Self {
        Self {
            entries: vec![ValueEntry {
                options: value_types,
            }],
        }
    }

    /// Create a ValueFormat for keywords only
    pub fn keywords(keywords: &[&'static str]) -> Self {
        Self {
            entries: vec![ValueEntry {
                options: keywords.iter().map(|&k| ValueType::Keyword(k)).collect(),
            }],
        }
    }

    /// Create a ValueFormat for a sequence of specific value types
    pub fn sequence(value_types: Vec<ValueType>) -> Self {
        Self {
            entries: value_types.into_iter().map(|vt| ValueEntry {
                options: vec![vt],
            }).collect(),
        }
    }

    /// Check if a slice of UssValues matches this value format
    /// 
    /// Special handling for CSS variables (var(--name)):
    /// - var() calls are treated as wildcards that can match 0-n values
    /// - If any var() is present, we validate non-var values and return true if they could potentially match
    pub fn is_match(&self, values: &[UssValue], definitions: &UssDefinitions) -> bool {
        // Check for CSS variables (var() calls)
        let has_variables = values.iter().any(|value| matches!(value, UssValue::VariableReference(_)));
        
        if has_variables {
            // With variables present, we use flexible matching
            // Separate variable and non-variable values
            let non_var_values: Vec<_> = values.iter()
                .filter(|value| !matches!(value, UssValue::VariableReference(_)))
                .collect();
            
            // If we have more non-variable values than format entries, it's definitely invalid
            if non_var_values.len() > self.entries.len() {
                return false;
            }
            
            // Try to match non-variable values against format entries
            // We'll be permissive here - if any reasonable assignment could work, return true
            if non_var_values.is_empty() {
                // Only variables - always valid since variables can match anything
                return true;
            }
            
            // Check if non-variable values can match any subset of our format entries
            return self.can_match_subset_values(&non_var_values, definitions);
        } else {
            // No variables - use strict matching
            if values.len() != self.entries.len() {
                return false;
            }

            // Validate each value against corresponding entry
            for (value, entry) in values.iter().zip(&self.entries) {
                if !self.is_value_valid(value, entry, definitions) {
                    return false;
                }
            }

            true
        }
    }

    /// Check if a subset of values can match any subset of format entries
    /// This is used for flexible matching when CSS variables are present
    fn can_match_subset_values(&self, values: &[&UssValue], definitions: &UssDefinitions) -> bool {
        // If we have no format entries, only variables can be valid
        if self.entries.is_empty() {
            return values.is_empty();
        }

        // Try to find a valid assignment of values to format entries
        // For simplicity, we'll check if each value can match at least one format entry
        for value in values {
            let mut found_match = false;
            for entry in &self.entries {
                if self.is_value_valid(value, entry, definitions) {
                    found_match = true;
                    break;
                }
            }
            if !found_match {
                return false;
            }
        }
        
        true
    }

    /// Check if a UssValue matches any of the types in a ValueEntry
    fn is_value_valid(&self, value: &UssValue, entry: &ValueEntry, definitions: &UssDefinitions) -> bool {
        for value_type in &entry.options {
            if self.is_value_of_type(value, *value_type, definitions) {
                return true;
            }
        }
        false
    }

    /// Check if a UssValue matches a specific ValueType
    fn is_value_of_type(&self, value: &UssValue, value_type: ValueType, definitions: &UssDefinitions) -> bool {
        match value {
            UssValue::Numeric { unit: Some(unit_str), has_fractional: _, .. } => {
                  // Check if this numeric value matches the expected type based on unit
                  match value_type {
                      ValueType::Length => unit_str == UNIT_PX || unit_str == UNIT_PERCENT,
                       ValueType::Time => unit_str == UNIT_S || unit_str == UNIT_MS,
                      ValueType::Angle => matches!(unit_str.as_str(), UNIT_DEG | UNIT_RAD | UNIT_GRAD | UNIT_TURN),
                      ValueType::Number => false, // Numbers with units don't match Number type
                      ValueType::Integer => false, // Integers with units don't match Integer type
                      _ => false,
                  }
              },
            UssValue::Numeric { unit: None, has_fractional, value, .. } => {
                 // Unitless numeric values can match Number, Integer, or Length (only for 0)
                 match value_type {
                     ValueType::Integer => !has_fractional, // Integers cannot have fractional parts
                     ValueType::Number => true, // Numbers can be fractional or not
                     ValueType::Length => *value == 0.0, // Length can only be unitless if it's exactly 0
                     _ => false,
                 }
             },
            UssValue::String(_) => matches!(value_type, ValueType::String),
            UssValue::Color(_) => matches!(value_type, ValueType::Color),
            UssValue::Identifier(keyword) => {
                match value_type {
                    ValueType::Keyword(expected) => keyword == expected,
                    ValueType::PropertyName => true, // Any identifier can be a property name
                    ValueType::Color => definitions.is_valid_color_keyword(keyword), // Check if identifier is a valid color name
                    _ => false,
                }
            },
            UssValue::Url(_) => matches!(value_type, ValueType::Asset),
            UssValue::Resource(_) => matches!(value_type, ValueType::Asset),
            // PropertyName is handled as Identifier
            // UssValue::PropertyName doesn't exist - property names use Identifier
            UssValue::VariableReference(_) => true, // Variables can match any type
        }
    }
    
    fn is_keyword_only(&self) -> bool {
        return self.entries.len() == 1 && self.entries[0].is_keyword_only();
    }
    
    fn is_color_only(&self) -> bool {
        return self.entries.len() == 1 && self.entries[0].is_color_only();
    }
}

/// Complete value specification for a property
#[derive(Debug, Clone)]
pub struct ValueSpec {
    /// All possible value formats for this property
    pub formats: Vec<ValueFormat>
}

impl ValueSpec {
    /// Create a ValueSpec for a single value type
    pub fn single(value_type: ValueType) -> Self {
        Self {
            formats: vec![ValueFormat::single(value_type)],
        }
    }

    /// Create a ValueSpec for color values (hex, keywords, rgb, rgba)
    pub fn color() -> Self {
        Self::single(ValueType::Color)
    }

    /// Create a ValueSpec for shorthand properties (1-4 values of the same type)
    pub fn repeat(value_type: ValueType, min_count: usize, max_count: usize) -> Self {
        let mut formats = Vec::new();
        
        for count in min_count..=max_count {
            let mut entries = Vec::new();
            for _i in 0..count {
                entries.push(ValueEntry {
                    options: vec![value_type],
                });
            }
            formats.push(ValueFormat { 
                entries,
            });
        }
        
        Self { formats }
    }

    /// Create a ValueSpec that accepts one of multiple value types
    pub fn one_of(value_types: Vec<ValueType>) -> Self {
        Self {
            formats: vec![ValueFormat::one_of(value_types)],
        }
    }

    /// Create a ValueSpec for keywords only
    pub fn keywords(keywords: &[&'static str]) -> Self {
        Self {
            formats: vec![ValueFormat::keywords(keywords)],
        }
    }

    /// Create a ValueSpec for a sequence of specific value types
    pub fn sequence(value_types: Vec<ValueType>) -> Self {
        Self {
            formats: vec![ValueFormat::sequence(value_types)],
        }
    }

    /// Create a ValueSpec with multiple possible formats
    pub fn new(formats: Vec<ValueFormat>) -> Self {
        Self { formats }
    }
    
    pub(crate) fn is_keyword_only(&self) -> bool {
        self.formats.len() == 1 && self.formats[0].is_keyword_only()
    }
    
    pub(crate) fn is_color_only(&self) -> bool {
        return self.formats.len() == 1 && self.formats[0].is_color_only();
    }
    
    /// Check if this value spec is a single format with a single entry
    pub(crate) fn is_single_format_and_entry(&self) -> bool {
        self.formats.len() == 1 && self.formats[0].entries.len() == 1
    }
}