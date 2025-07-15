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

def parse_unity_examples(md_content):
    """Parse Unity USS documentation to extract property examples"""
    property_examples = {}
    lines = md_content.split('\n')
    current_property = None
    in_examples_section = False
    examples_buffer = []
    
    for i, line in enumerate(lines):
        # Look for property sections (### property-name or ## property-name)
        if line.startswith('###') or line.startswith('##'):
            # Save previous property examples if any
            if current_property and examples_buffer:
                property_examples[current_property] = '\n'.join(examples_buffer).strip()
                examples_buffer = []
            
            # Extract property name from heading
            heading_text = line.strip('#').strip()
            if '`' in heading_text:
                # Extract property name from backticks
                match = re.search(r'`([^`]+)`', heading_text)
                if match:
                    current_property = match.group(1)
                    in_examples_section = False
            else:
                current_property = None
                in_examples_section = False
        
        # Look for "Examples" section
        elif line.strip() == '**Examples**' or line.strip().lower() == 'examples':
            in_examples_section = True
            examples_buffer = []
        
        # Collect example code blocks
        elif in_examples_section and current_property:
            if line.startswith('    ') and line.strip():
                # This is an indented code line
                code_line = line[4:]  # Remove indentation
                if ':' in code_line and not code_line.strip().startswith('//'):
                    # Clean up the example - remove property name if it's duplicated
                    if code_line.strip().startswith(current_property + ':'):
                        # Extract just the value part
                        value_part = code_line.strip()[len(current_property + ':'):].strip()
                        examples_buffer.append(f"{current_property}: {value_part}")
                    else:
                        examples_buffer.append(code_line.strip())
            elif line.strip() and not line.startswith(' '):
                # End of examples section
                if examples_buffer:
                    property_examples[current_property] = '\n'.join(examples_buffer).strip()
                    examples_buffer = []
                in_examples_section = False
    
    # Handle last property
    if current_property and examples_buffer:
        property_examples[current_property] = '\n'.join(examples_buffer).strip()
    
    return property_examples

def parse_mozilla_examples(md_content):
    """Parse Mozilla CSS documentation to extract property examples"""
    property_examples = {}
    lines = md_content.split('\n')
    current_property = None
    in_syntax_section = False
    in_examples_section = False
    in_try_it_section = False
    in_css_block = False
    examples_buffer = []
    
    for i, line in enumerate(lines):
        original_line = line
        line_stripped = line.strip()
        
        # Check for property headings (look for lines followed by ===)
        if i + 1 < len(lines) and lines[i + 1].strip().startswith('==='):
            # Save previous property examples if any
            if current_property and examples_buffer:
                property_examples[current_property] = '\n'.join(examples_buffer).strip()
                examples_buffer = []
            
            current_property = line_stripped.lower()
            in_syntax_section = False
            in_examples_section = False
            in_try_it_section = False
            in_css_block = False
            continue
            
        # Check for relevant sections
        if '[try it]' in line_stripped.lower():
            in_try_it_section = True
            in_syntax_section = False
            in_examples_section = False
            continue
        elif '[syntax]' in line_stripped.lower():
            in_syntax_section = True
            in_examples_section = False
            in_try_it_section = False
            continue
        elif '[examples]' in line_stripped.lower():
            in_examples_section = True
            in_syntax_section = False
            in_try_it_section = False
            continue
        elif line_stripped.startswith('[') and line_stripped.endswith(']'):
            in_syntax_section = False
            in_examples_section = False
            in_try_it_section = False
            in_css_block = False
            continue
            
        # Handle CSS code blocks (marked with 'css' on its own line)
        if line_stripped == 'css':
            in_css_block = True
            continue
        elif in_css_block and (line_stripped.startswith('The ') or line_stripped.startswith('###') or (line_stripped and not line.startswith(' '))):
            in_css_block = False
            
        # Extract examples from relevant sections
        if current_property and (in_syntax_section or in_examples_section or in_try_it_section or in_css_block):
            # Look for CSS property declarations in Try it section
            if in_try_it_section and ':' in line and ';' in line:
                # Extract property: value; patterns
                prop_match = re.search(r'([a-zA-Z-]+)\\s*:\\s*([^;]+);', line)
                if prop_match:
                    prop_name = prop_match.group(1).strip().lower()
                    value = prop_match.group(2).strip()
                    if prop_name == current_property and value:
                        examples_buffer.append(f"{prop_name}: {value};")
            
            # Look for CSS property declarations in CSS blocks
            elif in_css_block and line.startswith('    ') and ':' in line:
                code_line = line[4:].strip()  # Remove indentation
                if current_property in code_line and ':' in code_line:
                    # Extract the value part
                    if not code_line.startswith('/*') and not code_line.startswith('<'):
                        examples_buffer.append(code_line)
    
    # Save the last property's examples
    if current_property and examples_buffer:
        property_examples[current_property] = '\n'.join(examples_buffer).strip()
    
    return property_examples

