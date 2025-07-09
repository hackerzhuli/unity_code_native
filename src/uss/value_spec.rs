//! USS Value Specification Types
//!
//! Contains types for defining and validating USS property values,
//! including ValueType, ValueEntry, ValueFormat, and ValueSpec.

/// Basic value type that a property accepts
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone)]
pub struct ValueEntry {
    /// All valid value types for this entry
    pub types: Vec<ValueType>,
    /// Whether this entry is optional
    pub is_optional: bool,
}

/// Specific value format with exact type and count requirements
#[derive(Debug, Clone)]
pub struct ValueFormat {
    // this format should have these entries in this order(also, we allow optional entries)
    pub entries: Vec<ValueEntry>,
}

impl ValueFormat {
    /// Create a ValueFormat for a single value type
    pub fn single(value_type: ValueType) -> Self {
        Self {
            entries: vec![ValueEntry {
                types: vec![value_type],
                is_optional: false,
            }],
        }
    }

    /// Create a ValueFormat that accepts one of multiple value types
    pub fn one_of(value_types: Vec<ValueType>) -> Self {
        Self {
            entries: vec![ValueEntry {
                types: value_types,
                is_optional: false,
            }],
        }
    }

    /// Create a ValueFormat for keywords only
    pub fn keywords(keywords: &[&'static str]) -> Self {
        Self {
            entries: vec![ValueEntry {
                types: keywords.iter().map(|&k| ValueType::Keyword(k)).collect(),
                is_optional: false,
            }],
        }
    }

    /// Create a ValueFormat for a sequence of specific value types
    pub fn sequence(value_types: Vec<ValueType>) -> Self {
        Self {
            entries: value_types.into_iter().map(|vt| ValueEntry {
                types: vec![vt],
                is_optional: false,
            }).collect(),
        }
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
            for i in 0..count {
                entries.push(ValueEntry {
                    types: vec![value_type],
                    is_optional: i >= min_count,
                });
            }
            formats.push(ValueFormat { entries });
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
    pub fn multiple_formats(formats: Vec<ValueFormat>) -> Self {
        Self { formats }
    }
}