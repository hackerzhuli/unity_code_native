use std::collections::HashMap;
use crate::uss::parser::UssParser;
use crate::uss::value::UssValue;
use crate::uss::variable_resolver::VariableResolver;

fn main() {
    let content = r#"
        :root {
            --primary-color: #ff0000;
            --text-color: var(--primary-color);
        }
    "#;
    
    let mut parser = UssParser::new().unwrap();
    let tree = parser.parse(content, None).unwrap();
    let mut resolver = VariableResolver::new();
    resolver.extract_and_resolve(tree.root_node(), content);
    
    let variables = resolver.get_variables();
    println!("Found {} variables:", variables.len());
    
    for (name, var_def) in variables {
        println!("Variable: {}", name);
        println!("  Values: {:?}", var_def.values);
        println!("  Status: {:?}", var_def.status);
    }
}