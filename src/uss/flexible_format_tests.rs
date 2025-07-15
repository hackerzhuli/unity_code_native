//! Tests for FlexibleFormatBuilder
//!
//! Contains unit tests for the flexible format generation functionality.

use crate::uss::flexible_format::FlexibleFormatBuilder;
use crate::uss::value_spec::{ValueEntry, ValueType, ValueFormat};

#[test]
fn test_builder_basic_required_entries() {
    let formats = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .required(ValueEntry::new(vec![ValueType::Color]))
        .build();
    
    // Should generate exactly one format with both entries in order
    assert_eq!(formats.len(), 1);
    assert_eq!(formats[0].entries.len(), 2);
    
    // Check the format structure
    let expected_format = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    assert_eq!(formats[0], expected_format);
}

#[test]
fn test_builder_optional_entries() {
    let formats = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .optional(ValueEntry::new(vec![ValueType::Color]))
        .build();
    
    // Should generate two formats: with and without optional entry
    assert_eq!(formats.len(), 2);
    
    // Expected formats
    let format_without_optional = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::LengthOrPercentage])]
    };
    let format_with_optional = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    
    assert!(formats.contains(&format_without_optional));
    assert!(formats.contains(&format_with_optional));
}

#[test]
fn test_builder_multiple_optional_entries() {
    let formats = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .optional(ValueEntry::new(vec![ValueType::Color]))
        .optional(ValueEntry::new(vec![ValueType::Integer]))
        .build();
    
    // Should generate 4 formats: all combinations of optional entries
    assert_eq!(formats.len(), 4);
    
    // Expected formats
    let format_required_only = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::LengthOrPercentage])]
    };
    let format_with_color = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    let format_with_integer = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Integer])
        ]
    };
    let format_with_both = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::Integer])
        ]
    };
    
    assert!(formats.contains(&format_required_only));
    assert!(formats.contains(&format_with_color));
    assert!(formats.contains(&format_with_integer));
    assert!(formats.contains(&format_with_both));
}

#[test]
fn test_builder_any_order() {
    let formats = FlexibleFormatBuilder::any_order()
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .required(ValueEntry::new(vec![ValueType::Color]))
        .build();
    
    // Should generate 2 formats: both possible orders
    assert_eq!(formats.len(), 2);
    
    // Expected formats
    let format_length_first = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    let format_color_first = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::LengthOrPercentage])
        ]
    };
    
    assert!(formats.contains(&format_length_first));
    assert!(formats.contains(&format_color_first));
}

#[test]
fn test_builder_any_order_with_optional() {
    let formats = FlexibleFormatBuilder::any_order()
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .required(ValueEntry::new(vec![ValueType::Color]))
        .optional(ValueEntry::new(vec![ValueType::Integer]))
        .build();
    
    // Should generate multiple formats for all combinations and permutations
    // With 2 required + 1 optional, we get:
    // - 2 permutations of required entries (without optional)
    // - 6 permutations with optional entry in different positions
    assert_eq!(formats.len(), 8); // 2 + 6 = 8 total formats
    
    // Test that we have some expected formats
    let format_length_color = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    let format_color_length = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::LengthOrPercentage])
        ]
    };
    let format_with_optional_at_end = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::Integer])
        ]
    };
    
    assert!(formats.contains(&format_length_color));
    assert!(formats.contains(&format_color_length));
    assert!(formats.contains(&format_with_optional_at_end));
}

#[test]
fn test_builder_empty() {
    let formats = FlexibleFormatBuilder::new().build();
    
    // Should generate no formats when no entries are added
    assert_eq!(formats.len(), 0);
}

#[test]
fn test_builder_only_optional() {
    let formats = FlexibleFormatBuilder::new()
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .optional(ValueEntry::new(vec![ValueType::Color]))
        .build();
    
    // Should generate 3 formats: all combinations of optional entries (excluding empty)
    assert_eq!(formats.len(), 3);
    
    // Expected formats (no empty format)
    let format_length_only = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::LengthOrPercentage])]
    };
    let format_color_only = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::Color])]
    };
    let format_both = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    
    assert!(formats.contains(&format_length_only));
    assert!(formats.contains(&format_color_only));
    assert!(formats.contains(&format_both));
    
    // Verify empty format is NOT generated
    let format_empty = ValueFormat {
        entries: vec![]
    };
    assert!(!formats.contains(&format_empty), "Should NOT contain empty format");
}

#[test]
fn test_builder_keywords() {
    let formats = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::Keyword("center")]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .build();
    
    // Should generate 2 formats: with and without optional length
    assert_eq!(formats.len(), 2);
    
    // Expected formats
    let format_keyword_only = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::Keyword("center")])]
    };
    let format_keyword_with_length = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::Keyword("center")]),
            ValueEntry::new(vec![ValueType::LengthOrPercentage])
        ]
    };
    
    assert!(formats.contains(&format_keyword_only));
    assert!(formats.contains(&format_keyword_with_length));
}

