//! Example demonstrating the new occurrence range functionality
//! 
//! This example shows how to use the generalized FlexibleFormatBuilder
//! with custom occurrence ranges instead of just required/optional.

use crate::uss::flexible_format::FlexibleFormatBuilder;
use crate::uss::value_spec::{ValueEntry, ValueType};

fn main() {
    // Example 1: Traditional required/optional (still works)
    let traditional_formats = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::Length]))  // exactly 1 time
        .optional(ValueEntry::new(vec![ValueType::Color]))   // 0-1 times
        .build();
    
    println!("Traditional approach generates {} formats", traditional_formats.len());
    
    // Example 2: Custom occurrence ranges
    let custom_formats = FlexibleFormatBuilder::new()
        .with_occurrences(ValueEntry::new(vec![ValueType::Length]), 1, 3)  // 1-3 times
        .with_occurrences(ValueEntry::new(vec![ValueType::Color]), 0, 2)   // 0-2 times
        .build();
    
    println!("Custom ranges generate {} formats", custom_formats.len());
    
    // Example 3: Complex scenario with any_order
    let complex_formats = FlexibleFormatBuilder::any_order()
        .required(ValueEntry::new(vec![ValueType::Keyword("center")]))     // exactly 1
        .with_occurrences(ValueEntry::new(vec![ValueType::Length]), 2, 4)  // 2-4 times
        .optional(ValueEntry::new(vec![ValueType::Integer]))               // 0-1 times
        .build();
    
    println!("Complex any_order scenario generates {} formats", complex_formats.len());
    
    // Example 4: Multiple entries with same type but different ranges
    let multi_range_formats = FlexibleFormatBuilder::new()
        .with_occurrences(ValueEntry::new(vec![ValueType::Length]), 1, 2)  // primary lengths
        .with_occurrences(ValueEntry::new(vec![ValueType::Length]), 0, 1)  // optional length
        .with_occurrences(ValueEntry::new(vec![ValueType::Color]), 1, 1)   // required color
        .build();
    
    println!("Multi-range scenario generates {} formats", multi_range_formats.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_occurrence_range_examples() {
        // Test that our examples work correctly
        
        // Traditional: 1 required + 1 optional = 2 formats
        let traditional = FlexibleFormatBuilder::new()
            .required(ValueEntry::new(vec![ValueType::Length]))
            .optional(ValueEntry::new(vec![ValueType::Color]))
            .build();
        assert_eq!(traditional.len(), 2);
        
        // Custom: (1-3 lengths) × (0-2 colors) = 3 × 3 = 9 formats
        let custom = FlexibleFormatBuilder::new()
            .with_occurrences(ValueEntry::new(vec![ValueType::Length]), 1, 3)
            .with_occurrences(ValueEntry::new(vec![ValueType::Color]), 0, 2)
            .build();
        assert_eq!(custom.len(), 9);
        
        // Verify we can still use the convenience methods
        let mixed = FlexibleFormatBuilder::new()
            .required(ValueEntry::new(vec![ValueType::Length]))              // 1-1
            .optional(ValueEntry::new(vec![ValueType::Color]))               // 0-1  
            .with_occurrences(ValueEntry::new(vec![ValueType::Integer]), 2, 3) // 2-3
            .build();
        // 1 × 2 × 2 = 4 formats
        assert_eq!(mixed.len(), 4);
    }
}