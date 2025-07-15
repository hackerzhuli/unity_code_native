//! Flexible format builder for generating multiple ValueFormat combinations
//!
//! This module provides utilities for creating flexible CSS value format specifications
//! that can handle optional entries and different orderings.

use crate::uss::value_spec::{ValueEntry, ValueFormat};

/// A flexible entry that can be optional
#[derive(Debug, Clone)]
pub struct FlexibleEntry {
    pub entry: ValueEntry,
    pub optional: bool,
}

/// Builder for creating flexible format specifications
#[derive(Debug, Clone)]
pub struct FlexibleFormatBuilder {
    entries: Vec<FlexibleEntry>,
    any_order: bool,
}

impl FlexibleFormatBuilder {
    /// Create a new flexible format builder with ordered entries
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            any_order: false,
        }
    }
    
    /// Create a new flexible format builder where entries can appear in any order
    pub fn any_order() -> Self {
        Self {
            entries: Vec::new(),
            any_order: true,
        }
    }
    
    /// Add a required entry
    pub fn required(mut self, entry: ValueEntry) -> Self {
        self.entries.push(FlexibleEntry {
            entry,
            optional: false,
        });
        self
    }
    
    /// Add an optional entry
    pub fn optional(mut self, entry: ValueEntry) -> Self {
        self.entries.push(FlexibleEntry {
            entry,
            optional: true,
        });
        self
    }
    
    /// Generate all possible ValueFormat combinations
    pub fn build(self) -> Vec<ValueFormat> {
        // Generate all combinations of optional entries
        let optional_combinations = self.generate_optional_combinations();
        
        let mut formats = if self.any_order {
            // Generate all permutations for each optional combination
            let mut formats = Vec::new();
            for combo in optional_combinations {
                let permutations = self.generate_permutations(&combo);
                for perm in permutations {
                    formats.push(ValueFormat {
                        entries: perm.into_iter().map(|fe| fe.entry).collect(),
                    });
                }
            }
            formats
        } else {
            // Keep original order, just handle optional entries
            optional_combinations.into_iter().map(|combo| ValueFormat {
                entries: combo.into_iter().map(|fe| fe.entry).collect(),
            }).collect()
        };
        
        // Filter out empty formats - we should never generate formats with no entries
        formats.retain(|format| !format.entries.is_empty());
        
        // Remove duplicate formats using the PartialEq implementation
        formats.sort();
        formats.dedup();
        
        formats
    }
    
    /// Generate all combinations of optional entries (include/exclude each optional entry)
    fn generate_optional_combinations(&self) -> Vec<Vec<FlexibleEntry>> {
        let optional_count = self.entries.iter().filter(|entry| entry.optional).count();
        let mut combinations = Vec::new();
        
        // Generate all 2^n combinations for optional entries
        for mask in 0..(1 << optional_count) {
            let mut combo = Vec::new();
            let mut optional_index = 0;
            
            for entry in &self.entries {
                if entry.optional {
                    if (mask >> optional_index) & 1 == 1 {
                        combo.push(entry.clone());
                    }
                    optional_index += 1;
                } else {
                    combo.push(entry.clone());
                }
            }
            
            combinations.push(combo);
        }
        
        combinations
    }
    
    /// Generate all permutations of entries
    fn generate_permutations(&self, entries: &[FlexibleEntry]) -> Vec<Vec<FlexibleEntry>> {
        if entries.is_empty() {
            return vec![Vec::new()];
        }
        
        if entries.len() == 1 {
            return vec![entries.to_vec()];
        }
        
        let mut permutations = Vec::new();
        
        for i in 0..entries.len() {
            let mut remaining = entries.to_vec();
            let current = remaining.remove(i);
            
            let sub_perms = self.generate_permutations(&remaining);
            
            for mut sub_perm in sub_perms {
                sub_perm.insert(0, current.clone());
                permutations.push(sub_perm);
            }
        }
        
        permutations
    }
}

impl Default for FlexibleFormatBuilder {
    fn default() -> Self {
        Self::new()
    }
}