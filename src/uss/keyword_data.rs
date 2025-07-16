//! USS Keyword Data
//!
//! Contains documentation and information for USS keywords.
//! This module provides structured information about each keyword value
//! that can be used in USS properties.

use std::collections::{HashMap, HashSet};
use crate::uss::definitions::{KeywordInfo, PropertyInfo};
use crate::uss::value_spec::ValueType;

/// Helper function to create a KeywordInfo with specified properties
fn create(name: &'static str, doc: &'static str, properties: &[&'static str]) -> KeywordInfo {
    create_with_property_docs(name, doc, properties, &[])
}

/// Helper function to create a KeywordInfo with property-specific documentation
fn create_with_property_docs(name: &'static str, doc: &'static str, properties: &[&'static str], property_docs: &[(&'static str, &'static str)]) -> KeywordInfo {
    let mut used_by_properties = HashSet::new();
    for prop in properties {
        used_by_properties.insert(*prop);
    }
    let mut docs_for_property = HashMap::new();
    for (prop, prop_doc) in property_docs {
        docs_for_property.insert(*prop, *prop_doc);
    }
    KeywordInfo::new_with_property_docs(name, doc, used_by_properties, docs_for_property)
}

/// Create a map of all USS keywords with their documentation
pub fn create_keyword_info() -> HashMap<&'static str, KeywordInfo> {
    let mut keywords = HashMap::new();
    
    // Flexbox alignment keywords
    keywords.insert("flex-start", create("flex-start", "Place items at the start of the direction.", &["justify-content", "align-items", "align-content", "align-self"]));
    keywords.insert("flex-end", create("flex-end", "Place items at the end of the direction.", &["justify-content", "align-items", "align-content", "align-self"]));
    keywords.insert("center", create_with_property_docs("center", "Place items or content in the center.", &["justify-content", "align-items", "align-content", "align-self", "background-position", "background-position-x", "background-position-y", "transform-origin"], &[
        ("justify-content", "Centers flex items along the main axis."),
        ("align-items", "Centers flex items along the cross axis."),
        ("align-content", "Centers flex lines when there's extra space in the cross axis."),
        ("align-self", "Centers this flex item along the cross axis."),
        ("background-position", "Centers the background image both horizontally and vertically."),
        ("background-position-x", "Centers the background image horizontally."),
        ("background-position-y", "Centers the background image vertically."),
        ("transform-origin", "Sets the transform origin to the center of the element.")
    ]));
    keywords.insert("space-between", create("space-between", "Items are evenly distributed with the first item at the start and the last item at the end of the direction.", &["justify-content"]));
    keywords.insert("space-around", create("space-around", "Items are evenly distributed with equal space around them.", &["justify-content"]));
    keywords.insert("stretch", create("stretch", "Items are stretched to fill the container.", &["align-items", "align-content", "align-self"]));
    
    // Auto keyword
    keywords.insert("auto", create_with_property_docs("auto", "Unity Engine calculates the value automatically.", &["width", "height", "min-width", "min-height", "flex-basis", "margin", "margin-top", "margin-right", "margin-bottom", "margin-left", "top", "right", "bottom", "left", "align-self", "align-items", "background-size", "flex"], &[
        ("width", "Automatically calculates the width based on content and constraints."),
        ("height", "Automatically calculates the height based on content and constraints."),
        ("min-width", "No minimum width constraint."),
        ("min-height", "No minimum height constraint."),
        ("flex-basis", "Uses the element's main size property (width or height) as the flex basis."),
        ("margin", "Automatically distributes available space as margin."),
        ("margin-top", "Automatically calculates top margin."),
        ("margin-right", "Automatically calculates right margin."),
        ("margin-bottom", "Automatically calculates bottom margin."),
        ("margin-left", "Automatically calculates left margin."),
        ("top", "Automatically positions the element from the top."),
        ("right", "Automatically positions the element from the right."),
        ("bottom", "Automatically positions the element from the bottom."),
        ("left", "Automatically positions the element from the left."),
        ("align-self", "Uses the parent's align-items value."),
        ("align-items", "Uses the default alignment for the flex container."),
        ("background-size", "Uses the intrinsic size of the background image."),
        ("flex", "Equivalent to flex: 1 1 auto, making the item flexible.")
    ]));
    
    // Initial keyword
    keywords.insert("initial", create("initial", "Sets the property to its initial value.", &[
        "align-content", "align-items", "align-self", "all", "background-color", "background-image", "background-position", "background-position-x", "background-position-y", "background-repeat", "background-size", "border-bottom-color", "border-bottom-left-radius", "border-bottom-right-radius", "border-bottom-width", "border-color", "border-left-color", "border-left-width", "border-radius", "border-right-color", "border-top-left-radius", "border-top-color", "border-right-width", "border-top-right-radius", "border-top-width", "border-width", "bottom", "color", "cursor", "display", "flex", "flex-basis", "flex-direction", "flex-grow", "flex-shrink", "flex-wrap", "font-size", "height", "justify-content", "left", "letter-spacing", "margin", "margin-bottom", "margin-left", "margin-right", "margin-top", "max-height", "max-width", "min-height", "min-width", "opacity", "overflow", "padding", "padding-bottom", "padding-left", "padding-right", "padding-top", "position", "right", "rotate", "scale", "text-overflow", "text-shadow", "top", "transform-origin", "transition", "transition-delay", "transition-duration", "transition-property", "transition-timing-function", "translate", "-unity-background-image-tint-color", "-unity-background-scale-mode", "-unity-editor-text-rendering-mode", "-unity-font", "-unity-font-definition", "-unity-font-style", "-unity-overflow-clip-box", "-unity-paragraph-spacing", "-unity-slice-bottom", "-unity-slice-left", "-unity-slice-right", "-unity-slice-scale", "-unity-slice-top", "-unity-slice-type", "-unity-text-align", "-unity-text-generator", "-unity-text-outline", "-unity-text-outline-color", "-unity-text-outline-width", "-unity-text-overflow-position", "visibility", "white-space", "width", "word-spacing"
    ]));
    
    // None keyword
    keywords.insert("none", create_with_property_docs("none", "No value is applied.", &["background-image", "transition-property", "scale", "rotate", "max-height", "display", "flex", "max-width", "translate"], &[
        ("background-image", "No background image is displayed."),
        ("transition-property", "No properties will be transitioned."),
        ("scale", "No scaling transformation is applied."),
        ("rotate", "No rotation transformation is applied."),
        ("max-height", "No maximum height constraint."),
        ("display", "The element is not displayed (hidden)."),
        ("flex", "Equivalent to flex: 0 0 auto, making the item inflexible."),
        ("max-width", "No maximum width constraint."),
        ("translate", "No translation transformation is applied.")
    ]));
    
    // Background repeat keywords
    keywords.insert("repeat", create("repeat", "The background image is repeated both horizontally and vertically.", &["background-repeat"]));
    keywords.insert("repeat-x", create("repeat-x", "The background image is repeated only horizontally.", &["background-repeat"]));
    keywords.insert("repeat-y", create("repeat-y", "The background image is repeated only vertically.", &["background-repeat"]));
    keywords.insert("no-repeat", create("no-repeat", "The background image is not repeated.", &["background-repeat"]));
    keywords.insert("space", create("space", "The background image is repeated as much as possible without clipping.", &["background-repeat"]));
    keywords.insert("round", create("round", "The background image is repeated and rescaled to fit the container.", &["background-repeat"]));
    
    // Background size keywords
    keywords.insert("cover", create("cover", "Scale the image to cover the entire container, possibly cropping the image.", &["background-size"]));
    keywords.insert("contain", create("contain", "Scale the image to fit entirely within the container.", &["background-size"]));
    
    // Display keywords
    keywords.insert("flex", create("flex", "Layout the element with flexbox model.", &["display"]));
    
    // Flex direction keywords
    keywords.insert("row", create("row", "The flex container's main axis is horizontal (left to right).", &["flex-direction"]));
    keywords.insert("row-reverse", create("row-reverse", "The flex container's main axis is horizontal (right to left).", &["flex-direction"]));
    keywords.insert("column", create("column", "The flex container's main axis is vertical (top to bottom).", &["flex-direction"]));
    keywords.insert("column-reverse", create("column-reverse", "The flex container's main axis is vertical (bottom to top).", &["flex-direction"]));
    
    // Flex wrap keywords
    keywords.insert("wrap", create("wrap", "Flex items wrap onto multiple lines from top to bottom.", &["flex-wrap"]));
    keywords.insert("wrap-reverse", create("wrap-reverse", "Flex items wrap onto multiple lines from bottom to top.", &["flex-wrap"]));
    
    // Overflow keywords
    keywords.insert("visible", create_with_property_docs("visible", "Content is not clipped and may be rendered outside the element's box.", &["overflow", "visibility"], &[
        ("overflow", "Content is not clipped and may overflow the element's bounds."),
        ("visibility", "The element is visible and takes up space in the layout.")
    ]));
    keywords.insert("hidden", create_with_property_docs("hidden", "Content is clipped and no scrollbars are provided.", &["overflow", "visibility"], &[
        ("overflow", "Content that overflows is clipped and hidden."),
        ("visibility", "The element is invisible but still takes up space in the layout.")
    ]));
    
    // Position keywords
    keywords.insert("relative", create("relative", "The element is positioned relative to its normal position.", &["position"]));
    keywords.insert("absolute", create("absolute", "The element is positioned relative to its parent.", &["position"]));
    
    // Text overflow keywords
    keywords.insert("clip", create("clip", "Text is clipped at the overflow point.", &["text-overflow"]));
    keywords.insert("ellipsis", create("ellipsis", "Text is clipped and an ellipsis (...) is displayed.", &["text-overflow"]));
    
    // All keyword with different meanings
    keywords.insert("all", create("all", "All properties that can transition will transition.", &["transition-property"]));
    
    // Transition timing function keywords
    keywords.insert("ease", create("ease", "Slow start, fast middle, slow end (cubic-bezier(0.25, 0.1, 0.25, 1)).", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in", create("ease-in", "Slow start (cubic-bezier(0.42, 0, 1, 1)).", &["transition-timing-function", "transition"]));
    keywords.insert("ease-out", create("ease-out", "Slow end (cubic-bezier(0, 0, 0.58, 1)).", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-out", create("ease-in-out", "Slow start and end (cubic-bezier(0.42, 0, 0.58, 1)).", &["transition-timing-function", "transition"]));
    keywords.insert("linear", create("linear", "Constant speed (cubic-bezier(0, 0, 1, 1)).", &["transition-timing-function", "transition"]));
    
    // White space keywords
    keywords.insert("normal", create_with_property_docs("normal", "Sequences of whitespace are collapsed. Newlines are treated as whitespace.", &["white-space", "-unity-font-style"], &[
        ("white-space", "Sequences of whitespace are collapsed and newlines are treated as whitespace."),
        ("-unity-font-style", "Normal font weight and style (not bold or italic).")
    ]));
    keywords.insert("nowrap", create_with_property_docs("nowrap", "Flex items are laid out in a single line.", &["flex-wrap", "white-space"], &[
        ("flex-wrap", "Flex items are laid out in a single line and do not wrap."),
        ("white-space", "Text does not wrap to new lines.")
    ]));
    
    // Cursor keywords
    keywords.insert("arrow", create("arrow", "Default arrow cursor.", &["cursor"]));
    keywords.insert("text", create("text", "Text selection cursor (I-beam).", &["cursor"]));
    keywords.insert("resize-vertical", create("resize-vertical", "Vertical resize cursor.", &["cursor"]));
    keywords.insert("resize-horizontal", create("resize-horizontal", "Horizontal resize cursor.", &["cursor"]));
    keywords.insert("link", create("link", "Link cursor (pointing hand).", &["cursor"]));
    keywords.insert("slide-arrow", create("slide-arrow", "Slide arrow cursor.", &["cursor"]));
    keywords.insert("resize-up-right", create("resize-up-right", "Diagonal resize cursor (up-right).", &["cursor"]));
    keywords.insert("resize-up-left", create("resize-up-left", "Diagonal resize cursor (up-left).", &["cursor"]));
    keywords.insert("move-arrow", create("move-arrow", "Move cursor (four-way arrow).", &["cursor"]));
    keywords.insert("rotate-arrow", create("rotate-arrow", "Rotate cursor.", &["cursor"]));
    keywords.insert("scale-arrow", create("scale-arrow", "Scale cursor.", &["cursor"]));
    keywords.insert("arrow-plus", create("arrow-plus", "Arrow with plus sign cursor.", &["cursor"]));
    keywords.insert("arrow-minus", create("arrow-minus", "Arrow with minus sign cursor.", &["cursor"]));
    keywords.insert("pan", create("pan", "Pan cursor (hand).", &["cursor"]));
    keywords.insert("orbit", create("orbit", "Orbit cursor.", &["cursor"]));
    keywords.insert("zoom", create("zoom", "Zoom cursor.", &["cursor"]));
    keywords.insert("fps", create("fps", "First-person shooter cursor.", &["cursor"]));
    keywords.insert("split-resize-up-down", create("split-resize-up-down", "Split resize cursor (up-down).", &["cursor"]));
    keywords.insert("split-resize-left-right", create("split-resize-left-right", "Split resize cursor (left-right).", &["cursor"]));
    
    // Unity-specific background scale mode keywords
    keywords.insert("stretch-to-fill", create("stretch-to-fill", "Stretches the image to fill the entire area of the visual element.", &["-unity-background-scale-mode"]));
    keywords.insert("scale-and-crop", create("scale-and-crop", "Scales the image to fit the visual element. If the image is larger than the visual element, the image is cropped.", &["-unity-background-scale-mode"]));
    keywords.insert("scale-to-fit", create("scale-to-fit", "Scales the image to fit the visual element. It's similar to the stretch-to-fill mode, but the aspect ratio of the image is preserved.", &["-unity-background-scale-mode"]));
    
    // Unity text rendering mode keywords
    keywords.insert("legacy", create("legacy", "Use legacy text rendering.", &["-unity-editor-text-rendering-mode"]));
    keywords.insert("distance-field", create("distance-field", "Use distance field text rendering for better quality at various sizes.", &["-unity-editor-text-rendering-mode"]));
    
    // Unity font style keywords
    keywords.insert("bold", create("bold", "Bold font weight.", &["-unity-font-style"]));
    keywords.insert("italic", create("italic", "Italic font style.", &["-unity-font-style"]));
    keywords.insert("bold-and-italic", create("bold-and-italic", "Both bold weight and italic style.", &["-unity-font-style"]));
    
    // Unity overflow clip box keywords
    keywords.insert("padding-box", create("padding-box", "Clip to the padding box.", &["-unity-overflow-clip-box"]));
    keywords.insert("content-box", create("content-box", "Clip to the content box.", &["-unity-overflow-clip-box"]));
    
    // Unity slice type keywords
    
    // Unity text alignment keywords
    keywords.insert("upper-left", create("upper-left", "Align text to the upper-left corner.", &["-unity-text-align"]));
    keywords.insert("middle-left", create("middle-left", "Align text to the middle-left.", &["-unity-text-align"]));
    keywords.insert("lower-left", create("lower-left", "Align text to the lower-left corner.", &["-unity-text-align"]));
    keywords.insert("upper-center", create("upper-center", "Align text to the upper-center.", &["-unity-text-align"]));
    keywords.insert("middle-center", create("middle-center", "Align text to the middle-center.", &["-unity-text-align"]));
    keywords.insert("lower-center", create("lower-center", "Align text to the lower-center.", &["-unity-text-align"]));
    keywords.insert("upper-right", create("upper-right", "Align text to the upper-right corner.", &["-unity-text-align"]));
    keywords.insert("middle-right", create("middle-right", "Align text to the middle-right.", &["-unity-text-align"]));
    keywords.insert("lower-right", create("lower-right", "Align text to the lower-right corner.", &["-unity-text-align"]));
    
    // Unity text generator keywords
    keywords.insert("standard", create("standard", "Use standard text generator.", &["-unity-text-generator"]));
    keywords.insert("advanced", create("advanced", "Advanced Text Generator is a text rendering module that employs Harfbuzz, ICU, and FreeType to deliver comprehensive Unicode support and text shaping capabilities. With Advanced Text Generator, you can use a wide array of languages and scripts, such as right-to-left (RTL) languages like Arabic and Hebrew.", &["-unity-text-generator"]));
    
    // Unity text overflow position keywords
    keywords.insert("start", create("start", "Text overflow occurs at the start.", &["-unity-text-overflow-position"]));
    keywords.insert("middle", create_with_property_docs("middle", "Text overflow occurs in the middle.", &["-unity-text-overflow-position"], &[
        ("-unity-text-overflow-position", "Text overflow occurs in the middle of the text.")
    ]));
    keywords.insert("end", create_with_property_docs("end", "Text overflow occurs at the end.", &["-unity-text-overflow-position"], &[
        ("-unity-text-overflow-position", "Text overflow occurs at the end of the text.")
    ]));
    
    // Directional keywords
    keywords.insert("top", create_with_property_docs("top", "Top position or alignment.", &["background-position", "background-position-y", "transform-origin"], &[
        ("background-position", "Positions the background image at the top."),
        ("background-position-y", "Positions the background image at the top vertically."),
        ("transform-origin", "Sets the transform origin to the top edge of the element.")
    ]));
    keywords.insert("bottom", create_with_property_docs("bottom", "Bottom position or alignment.", &["background-position", "background-position-y", "transform-origin"], &[
        ("background-position", "Positions the background image at the bottom."),
        ("background-position-y", "Positions the background image at the bottom vertically."),
        ("transform-origin", "Sets the transform origin to the bottom edge of the element.")
    ]));
    keywords.insert("left", create_with_property_docs("left", "Left position or alignment.", &["background-position", "background-position-x", "transform-origin"], &[
        ("background-position", "Positions the background image at the left."),
        ("background-position-x", "Positions the background image at the left horizontally."),
        ("transform-origin", "Sets the transform origin to the left edge of the element.")
    ]));
    keywords.insert("right", create_with_property_docs("right", "Right position or alignment.", &["background-position", "background-position-x", "transform-origin"], &[
        ("background-position", "Positions the background image at the right."),
        ("background-position-x", "Positions the background image at the right horizontally."),
        ("transform-origin", "Sets the transform origin to the right edge of the element.")
    ]));
    
    // Coordinate system keywords
    keywords.insert("x", create("x", "Rotate around the X-axis.", &["rotate"]));
    keywords.insert("y", create("y", "Rotate around the Y-axis.", &["rotate"]));
    keywords.insert("z", create("z", "Rotate around the Z-axis.", &["rotate"]));
    keywords.insert("x-start", create("x-start", "Position the background image at the start of the horizontal axis (left edge).", &["background-position-x"]));
    keywords.insert("x-end", create("x-end", "Position the background image at the end of the horizontal axis (right edge).", &["background-position-x"]));
    keywords.insert("y-start", create("y-start", "Position the background image at the start of the vertical axis (top edge).", &["background-position-y"]));
    keywords.insert("y-end", create("y-end", "Position the background image at the end of the vertical axis (bottom edge).", &["background-position-y"]));
    
    // Unity slice type keywords
    keywords.insert("sliced", create("sliced", "The center of image is scaled instead of tiled.", &["-unity-slice-type"]));
    keywords.insert("tiled", create("tiled", "The center of image is tiled instead of scaled.", &["-unity-slice-type"]));
    
    // Animation easing keywords
    keywords.insert("ease-in-back", create("ease-in-back", "Ease-in with back overshoot at the beginning.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-bounce", create("ease-in-bounce", "Ease-in with bouncing effect at the beginning.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-circ", create("ease-in-circ", "Ease-in with circular acceleration.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-cubic", create("ease-in-cubic", "Ease-in with cubic acceleration.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-elastic", create("ease-in-elastic", "Ease-in with elastic effect at the beginning.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-sine", create("ease-in-sine", "Ease-in with sine wave acceleration.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-out-back", create("ease-out-back", "Ease-out with back overshoot at the end.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-out-bounce", create("ease-out-bounce", "Ease-out with bouncing effect at the end.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-out-circ", create("ease-out-circ", "Ease-out with circular deceleration.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-out-cubic", create("ease-out-cubic", "Ease-out with cubic deceleration.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-out-elastic", create("ease-out-elastic", "Ease-out with elastic effect at the end.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-out-sine", create("ease-out-sine", "Ease-out with sine wave deceleration.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-out-back", create("ease-in-out-back", "Ease-in-out with back overshoot at both ends.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-out-bounce", create("ease-in-out-bounce", "Ease-in-out with bouncing effect at both ends.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-out-circ", create("ease-in-out-circ", "Ease-in-out with circular acceleration and deceleration.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-out-cubic", create("ease-in-out-cubic", "Ease-in-out with cubic acceleration and deceleration.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-out-elastic", create("ease-in-out-elastic", "Ease-in-out with elastic effect at both ends.", &["transition-timing-function", "transition"]));
    keywords.insert("ease-in-out-sine", create("ease-in-out-sine", "Ease-in-out with sine wave acceleration and deceleration.", &["transition-timing-function", "transition"]));
    
    // Unity-specific keywords
    keywords.insert("ignored", create("ignored", "Ignores transitions for the specified duration, delay, and easing function.", &["transition-property"]));
    
    keywords
}
