//! Flexible format builder for generating multiple ValueFormat combinations
//!
//! This module provides utilities for creating flexible CSS value format specifications
//! that can handle optional entries and different orderings.
//! So we don't have to write all cases manually, just let the builder build cases when the format is more complex.

use crate::uss::value_spec::{ValueEntry, ValueFormat};

/// A flexible entry that can appear a specified number of times
#[derive(Debug, Clone)]
pub struct FlexibleEntry {
    pub entry: ValueEntry,
    pub min_occurrences: usize,
    pub max_occurrences: usize,
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
    
    /// Add a required entry (appears exactly once)
    pub fn required(mut self, entry: ValueEntry) -> Self {
        self.entries.push(FlexibleEntry {
            entry,
            min_occurrences: 1,
            max_occurrences: 1,
        });
        self
    }
    
    /// Add an optional entry (appears 0 or 1 times)
    pub fn optional(mut self, entry: ValueEntry) -> Self {
        self.entries.push(FlexibleEntry {
            entry,
            min_occurrences: 0,
            max_occurrences: 1,
        });
        self
    }
    
    /// Add an entry with custom occurrence range
    pub fn range(mut self, entry: ValueEntry, min: usize, max: usize) -> Self {
        assert!(min <= max, "min_occurrences must be <= max_occurrences");
        self.entries.push(FlexibleEntry {
            entry,
            min_occurrences: min,
            max_occurrences: max,
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
    
    /// Generate all combinations of entries with their occurrence ranges
    fn generate_optional_combinations(&self) -> Vec<Vec<FlexibleEntry>> {
        if self.entries.is_empty() {
            return vec![Vec::new()];
        }
        
        self.generate_combinations_recursive(&self.entries, 0)
    }
    
    /// Recursively generate all valid combinations of entries with their occurrence counts
    fn generate_combinations_recursive(&self, entries: &[FlexibleEntry], index: usize) -> Vec<Vec<FlexibleEntry>> {
        if index >= entries.len() {
            return vec![Vec::new()];
        }
        
        let current_entry = &entries[index];
        let remaining_combinations = self.generate_combinations_recursive(entries, index + 1);
        let mut all_combinations = Vec::new();
        
        // For each possible occurrence count of the current entry
        for count in current_entry.min_occurrences..=current_entry.max_occurrences {
            for remaining_combo in &remaining_combinations {
                let mut new_combo = Vec::new();
                
                // Add the current entry 'count' times
                for _ in 0..count {
                    new_combo.push(current_entry.clone());
                }
                
                // Add the remaining entries
                new_combo.extend(remaining_combo.iter().cloned());
                
                all_combinations.push(new_combo);
            }
        }
        
        all_combinations
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