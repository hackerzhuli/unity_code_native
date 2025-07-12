//! USS Keyword Data
//!
//! Contains documentation and information for USS keywords.
//! This module provides structured information about each keyword value
//! that can be used in USS properties.

use std::collections::HashMap;

/// Information about a USS keyword
#[derive(Debug, Clone)]
pub struct KeywordInfo {
    /// The keyword name
    pub name: &'static str,
    /// Markdown documentation for the keyword
    pub doc: &'static str,
}

impl KeywordInfo {
    /// Create a new KeywordInfo
    pub fn new(name: &'static str, doc: &'static str) -> Self {
        Self { name, doc }
    }
}

/// Create a map of all USS keywords with their documentation
pub fn create_keyword_info() -> HashMap<&'static str, KeywordInfo> {
    let mut keywords = HashMap::new();
    
    // Flexbox alignment keywords
    keywords.insert("flex-start", KeywordInfo::new("flex-start", "Items are packed toward the start of the flex direction."));
    keywords.insert("flex-end", KeywordInfo::new("flex-end", "Items are packed toward the end of the flex direction."));
    keywords.insert("center", KeywordInfo::new("center", "Items are centered along the line."));
    keywords.insert("space-between", KeywordInfo::new("space-between", "Items are evenly distributed with the first item at the start and the last item at the end."));
    keywords.insert("space-around", KeywordInfo::new("space-around", "Items are evenly distributed with equal space around them."));
    keywords.insert("stretch", KeywordInfo::new("stretch", "Items are stretched to fill the container."));
    keywords.insert("baseline", KeywordInfo::new("baseline", "Items are aligned such that their baselines align."));
    
    // Auto keyword
    keywords.insert("auto", KeywordInfo::new("auto", "The browser calculates the value automatically."));
    
    // Initial keyword
    keywords.insert("initial", KeywordInfo::new("initial", "Sets the property to its initial value."));
    
    // None keyword
    keywords.insert("none", KeywordInfo::new("none", "No value is applied."));
    
    // Background repeat keywords
    keywords.insert("repeat", KeywordInfo::new("repeat", "The background image is repeated both horizontally and vertically."));
    keywords.insert("repeat-x", KeywordInfo::new("repeat-x", "The background image is repeated only horizontally."));
    keywords.insert("repeat-y", KeywordInfo::new("repeat-y", "The background image is repeated only vertically."));
    keywords.insert("no-repeat", KeywordInfo::new("no-repeat", "The background image is not repeated."));
    keywords.insert("space", KeywordInfo::new("space", "The background image is repeated as much as possible without clipping."));
    keywords.insert("round", KeywordInfo::new("round", "The background image is repeated and rescaled to fit the container."));
    
    // Background size keywords
    keywords.insert("cover", KeywordInfo::new("cover", "Scale the image to cover the entire container, possibly cropping the image."));
    keywords.insert("contain", KeywordInfo::new("contain", "Scale the image to fit entirely within the container."));
    
    // Display keywords
    keywords.insert("flex", KeywordInfo::new("flex", "The element behaves like a block element and lays out its content according to the flexbox model."));
    
    // Flex direction keywords
    keywords.insert("row", KeywordInfo::new("row", "The flex container's main axis is horizontal (left to right)."));
    keywords.insert("row-reverse", KeywordInfo::new("row-reverse", "The flex container's main axis is horizontal (right to left)."));
    keywords.insert("column", KeywordInfo::new("column", "The flex container's main axis is vertical (top to bottom)."));
    keywords.insert("column-reverse", KeywordInfo::new("column-reverse", "The flex container's main axis is vertical (bottom to top)."));
    
    // Flex wrap keywords
    keywords.insert("nowrap", KeywordInfo::new("nowrap", "Flex items are laid out in a single line."));
    keywords.insert("wrap", KeywordInfo::new("wrap", "Flex items wrap onto multiple lines from top to bottom."));
    keywords.insert("wrap-reverse", KeywordInfo::new("wrap-reverse", "Flex items wrap onto multiple lines from bottom to top."));
    
    // Overflow keywords
    keywords.insert("visible", KeywordInfo::new("visible", "Content is not clipped and may be rendered outside the element's box."));
    keywords.insert("hidden", KeywordInfo::new("hidden", "Content is clipped and no scrollbars are provided."));
    keywords.insert("scroll", KeywordInfo::new("scroll", "Content is clipped and scrollbars are provided."));
    
    // Position keywords
    keywords.insert("relative", KeywordInfo::new("relative", "The element is positioned relative to its normal position."));
    keywords.insert("absolute", KeywordInfo::new("absolute", "The element is positioned relative to its nearest positioned ancestor."));
    
    // Text overflow keywords
    keywords.insert("clip", KeywordInfo::new("clip", "Text is clipped at the overflow point."));
    keywords.insert("ellipsis", KeywordInfo::new("ellipsis", "Text is clipped and an ellipsis (...) is displayed."));
    
    // Transition property keywords
    keywords.insert("all", KeywordInfo::new("all", "All properties that can transition will transition."));
    
    // Transition timing function keywords
    keywords.insert("ease", KeywordInfo::new("ease", "Slow start, fast middle, slow end (cubic-bezier(0.25, 0.1, 0.25, 1))."));
    keywords.insert("ease-in", KeywordInfo::new("ease-in", "Slow start (cubic-bezier(0.42, 0, 1, 1))."));
    keywords.insert("ease-out", KeywordInfo::new("ease-out", "Slow end (cubic-bezier(0, 0, 0.58, 1))."));
    keywords.insert("ease-in-out", KeywordInfo::new("ease-in-out", "Slow start and end (cubic-bezier(0.42, 0, 0.58, 1))."));
    keywords.insert("linear", KeywordInfo::new("linear", "Constant speed (cubic-bezier(0, 0, 1, 1))."));
    
    // White space keywords
    keywords.insert("normal", KeywordInfo::new("normal", "Sequences of whitespace are collapsed. Newlines are treated as whitespace."));
    
    // Cursor keywords
    keywords.insert("arrow", KeywordInfo::new("arrow", "Default arrow cursor."));
    keywords.insert("text", KeywordInfo::new("text", "Text selection cursor (I-beam)."));
    keywords.insert("resize-vertical", KeywordInfo::new("resize-vertical", "Vertical resize cursor."));
    keywords.insert("resize-horizontal", KeywordInfo::new("resize-horizontal", "Horizontal resize cursor."));
    keywords.insert("link", KeywordInfo::new("link", "Link cursor (pointing hand)."));
    keywords.insert("slide-arrow", KeywordInfo::new("slide-arrow", "Slide arrow cursor."));
    keywords.insert("resize-up-right", KeywordInfo::new("resize-up-right", "Diagonal resize cursor (up-right)."));
    keywords.insert("resize-up-left", KeywordInfo::new("resize-up-left", "Diagonal resize cursor (up-left)."));
    keywords.insert("move-arrow", KeywordInfo::new("move-arrow", "Move cursor (four-way arrow)."));
    keywords.insert("rotate-arrow", KeywordInfo::new("rotate-arrow", "Rotate cursor."));
    keywords.insert("scale-arrow", KeywordInfo::new("scale-arrow", "Scale cursor."));
    keywords.insert("arrow-plus", KeywordInfo::new("arrow-plus", "Arrow with plus sign cursor."));
    keywords.insert("arrow-minus", KeywordInfo::new("arrow-minus", "Arrow with minus sign cursor."));
    keywords.insert("pan", KeywordInfo::new("pan", "Pan cursor (hand)."));
    keywords.insert("orbit", KeywordInfo::new("orbit", "Orbit cursor."));
    keywords.insert("zoom", KeywordInfo::new("zoom", "Zoom cursor."));
    keywords.insert("fps", KeywordInfo::new("fps", "First-person shooter cursor."));
    keywords.insert("split-resize-up-down", KeywordInfo::new("split-resize-up-down", "Split resize cursor (up-down)."));
    keywords.insert("split-resize-left-right", KeywordInfo::new("split-resize-left-right", "Split resize cursor (left-right)."));
    
    // Unity-specific background scale mode keywords
    keywords.insert("stretch-to-fill", KeywordInfo::new("stretch-to-fill", "Stretch the image to fill the entire element, ignoring aspect ratio."));
    keywords.insert("scale-and-crop", KeywordInfo::new("scale-and-crop", "Scale the image to fill the element while maintaining aspect ratio, cropping if necessary."));
    keywords.insert("scale-to-fit", KeywordInfo::new("scale-to-fit", "Scale the image to fit within the element while maintaining aspect ratio."));
    
    // Unity text rendering mode keywords
    keywords.insert("legacy", KeywordInfo::new("legacy", "Use legacy text rendering."));
    keywords.insert("distance-field", KeywordInfo::new("distance-field", "Use distance field text rendering for better quality at various sizes."));
    
    // Unity font style keywords
    keywords.insert("bold", KeywordInfo::new("bold", "Bold font weight."));
    keywords.insert("italic", KeywordInfo::new("italic", "Italic font style."));
    keywords.insert("bold-and-italic", KeywordInfo::new("bold-and-italic", "Both bold weight and italic style."));
    
    // Unity overflow clip box keywords
    keywords.insert("padding-box", KeywordInfo::new("padding-box", "Clip to the padding box."));
    keywords.insert("content-box", KeywordInfo::new("content-box", "Clip to the content box."));
    
    // Unity slice type keywords
    keywords.insert("tile", KeywordInfo::new("tile", "Tile the slice edges."));
    keywords.insert("mirror", KeywordInfo::new("mirror", "Mirror the slice edges."));
    
    // Unity text alignment keywords
    keywords.insert("upper-left", KeywordInfo::new("upper-left", "Align text to the upper-left corner."));
    keywords.insert("middle-left", KeywordInfo::new("middle-left", "Align text to the middle-left."));
    keywords.insert("lower-left", KeywordInfo::new("lower-left", "Align text to the lower-left corner."));
    keywords.insert("upper-center", KeywordInfo::new("upper-center", "Align text to the upper-center."));
    keywords.insert("middle-center", KeywordInfo::new("middle-center", "Align text to the middle-center."));
    keywords.insert("lower-center", KeywordInfo::new("lower-center", "Align text to the lower-center."));
    keywords.insert("upper-right", KeywordInfo::new("upper-right", "Align text to the upper-right corner."));
    keywords.insert("middle-right", KeywordInfo::new("middle-right", "Align text to the middle-right."));
    keywords.insert("lower-right", KeywordInfo::new("lower-right", "Align text to the lower-right corner."));
    
    // Unity text generator keywords
    keywords.insert("standard", KeywordInfo::new("standard", "Use standard text generator."));
    keywords.insert("advanced", KeywordInfo::new("advanced", "Use advanced text generator with more features."));
    
    // Unity text overflow position keywords
    keywords.insert("start", KeywordInfo::new("start", "Text overflow occurs at the start."));
    keywords.insert("middle", KeywordInfo::new("middle", "Text overflow occurs in the middle."));
    keywords.insert("end", KeywordInfo::new("end", "Text overflow occurs at the end."));
    
    keywords
}
