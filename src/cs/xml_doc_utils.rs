use regex::Regex;

/// Merge two XML documentation strings, handling inheritdoc tags
/// 
/// This function supports two cases:
/// 1. Top-level inheritdoc: `<inheritdoc cref="..."/>` - inherits everything but overrides locally defined content
/// 2. Nested inheritdoc: `<summary><inheritdoc cref="..."/></summary>` - replaces only the content of that specific tag
/// 
/// If multiple inheritdoc tags are present, returns the original XML unchanged.
pub fn merge_xml_docs(original_xml: &str, target_xml: &str) -> Option<String> {
    let original_trimmed = original_xml.trim();
    let target_trimmed = target_xml.trim();
    
    // Count inheritdoc tags
    let count = count_inheritdoc_tags(original_trimmed);
    
    // If no inheritdoc or multiple inheritdoc tags, return original unchanged
    if count == 0 || count > 1 {
        return Some(original_trimmed.to_string());
    }
    
    // Case 1: Single top-level inheritdoc tag
    if count == 1 && is_top_level_inheritdoc(original_trimmed) {
        // If original only contains inheritdoc, return target as-is
        let trimmed = original_trimmed.trim();
        if trimmed.starts_with("<inheritdoc") && trimmed.ends_with("/>") && !trimmed.contains('\n') {
            return Some(target_trimmed.to_string());
        }
        
        // Otherwise, merge: inherit everything from target but override with local content
        return Some(merge_top_level_inheritdoc(original_trimmed, target_trimmed));
    }
    
    // Case 2: Nested inheritdoc (inheritdoc inside specific tags)
    Some(merge_nested_inheritdoc(original_trimmed, target_trimmed))
}

/// Count the number of inheritdoc tags in the XML
fn count_inheritdoc_tags(xml: &str) -> usize {
    let re = Regex::new(r"<inheritdoc[^>]*/?>")
        .expect("Failed to compile inheritdoc regex");
    re.find_iter(xml).count()
}

/// Check if the XML contains a top-level inheritdoc tag
fn is_top_level_inheritdoc(xml: &str) -> bool {
    let trimmed = xml.trim();
    
    // Check if it starts with inheritdoc or has inheritdoc at the beginning of a line
    if trimmed.starts_with("<inheritdoc") {
        return true;
    }
    
    // Check if inheritdoc appears at the start of any line
    for line in trimmed.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with("<inheritdoc") {
            // Make sure it's not nested inside another tag on the same line
            let before_inheritdoc = &trimmed[..trimmed.find("<inheritdoc").unwrap()];
            let open_tags = before_inheritdoc.matches('<').count() - before_inheritdoc.matches("</").count();
            if open_tags == 0 {
                return true;
            }
        }
    }
    
    false
}

/// Merge XML docs when inheritdoc is at top level with local overrides
fn merge_top_level_inheritdoc(original_xml: &str, target_xml: &str) -> String {
    // Start with target XML and override with local content from original
    let mut result = target_xml.to_string();
    
    // Find all tags in original that are not inheritdoc
    let lines: Vec<&str> = original_xml.lines().collect();
    for line in lines {
        let trimmed_line = line.trim();
        if !trimmed_line.starts_with("<inheritdoc") && !trimmed_line.is_empty() {
            // Extract tag name from the line
            if let Some(tag_start) = trimmed_line.find('<') {
                if let Some(tag_end) = trimmed_line[tag_start + 1..].find('>') {
                    let tag_name = &trimmed_line[tag_start + 1..tag_start + 1 + tag_end];
                    let tag_name = tag_name.split_whitespace().next().unwrap_or("");
                    
                    // Find and replace existing tag in result
                     let tag_pattern = format!("<{}", tag_name);
                     let end_tag_pattern = format!("</{}>", tag_name);
                    
                    if let Some(existing_start) = result.find(&tag_pattern) {
                        // Find the end of this tag
                        let search_from = existing_start;
                        if let Some(relative_end) = result[search_from..].find(&end_tag_pattern) {
                            let full_end = search_from + relative_end + end_tag_pattern.len();
                            result.replace_range(existing_start..full_end, trimmed_line);
                        }
                    } else {
                        // Add the new tag
                        if !result.is_empty() {
                            result.push('\n');
                        }
                        result.push_str(trimmed_line);
                    }
                }
            }
        }
    }
    
    result
}

