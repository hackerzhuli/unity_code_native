//! USS Utilities
//!
//! Utility functions for USS parsing and value conversion.

/// Error type for USS string parsing
#[derive(Debug, Clone, PartialEq)]
pub struct UssStringError {
    /// Descriptive error message
    pub message: String,
    /// Position in the string where the error occurred
    pub position: usize,
}

impl UssStringError {
    fn new(message: String, position: usize) -> Self {
        Self { message, position }
    }
}

/// Convert a USS string literal (including quotes) to its actual string value.
/// 
/// This function handles:
/// - Removing surrounding quotes (single or double)
/// - Processing escape sequences according to CSS/USS specification:
///   - Backslash followed by newline is ignored
///   - Backslash cancels special meaning of characters
///   - Hexadecimal escapes (\XXXXXX or \XX with optional space)
///   - Unicode replacement for invalid codepoints
/// 
/// # Arguments
/// * `input` - The raw string from tree-sitter parser (includes quotes)
/// 
/// # Returns
/// * `Ok(String)` - The actual string value
/// * `Err(UssStringError)` - If the string is malformed
/// 
/// # Examples
/// ```
/// use unity_code_native::uss::uss_utils::convert_uss_string;
/// 
/// assert_eq!(convert_uss_string(r#""hello""#).unwrap(), "hello");
/// assert_eq!(convert_uss_string(r#"'world'"#).unwrap(), "world");
/// assert_eq!(convert_uss_string(r#""test\"quote""#).unwrap(), "test\"quote");
/// assert_eq!(convert_uss_string(r#""\26 B""#).unwrap(), "&B");
/// ```
pub fn convert_uss_string(input: &str) -> Result<String, UssStringError> {
    if input.len() < 2 {
        return Err(UssStringError::new(
            "String must be at least 2 characters (quotes)".to_string(),
            0,
        ));
    }

    let chars: Vec<char> = input.chars().collect();
    let quote_char = chars[0];
    
    // Check if string starts and ends with matching quotes
    if !matches!(quote_char, '"' | '\'') {
        return Err(UssStringError::new(
            "String must start with quote character".to_string(),
            0,
        ));
    }
    
    if chars[chars.len() - 1] != quote_char {
        return Err(UssStringError::new(
            "String must end with matching quote character".to_string(),
            chars.len() - 1,
        ));
    }

    // Extract content between quotes
    let content = &chars[1..chars.len() - 1];
    let mut result = String::new();
    let mut i = 0;

    while i < content.len() {
        let ch = content[i];
        
        if ch == '\\' {
            if i + 1 >= content.len() {
                // Backslash at end of string stands for itself
                result.push(ch);
                break;
            }
            
            let next_ch = content[i + 1];
            
            // Handle backslash followed by newline (ignored)
            if next_ch == '\n' || next_ch == '\r' {
                i += 2;
                // Handle CRLF as single whitespace
                if next_ch == '\r' && i < content.len() && content[i] == '\n' {
                    i += 1;
                }
                continue;
            }
            
            // Handle hexadecimal escapes
            if next_ch.is_ascii_hexdigit() {
                let hex_start = i + 1;
                let mut hex_end = hex_start;
                
                // Collect up to 6 hex digits
                while hex_end < content.len() && 
                      hex_end - hex_start < 6 && 
                      content[hex_end].is_ascii_hexdigit() {
                    hex_end += 1;
                }
                
                let hex_str: String = content[hex_start..hex_end].iter().collect();
                
                if let Ok(codepoint) = u32::from_str_radix(&hex_str, 16) {
                    // Check if valid Unicode codepoint
                    if codepoint == 0 {
                        return Err(UssStringError::new(
                            "Unicode codepoint zero is not allowed".to_string(),
                            i,
                        ));
                    }
                    
                    if codepoint > 0x10FFFF {
                        // Replace with Unicode replacement character
                        result.push('\u{FFFD}');
                    } else if let Some(unicode_char) = char::from_u32(codepoint) {
                        result.push(unicode_char);
                    } else {
                        // Invalid codepoint, use replacement character
                        result.push('\u{FFFD}');
                    }
                } else {
                    return Err(UssStringError::new(
                        format!("Invalid hexadecimal escape: {}", hex_str),
                        i,
                    ));
                }
                
                i = hex_end;
                
                // Per CSS spec: "Only one white space character is ignored after a hexadecimal escape"
                // This means we consume exactly one whitespace if present, regardless of what follows
                // Examples from spec: "\26 B" -> "&B", "\000026B" -> "&B"
                if i < content.len() && content[i].is_whitespace() {
                    // Handle CRLF as single whitespace
                    if content[i] == '\r' && i + 1 < content.len() && content[i + 1] == '\n' {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                continue;
            }
            
            // Handle other escaped characters (cancel special meaning)
            // Any character except hex digit, LF, CR, FF can be escaped
            if !matches!(next_ch, '\n' | '\r' | '\x0C') {
                result.push(next_ch);
                i += 2;
                continue;
            }
            
            // Backslash followed by LF, CR, or FF - backslash stands for itself
            result.push(ch);
            i += 1;
        } else {
            result.push(ch);
            i += 1;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_strings() {
        assert_eq!(convert_uss_string(r#""hello""#).unwrap(), "hello");
        assert_eq!(convert_uss_string("'world'").unwrap(), "world");
        assert_eq!(convert_uss_string(r#""""#).unwrap(), "");
        assert_eq!(convert_uss_string("''").unwrap(), "");
    }

    #[test]
    fn test_escaped_quotes() {
        assert_eq!(convert_uss_string(r#""test\"quote""#).unwrap(), "test\"quote");
        assert_eq!(convert_uss_string("'test\\'quote'").unwrap(), "test'quote");
    }

    #[test]
    fn test_escaped_backslash() {
        assert_eq!(convert_uss_string(r#""test\\backslash""#).unwrap(), "test\\backslash");
    }

    #[test]
    fn test_hex_escapes() {
        // \26 = & (ampersand) - space should be consumed
        assert_eq!(convert_uss_string(r#""\26 B""#).unwrap(), "&B");
        assert_eq!(convert_uss_string(r#""\000026B""#).unwrap(), "&B");
        
        // \7B = { (left brace) - space should be consumed per CSS spec
        assert_eq!(convert_uss_string(r#""\7B test""#).unwrap(), "{test");
        
        // \32 = 2 (digit 2) - space should be consumed per CSS spec
        assert_eq!(convert_uss_string(r#""\32 test""#).unwrap(), "2test");
    }

    #[test]
    fn test_newline_escapes() {
        // Backslash followed by newline should be ignored
        assert_eq!(convert_uss_string("\"test\\
more\"").unwrap(), "testmore");
        assert_eq!(convert_uss_string("\"test\\
more\"").unwrap(), "testmore");
    }
    
    #[test]
    fn test_unicode_replacement() {
        // Codepoint above Unicode maximum should be replaced
        // \110000 is above Unicode max (10FFFF), gets replaced with U+FFFD
        // The space after the hex escape is consumed per CSS spec
        assert_eq!(convert_uss_string(r#""\110000 ""#).unwrap(), "\u{FFFD}");
    }

    #[test]
    fn test_whitespace_after_hex() {
        // Only one whitespace should be consumed after hex escape
        assert_eq!(convert_uss_string(r#""\26  B""#).unwrap(), "& B");
        assert_eq!(convert_uss_string("\"\\26\r\nB\"").unwrap(), "&B");
    }

    #[test]
    fn test_identifier_equivalence() {
        // CSS spec: "The identifier 'te\st' is exactly the same identifier as 'test'"
        // \73 is 's' in hex, space should be consumed per CSS spec
        assert_eq!(convert_uss_string(r#""te\73 t""#).unwrap(), "test");
    }

    #[test]
    fn test_error_cases() {
        // Missing quotes
        assert!(convert_uss_string("hello").is_err());
        
        // Mismatched quotes
        assert!(convert_uss_string("'hello\"").is_err());
        
        // Too short
        assert!(convert_uss_string("'").is_err());
        
        // Zero codepoint
        assert!(convert_uss_string(r#""\0 ""#).is_err());
    }

    #[test]
    fn test_backslash_at_end() {
        // Backslash at end should stand for itself
        assert_eq!(convert_uss_string(r#""test\""#).unwrap(), "test\\");
    }

    #[test]
    fn test_special_characters() {
        // Test various special characters that can be escaped
        assert_eq!(convert_uss_string(r#""\{\}\(\)""#).unwrap(), "{}()");
        assert_eq!(convert_uss_string(r#""\;\:\,""#).unwrap(), ";:,");
    }
}