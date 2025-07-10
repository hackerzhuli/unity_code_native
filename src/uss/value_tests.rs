use crate::uss::value::UssValue;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_value_and_unit() {
        // Test the helper function directly
        assert_eq!(UssValue::extract_value_and_unit("100"), ("100", None));
        assert_eq!(UssValue::extract_value_and_unit("100px"), ("100", Some("px".to_string())));
        assert_eq!(UssValue::extract_value_and_unit("10.5em"), ("10.5", Some("em".to_string())));
        assert_eq!(UssValue::extract_value_and_unit("-50%"), ("-50", Some("%".to_string())));
        assert_eq!(UssValue::extract_value_and_unit("0.75"), ("0.75", None));
        assert_eq!(UssValue::extract_value_and_unit("12.5vh"), ("12.5", Some("vh".to_string())));
        assert_eq!(UssValue::extract_value_and_unit("+3.14rad"), ("+3.14", Some("rad".to_string())));
    }

    #[test]
    fn test_extract_value_and_unit_edge_cases() {
        // Edge cases for value and unit extraction
        assert_eq!(UssValue::extract_value_and_unit(""), ("", None));
        assert_eq!(UssValue::extract_value_and_unit("px"), ("", Some("px".to_string())));
        assert_eq!(UssValue::extract_value_and_unit("123abc"), ("123", Some("abc".to_string())));
        assert_eq!(UssValue::extract_value_and_unit("0"), ("0", None));
        assert_eq!(UssValue::extract_value_and_unit("-0"), ("-0", None));
        assert_eq!(UssValue::extract_value_and_unit(".5"), (".5", None));
        assert_eq!(UssValue::extract_value_and_unit("-.5px"), ("-.5", Some("px".to_string())));
    }

    #[test]
    fn test_to_string_conversion() {
        // Test that values can be converted back to strings
        let numeric = UssValue::Numeric { value: 100.0, unit: Some("px".to_string()), has_fractional: false };
        assert_eq!(numeric.to_string(), "100px");
        
        let color = UssValue::Color("#ff0000".to_string());
        assert_eq!(color.to_string(), "#ff0000");
        
        let var_ref = UssValue::VariableReference("primary-color".to_string());
        assert_eq!(var_ref.to_string(), "var(--primary-color)");
        
        let identifier = UssValue::Identifier("flex".to_string());
        assert_eq!(identifier.to_string(), "flex");
        
        let asset = UssValue::Asset("url(\"image.png\")".to_string());
        assert_eq!(asset.to_string(), "url(\"image.png\")");
        
        let string_val = UssValue::String("\"Arial\"".to_string());
        assert_eq!(string_val.to_string(), "\"Arial\"");
    }
}