/// Merge XML docs when inheritdoc is nested inside specific tags
fn merge_nested_inheritdoc(original_xml: &str, target_xml: &str) -> String {
    let mut result = original_xml.to_string();
    
    // Find inheritdoc tags and their containing elements
    let inheritdoc_re = Regex::new(r"<inheritdoc[^>]*/?>")
        .expect("Failed to compile inheritdoc regex");
    
    // For each inheritdoc tag, find its parent tag and replace the content
    if let Some(inheritdoc_match) = inheritdoc_re.find(&result) {
        let inheritdoc_pos = inheritdoc_match.start();
        
        // Find the containing tag
        if let Some((tag_name, tag_start, tag_end)) = find_containing_tag(original_xml, inheritdoc_pos) {
            // Extract the corresponding content from target XML
            if let Some(target_content) = extract_tag_content(target_xml, &tag_name) {
                // Find the opening tag end position
                let opening_tag_pattern = format!("<{}[^>]*>", regex::escape(&tag_name));
                let opening_tag_re = Regex::new(&opening_tag_pattern)
                    .expect("Failed to compile opening tag regex");
                
                if let Some(opening_match) = opening_tag_re.find(&original_xml[tag_start..tag_end]) {
                    let content_start = tag_start + opening_match.end();
                    let closing_tag = format!("</{}>", tag_name);
                    let content_end = tag_end - closing_tag.len();
                    
                    // Replace the content between opening and closing tags
                    let before = &original_xml[..content_start];
                    let after = &original_xml[content_end..];
                    result = format!("{}{}{}", before, target_content, after);
                }
            }
        }
    }
    
    result
}

/// Find the containing tag for a given position in the XML
fn find_containing_tag(xml: &str, pos: usize) -> Option<(String, usize, usize)> {
    // Look backwards from the position to find the opening tag
    let before = &xml[..pos];
    
    // Find the last opening tag before this position
    let tag_re = Regex::new(r"<(\w+)[^>]*>")
        .expect("Failed to compile tag regex");
    let mut last_tag: Option<(String, usize)> = None;
    
    for cap in tag_re.captures_iter(before) {
        if let Some(tag_match) = cap.get(0) {
            if let Some(tag_name) = cap.get(1) {
                last_tag = Some((tag_name.as_str().to_string(), tag_match.start()));
            }
        }
    }
    
    if let Some((tag_name, tag_start)) = last_tag {
        // Find the corresponding closing tag
        let closing_tag = format!("</{}>", tag_name);
        if let Some(closing_pos) = xml[pos..].find(&closing_tag) {
            let tag_end = pos + closing_pos + closing_tag.len();
            return Some((tag_name, tag_start, tag_end));
        }
    }
    
    None
}