#[test]
fn test_builder_all_optional_no_empty_entries() {
    let formats = FlexibleFormatBuilder::new()
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .optional(ValueEntry::new(vec![ValueType::Color]))
        .optional(ValueEntry::new(vec![ValueType::Integer]))
        .build();
    
    // Should generate 7 formats: all combinations of 3 optional entries (2^3 - 1, excluding empty)
    assert_eq!(formats.len(), 7);
    
    // Verify no format is empty and no entry has empty options
    for format in &formats {
        assert!(!format.entries.is_empty(), "Format should not be empty");
        for entry in &format.entries {
            assert!(!entry.options.is_empty(), "Entry should not have empty options");
        }
    }
    
    // Verify we have the expected formats (no empty format)
    let format_length_only = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::LengthOrPercentage])]
    };
    let format_color_only = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::Color])]
    };
    let format_all_three = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::Integer])
        ]
    };
    
    assert!(formats.contains(&format_length_only));
    assert!(formats.contains(&format_color_only));
    assert!(formats.contains(&format_all_three));
    
    // Verify empty format is NOT generated
    let format_empty = ValueFormat { entries: vec![] };
    assert!(!formats.contains(&format_empty), "Should NOT contain empty format");
}

#[test]
fn test_builder_deduplication() {
    // Create a scenario that could generate duplicates without deduplication
    // Using any_order with identical entries should deduplicate
    let formats = FlexibleFormatBuilder::any_order()
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage])) // Identical entry
        .build();
    
    // Should generate only 1 format since both entries are identical
    // Without deduplication, this would generate 2 identical formats
    assert_eq!(formats.len(), 1);
    
    let expected_format = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::LengthOrPercentage])
        ]
    };
    
    assert!(formats.contains(&expected_format));
    
    // Test with optional entries that could create duplicates
    let formats2 = FlexibleFormatBuilder::any_order()
        .optional(ValueEntry::new(vec![ValueType::Color]))
        .optional(ValueEntry::new(vec![ValueType::Color])) // Identical optional entry
        .build();
    
    // Should generate 3 unique formats: empty (filtered out), single color, double color
    // Without deduplication, this could generate more duplicates
    assert_eq!(formats2.len(), 2); // Single color and double color (empty filtered out)
    
    let format_single = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::Color])]
    };
    let format_double = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    
    assert!(formats2.contains(&format_single));
    assert!(formats2.contains(&format_double));
}

#[test]
fn test_builder_with_occurrences() {
    let formats = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .range(ValueEntry::new(vec![ValueType::Color]), 2, 3)
        .build();
    
    // Should generate 2 formats: with 2 colors and with 3 colors
    assert_eq!(formats.len(), 2);
    
    // Expected formats
    let format_with_2_colors = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    let format_with_3_colors = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    
    assert!(formats.contains(&format_with_2_colors));
    assert!(formats.contains(&format_with_3_colors));
}

#[test]
fn test_builder_with_occurrences_zero_min() {
    let formats = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .range(ValueEntry::new(vec![ValueType::Color]), 0, 2)
        .build();
    
    // Should generate 3 formats: with 0, 1, and 2 colors
    assert_eq!(formats.len(), 3);
    
    // Expected formats
    let format_no_color = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::LengthOrPercentage])]
    };
    let format_one_color = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    let format_two_colors = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    
    assert!(formats.contains(&format_no_color));
    assert!(formats.contains(&format_one_color));
    assert!(formats.contains(&format_two_colors));
}

#[test]
fn test_builder_multiple_occurrence_ranges() {
    let formats = FlexibleFormatBuilder::new()
        .range(ValueEntry::new(vec![ValueType::LengthOrPercentage]), 1, 2)
        .range(ValueEntry::new(vec![ValueType::Color]), 0, 1)
        .build();
    
    // Should generate 6 formats: (1 or 2 lengths) × (0 or 1 colors)
    assert_eq!(formats.len(), 4);
    
    // Expected formats
    let format_1l_0c = ValueFormat {
        entries: vec![ValueEntry::new(vec![ValueType::LengthOrPercentage])]
    };
    let format_1l_1c = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    let format_2l_0c = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::LengthOrPercentage])
        ]
    };
    let format_2l_1c = ValueFormat {
        entries: vec![
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::LengthOrPercentage]),
            ValueEntry::new(vec![ValueType::Color])
        ]
    };
    
    assert!(formats.contains(&format_1l_0c));
    assert!(formats.contains(&format_1l_1c));
    assert!(formats.contains(&format_2l_0c));
    assert!(formats.contains(&format_2l_1c));
}

#[test]
#[should_panic(expected = "min_occurrences must be <= max_occurrences")]
fn test_builder_invalid_occurrence_range() {
    FlexibleFormatBuilder::new()
        .range(ValueEntry::new(vec![ValueType::LengthOrPercentage]), 3, 1); // min > max should panic
}