#!/usr/bin/env python3
import re
import sys
import os
from pathlib import Path

def parse_markdown_formats(md_content):
    """Parse the markdown file to extract property formats"""
    property_formats = {}
    lines = md_content.split('\n')
    in_css_example = False
    
    for i, line in enumerate(lines):
        trimmed = line.strip()
        
        # Check if we're entering or leaving a code block
        if trimmed.startswith('```'):
            continue
        
        # Detect CSS example blocks
        if line.startswith('    '):
            code_line = line[4:]  # Remove the 4-space indentation
            if code_line.strip().startswith('.') and '{' in code_line:
                in_css_example = True
            if code_line.strip() == '}' and in_css_example:
                in_css_example = False
                continue
        
        # Skip if we're in a CSS example block
        if in_css_example:
            continue
        
        # Check if this line is indented (indicating it's in a code block)
        if line.startswith('    ') and line.strip():
            code_line = line[4:]  # Remove the 4-space indentation
            
            # Skip comments
            if code_line.strip().startswith('/*') or code_line.strip().startswith('*/'):
                continue
            
            # Look for property definitions (property: format)
            if ':' in code_line:
                colon_pos = code_line.find(':')
                property_name = code_line[:colon_pos].strip()
                format_spec = code_line[colon_pos + 1:].strip()
                
                # Skip empty property names or format specs
                if not property_name or not format_spec:
                    continue
                
                # Skip CSS selectors and other non-property lines
                if ('.' in property_name or '#' in property_name or 
                    ' ' in property_name or property_name.startswith('@')):
                    continue
                
                # Skip lines that look like CSS values
                if (format_spec.endswith(';') and 
                    ('px' in format_spec or 'red' in format_spec or 
                     'blue' in format_spec or '0.5' in format_spec or
                     format_spec == 'initial;' or len(format_spec) < 20)):
                    continue
                
                # Only process lines that look like actual format specifications
                if '<' not in format_spec and '|' not in format_spec:
                    continue
                
                property_formats[property_name] = format_spec
    
    return property_formats

def update_property_data(file_path, property_formats):
    """Update the property_data.rs file with the correct formats"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # For each property that needs updating
    for prop_name, format_spec in property_formats.items():
        # Escape special regex characters in the property name
        escaped_prop_name = re.escape(prop_name)
        
        # Pattern to find the PropertyInfo struct for this property
        pattern = rf'(PropertyInfo \{{\s*name: "{escaped_prop_name}",\s*description: "[^"]*",\s*)format: None,(\s*documentation_url:)'
        
        # Replacement with the correct format
        escaped_format = format_spec.replace('\\', '\\\\\\\\')
        escaped_format = escaped_format.replace('"', '\\"')
        replacement = rf'\1format: Some("{escaped_format}"),\2'
        
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"Updated {len(property_formats)} properties with their formats")

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
    
    # Read the markdown file
    md_file_path = project_root / 'Assets' / 'data' / 'USS_property_format_6.0.md'
    with open(md_file_path, 'r', encoding='utf-8') as f:
        md_content = f.read()
    
    # Parse property formats
    property_formats = parse_markdown_formats(md_content)
    print(f"Found {len(property_formats)} property formats in markdown")
    
    # Update the property data file
    property_data_path = project_root / 'src' / 'uss' / 'property_data.rs'
    update_property_data(str(property_data_path), property_formats)
    
    print("Property data file updated successfully!")