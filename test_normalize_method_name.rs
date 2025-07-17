// Simple test program to demonstrate normalize_method_name_string function

use std::path::Path;
use std::env;

// Add the project src to the module path
mod src {
    pub mod cs {
        pub mod compile_utils;
    }
}

use src::cs::compile_utils::normalize_method_name_string;

fn main() {
    println!("=== Method Name Normalization Examples ===");
    
    let test_cases = vec![
        "Method()",
        "Method ( int , string )",
        "Method(System.Int32, System.String)",
        "GenericMethod{T}(T, System.Collections.Generic.List{T})",
        "ProcessData(ref System.Int32, in System.String, out System.Boolean)",
        "ComplexMethod( System.Collections.Generic.Dictionary{System.String, System.Collections.Generic.List{System.Int32}} )",
    ];
    
    for (i, test_case) in test_cases.iter().enumerate() {
        let normalized = normalize_method_name_string(test_case);
        println!("{}. Input:  {}", i + 1, test_case);
        println!("   Output: {}", normalized);
        println!();
    }
}