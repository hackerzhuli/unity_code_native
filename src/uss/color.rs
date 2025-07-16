//! Color handling utilities for USS
//!
//! This module provides a centralized Color type that can parse various color formats
//! and convert between different representations.

use std::fmt;

/// Represents a color with RGBA components
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0.0-1.0)
    pub a: f32,
}

impl Color {
    /// Create a new color with RGB components and full opacity
    pub fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a new color with RGBA components
    pub fn new_rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a hex color string and return a Color
    /// Supports 3-digit (#rgb), 6-digit (#rrggbb), and 8-digit (#rrggbbaa) hex formats
    pub fn from_hex(hex_value: &str) -> Option<Self> {
        // Remove # if present
        let hex_part = if hex_value.starts_with('#') { &hex_value[1..] } else { hex_value };
        
        match hex_part.len() {
            3 => {
                // 3-digit hex: #rgb -> #rrggbb
                let r = u8::from_str_radix(&hex_part[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex_part[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex_part[2..3].repeat(2), 16).ok()?;
                Some(Self::new_rgb(r, g, b))
            }
            6 => {
                // 6-digit hex: #rrggbb
                let r = u8::from_str_radix(&hex_part[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex_part[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex_part[4..6], 16).ok()?;
                Some(Self::new_rgb(r, g, b))
            }
            8 => {
                // 8-digit hex: #rrggbbaa
                let r = u8::from_str_radix(&hex_part[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex_part[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex_part[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex_part[6..8], 16).ok()?;
                let alpha = a as f32 / 255.0;
                Some(Self::new_rgba(r, g, b, alpha))
            }
            _ => None,
        }
    }

    /// Get RGB components as a tuple
    pub fn rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Convert to hex string format (#rrggbb)
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Convert to hex string with alpha (#rrggbbaa)
    pub fn to_hex_with_alpha(&self) -> String {
        let alpha = (self.a * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, alpha)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 1.0 {
            write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new_rgb(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 1.0);

        let color_rgba = Color::new_rgba(255, 128, 64, 0.5);
        assert_eq!(color_rgba.a, 0.5);
    }

    #[test]
    fn test_hex_parsing() {
        // 6-digit hex
        let color = Color::from_hex("#ff8040").unwrap();
        assert_eq!(color.rgb(), (255, 128, 64));
        assert_eq!(color.a, 1.0);

        // 6-digit hex without #
        let color = Color::from_hex("ff8040").unwrap();
        assert_eq!(color.rgb(), (255, 128, 64));

        // 3-digit hex
        let color = Color::from_hex("#f84").unwrap();
        assert_eq!(color.rgb(), (255, 136, 68));

        // 8-digit hex with alpha
        let color = Color::from_hex("#ff804080").unwrap();
        assert_eq!(color.rgb(), (255, 128, 64));
        assert!((color.a - 0.5019608).abs() < 0.001); // 128/255 ≈ 0.5019608

        // 8-digit hex without #
        let color = Color::from_hex("ff8040ff").unwrap();
        assert_eq!(color.rgb(), (255, 128, 64));
        assert_eq!(color.a, 1.0);

        // Invalid hex
        assert!(Color::from_hex("#invalid").is_none());
        assert!(Color::from_hex("#ff").is_none());
        assert!(Color::from_hex("#ffff").is_none());
    }

    #[test]
    fn test_hex_conversion() {
        let color = Color::new_rgb(255, 128, 64);
        assert_eq!(color.to_hex(), "#ff8040");

        let color_with_alpha = Color::new_rgba(255, 128, 64, 0.5);
        assert_eq!(color_with_alpha.to_hex_with_alpha(), "#ff804080");
    }

    #[test]
    fn test_display() {
        let color = Color::new_rgb(255, 128, 64);
        assert_eq!(format!("{}", color), "rgb(255, 128, 64)");

        let color_with_alpha = Color::new_rgba(255, 128, 64, 0.5);
        assert_eq!(format!("{}", color_with_alpha), "rgba(255, 128, 64, 0.5)");
    }
}