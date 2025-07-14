#!/usr/bin/env python3
import re
import sys
import os
from pathlib import Path

def add_format_field_to_properties(content):
    # Pattern to match PropertyInfo structs without format field
    pattern = r'(PropertyInfo \{\s*name: "([^"]+)",\s*description: "[^"]*",)(\s*documentation_url:)'
    
    def replacement(match):
        property_name = match.group(2)
        # Add format field with None as default
        return match.group(1) + '\n            format: None,' + match.group(3)
    
    return re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)

def find_project_root():
    """Find the project root by looking for Cargo.toml"""
    current_path = Path(__file__).resolve()
    
    # Start from the script's directory and go up
    for parent in [current_path.parent] + list(current_path.parents):
        if (parent / 'Cargo.toml').exists():
            return parent
    
    # If not found, raise an error
    raise FileNotFoundError("Could not find project root (Cargo.toml not found)")

if __name__ == '__main__':
    # Find project root dynamically
    project_root = find_project_root()
    print(f"Project root detected: {project_root}")
    
    file_path = project_root / 'src' / 'uss' / 'property_data.rs'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    updated_content = add_format_field_to_properties(content)
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(updated_content)
    
    print("Added format field to all PropertyInfo structs")