def update_property_data(file_path, property_formats, unity_examples=None, mozilla_examples=None):
    """Update the property_data.rs file with formats and examples"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Collect all properties that need updating
    all_properties = set()
    if property_formats:
        all_properties.update(property_formats.keys())
    if unity_examples:
        all_properties.update(unity_examples.keys())
    if mozilla_examples:
        all_properties.update(mozilla_examples.keys())
    
    updated_count = 0
    
    # For each property that needs updating
    for prop_name in all_properties:
        # Escape special regex characters in the property name
        escaped_prop_name = re.escape(prop_name)
        
        # Get the values for this property
        format_spec = property_formats.get(prop_name) if property_formats else None
        unity_example = unity_examples.get(prop_name) if unity_examples else None
        mozilla_example = mozilla_examples.get(prop_name) if mozilla_examples else None
        
        # Pattern to find the PropertyInfo struct for this property
        pattern = rf'(PropertyInfo \{{\s*name: "{escaped_prop_name}",\s*description: "[^"]*",\s*)(examples_unity: [^,]*,\s*)(examples_mozilla: [^,]*,\s*)(.*?\}},)'
        
        def build_replacement(match):
            result = match.group(1)  # name and description
            
            # Add examples_unity
            if unity_example:
                escaped_unity = unity_example.replace('\\', '\\\\\\\\')
                escaped_unity = escaped_unity.replace('"', '\\"')
                escaped_unity = escaped_unity.replace('\n', '\\n')
                result += f'examples_unity: Some("{escaped_unity}"),\n            '
            else:
                result += 'examples_unity: None,\n            '
            
            # Add examples_mozilla
            if mozilla_example:
                escaped_mozilla = mozilla_example.replace('\\', '\\\\\\\\')
                escaped_mozilla = escaped_mozilla.replace('"', '\\"')
                escaped_mozilla = escaped_mozilla.replace('\n', '\\n')
                result += f'examples_mozilla: Some("{escaped_mozilla}"),\n            '
            else:
                result += 'examples_mozilla: None,\n            '
            
            # Add the rest of the struct
            result += match.group(4)  # format and remaining fields
            
            return result
        
        # Apply the replacement
        new_content = re.sub(pattern, build_replacement, content, flags=re.MULTILINE | re.DOTALL)
        if new_content != content:
            content = new_content
            updated_count += 1
        else:
            print(f"Warning: Could not find PropertyInfo for property '{prop_name}'")
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"Updated {updated_count} properties with formats and/or examples")

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
    
    # Read the Unity USS documentation file
    unity_md_file_path = project_root / 'Assets' / 'data' / 'USS_property_format_6.0.md'
    with open(unity_md_file_path, 'r', encoding='utf-8') as f:
        unity_md_content = f.read()
    
    # Read the Mozilla CSS documentation file
    mozilla_md_file_path = project_root / 'Assets' / 'data' / 'Mozilla_CSS_properties_2025.md'
    with open(mozilla_md_file_path, 'r', encoding='utf-8') as f:
        mozilla_md_content = f.read()
    
    # Parse property formats from Unity documentation
    property_formats = parse_markdown_formats(unity_md_content)
    print(f"Found {len(property_formats)} property formats in Unity documentation")
    
    # Parse examples from Unity documentation
    unity_examples = parse_unity_examples(unity_md_content)
    print(f"Found {len(unity_examples)} property examples in Unity documentation")
    
    # Parse examples from Mozilla documentation
    mozilla_examples = parse_mozilla_examples(mozilla_md_content)
    print(f"Found {len(mozilla_examples)} property examples in Mozilla documentation")
    
    # Debug: Print some examples
    print("\nSample Unity examples:")
    for prop, example in list(unity_examples.items())[:3]:
        print(f"  {prop}: {example[:100]}..." if len(example) > 100 else f"  {prop}: {example}")
    
    print("\nSample Mozilla examples:")
    for prop, example in list(mozilla_examples.items())[:3]:
        print(f"  {prop}: {example[:100]}..." if len(example) > 100 else f"  {prop}: {example}")
    
    # Update the property data file
    property_data_path = project_root / 'src' / 'uss' / 'property_data.rs'
    update_property_data(str(property_data_path), property_formats, unity_examples, mozilla_examples)
    
    print("\nProperty data file updated successfully!")