/// Extract content from a specific tag in the XML
fn extract_tag_content(xml: &str, tag_name: &str) -> Option<String> {
    let open_tag_re = Regex::new(&format!(r"<{}[^>]*>", regex::escape(tag_name)))
        .expect("Failed to compile tag content regex");
    let close_tag = format!("</{}>", tag_name);
    
    if let Some(open_match) = open_tag_re.find(xml) {
        let content_start = open_match.end();
        if let Some(close_pos) = xml[content_start..].find(&close_tag) {
            let content_end = content_start + close_pos;
            let content = xml[content_start..content_end].trim();
            return Some(content.to_string());
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_level_inheritdoc() {
        let original = "<inheritdoc cref=\"Add(int, int, int)\"/>";
        let target = "<summary>doc for add with 3 parameters</summary>\n<param name=\"a\"></param>\n<param name=\"b\"></param>\n<param name=\"c\"></param>\n<returns></returns>";
        
        let result = merge_xml_docs(original, target).unwrap();
        assert_eq!(result, target);
    }

    #[test]
    fn test_nested_inheritdoc_in_summary() {
        let original = "<summary><inheritdoc cref=\"Add5\"/></summary>\n<remarks>remarks from Add6</remarks>";
        let target = "<summary>doc for add 5</summary>\n<remarks>remarks from Add5</remarks>\n<returns>return from Add5</returns>";
        
        let result = merge_xml_docs(original, target).unwrap();
        let expected = "<summary>doc for add 5</summary>\n<remarks>remarks from Add6</remarks>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_inheritdoc_tags() {
        let original = "<summary><inheritdoc cref=\"Add5\"/></summary>\n<remarks><inheritdoc cref=\"Add5\"/></remarks>";
        let target = "<summary>doc for add 5</summary>\n<remarks>remarks from Add5</remarks>";
        
        let result = merge_xml_docs(original, target).unwrap();
        // Should return original unchanged due to multiple inheritdoc tags
        assert_eq!(result, original);
    }

    #[test]
    fn test_no_inheritdoc() {
        let original = "<summary>original summary</summary>\n<remarks>original remarks</remarks>";
        let target = "<summary>target summary</summary>\n<remarks>target remarks</remarks>";
        
        let result = merge_xml_docs(original, target).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_count_inheritdoc_tags() {
        assert_eq!(count_inheritdoc_tags("<inheritdoc cref=\"test\"/>"), 1);
        assert_eq!(count_inheritdoc_tags("<summary><inheritdoc cref=\"test\"/></summary>"), 1);
        assert_eq!(count_inheritdoc_tags("<summary><inheritdoc cref=\"test\"/></summary><remarks><inheritdoc cref=\"test2\"/></remarks>"), 2);
        assert_eq!(count_inheritdoc_tags("<summary>no inheritdoc here</summary>"), 0);
    }

    #[test]
    fn test_is_top_level_inheritdoc() {
        assert!(is_top_level_inheritdoc("<inheritdoc cref=\"test\"/>"));
        assert!(is_top_level_inheritdoc("  <inheritdoc cref=\"test\"/>  "));
        assert!(!is_top_level_inheritdoc("<summary><inheritdoc cref=\"test\"/></summary>"));
        assert!(!is_top_level_inheritdoc("<summary>test</summary><inheritdoc cref=\"test\"/>"));
    }

    #[test]
    fn test_inherit1_examples() {
        // Test case: Add6 - nested inheritdoc in summary, keeps own remarks
        let add6_original = "<summary><inheritdoc cref=\"Add5\"/></summary>\n<remarks>remarks from Add6</remarks>";
        let add5_target = "<summary>doc for add 5</summary>\n<remarks>remarks from Add5</remarks>\n<returns>return from Add5</returns>";
        
        let result = merge_xml_docs(add6_original, add5_target).unwrap();
        let expected = "<summary>doc for add 5</summary>\n<remarks>remarks from Add6</remarks>";
        assert_eq!(result, expected);
        
        // Test case: Add7 - top-level inheritdoc with override
        // According to user requirements: "inherit everything but override what is defined in place"
        let add7_original = "<inheritdoc cref=\"Add5\"/>\n<remarks>remarks from Add7</remarks>";
        let add7_result = merge_xml_docs(add7_original, add5_target).unwrap();
        // Should inherit everything from target but keep local overrides
        let expected_add7 = "<summary>doc for add 5</summary>\n<remarks>remarks from Add7</remarks>\n<returns>return from Add5</returns>";
        assert_eq!(add7_result, expected_add7);
        
        // Test case: Add4 - inheritdoc without parameters (parameter omission)
        let add4_original = "<inheritdoc cref=\"Add5\"/>";
        let add4_result = merge_xml_docs(add4_original, add5_target).unwrap();
        assert_eq!(add4_result, add5_target);
    }
}