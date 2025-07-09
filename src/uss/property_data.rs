//! USS Property Data
//!
//! Contains the actual property definitions for USS properties.
//! This module is separated from definitions.rs to improve maintainability.

use crate::uss::definitions::{PropertyInfo, ValueType};
use std::collections::HashMap;

/// Create all standard CSS properties supported by USS
pub fn create_standard_properties() -> HashMap<&'static str, PropertyInfo> {
    let mut properties = HashMap::new();
    
    // Standard CSS properties supported by USS with Unity documentation
    let standard_props = [
        PropertyInfo {
            name: "align-content",
            description: "Alignment of the whole area of children on the cross axis if they span over multiple lines in this container.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],

        },
        PropertyInfo {
            name: "align-items",
            description: "Alignment of children on the cross axis of this container.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            },
        PropertyInfo {
            name: "align-self",
            description: "Similar to align-items, but only for this specific element.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
        },
        PropertyInfo {
            name: "all",
            description: "Allows resetting all properties with the initial keyword. Does not apply to custom USS properties.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#all".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Keyword],
        },
        PropertyInfo {
            name: "background-color",
            description: "Background color to paint in the element's box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-background".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Color],
        },
        PropertyInfo {
            name: "background-image",
            description: "Background image to paint in the element's box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-background".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Resource, ValueType::Keyword],
        },
        PropertyInfo {
            name: "background-position",
            description: "Background image position value.",
            documentation_url: "https://developer.mozilla.org/en-US/docs/Web/CSS/background-position".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],
        },
        PropertyInfo {
            name: "background-position-x",
            description: "Background image x position value.",
            documentation_url: "https://developer.mozilla.org/en-US/docs/Web/CSS/background-position-x".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Length, ValueType::Keyword],
            },
        PropertyInfo {
            name: "background-position-y",
            description: "Background image y position value.",
            documentation_url: "https://developer.mozilla.org/en-US/docs/Web/CSS/background-position-y".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Length, ValueType::Keyword],
        },
        PropertyInfo {
            name: "background-repeat",
            description: "Background image repeat value.",
            documentation_url: "https://developer.mozilla.org/en-US/docs/Web/CSS/background-repeat".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
        },
        PropertyInfo {
            name: "background-size",
            description: "Background image size value. Transitions are fully supported only when using size in pixels or percentages.",
            documentation_url: "https://developer.mozilla.org/en-US/docs/Web/CSS/background-size".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],
        },
        PropertyInfo {
            name: "border-bottom-color",
            description: "Color of the element's bottom border.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#border-color".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Color],
        },
        PropertyInfo {
            name: "border-bottom-left-radius",
            description: "The radius of the bottom-left corner when a rounded rectangle is drawn in the element's box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#drawing-borders".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-bottom-right-radius",
            description: "The radius of the bottom-right corner when a rounded rectangle is drawn in the element's box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#drawing-borders".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-bottom-width",
            description: "Space reserved for the bottom edge of the border during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-color",
            description: "Shorthand for border-top-color, border-right-color, border-bottom-color, border-left-color",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#border-color".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Color],
            
        },
        PropertyInfo {
            name: "border-left-color",
            description: "Color of the element's left border.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#border-color".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Color],
            
        },
        PropertyInfo {
            name: "border-left-width",
            description: "Space reserved for the left edge of the border during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-radius",
            description: "Shorthand for border-top-left-radius, border-top-right-radius, border-bottom-right-radius, border-bottom-left-radius",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#drawing-borders".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-right-color",
            description: "Color of the element's right border.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#border-color".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Color],
        },
        PropertyInfo {
            name: "border-right-width",
            description: "Space reserved for the right edge of the border during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-top-color",
            description: "Color of the element's top border.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#border-color".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Color],
            
        },
        PropertyInfo {
            name: "border-top-left-radius",
            description: "The radius of the top-left corner when a rounded rectangle is drawn in the element's box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#drawing-borders".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-top-right-radius",
            description: "The radius of the top-right corner when a rounded rectangle is drawn in the element's box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#drawing-borders".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-top-width",
            description: "Space reserved for the top edge of the border during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],
        },
        PropertyInfo {
            name: "border-width",
            description: "Space reserved for the borders during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],

        },
        PropertyInfo {
            name: "bottom",
            description: "Bottom distance from the element's box during layout.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#positioning".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "color",
            description: "Color to use when drawing the text of an element.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: true,
            value_types: vec![ValueType::Color],
            
        },
        PropertyInfo {
            name: "cursor",
            description: "Mouse cursor to display when the mouse pointer is over an element.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#cursor".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Resource, ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "display",
            description: "Defines how an element is displayed in the layout.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#display".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "flex",
            description: "Shorthand for flex-grow, flex-shrink, flex-basis.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Number, ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "flex-basis",
            description: "Initial main size of a flex item, on the main flex axis.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "flex-direction",
            description: "Direction of the main axis to layout children in a container.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "flex-grow",
            description: "Specifies how the item will grow relative to the rest of the flexible items inside the same container.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "flex-shrink",
            description: "Specifies how the item will shrink relative to the rest of the flexible items inside the same container.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "flex-wrap",
            description: "Placement of children over multiple lines if not enough space is available in this container.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "font-size",
            description: "Font size to draw the element's text, specified in point size.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: true,
            value_types: vec![ValueType::Length],

        },
        PropertyInfo {
            name: "height",
            description: "Fixed height of an element for the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "justify-content",
            description: "Alignment of children on the main axis of this container.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#flex-layout".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "left",
            description: "Left distance from the element's box during layout.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#positioning".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "letter-spacing",
            description: "Increases or decreases the space between characters in text.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: true,
            value_types: vec![ValueType::Length],

        },
        PropertyInfo {
            name: "margin",
            description: "Shorthand for margin-top, margin-right, margin-bottom, margin-left",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "margin-bottom",
            description: "Space reserved for the bottom edge of the margin during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "margin-left",
            description: "Space reserved for the left edge of the margin during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "margin-right",
            description: "Space reserved for the right edge of the margin during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "margin-top",
            description: "Space reserved for the top edge of the margin during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "max-height",
            description: "Maximum height for an element, when it is flexible or measures its own size.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "max-width",
            description: "Maximum width for an element, when it is flexible or measures its own size.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "min-height",
            description: "Minimum height for an element, when it is flexible or measures its own size.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "min-width",
            description: "Minimum width for an element, when it is flexible or measures its own size.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "opacity",
            description: "Specifies the transparency of an element.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#opacity".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "overflow",
            description: "How a container behaves if its content overflows its own box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#appearance".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "padding",
            description: "Shorthand for padding-top, padding-right, padding-bottom, padding-left",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "padding-bottom",
            description: "Space reserved for the bottom edge of the padding during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "padding-left",
            description: "Space reserved for the left edge of the padding during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "padding-right",
            description: "Space reserved for the right edge of the padding during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "padding-top",
            description: "Space reserved for the top edge of the padding during the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "position",
            description: "Element positioning type during layout.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#positioning".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "right",
            description: "Right distance from the element's box during layout.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#positioning".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "rotate",
            description: "Rotation to apply to the element.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transform.html".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Angle],

        },
        PropertyInfo {
            name: "scale",
            description: "Scaling to apply to the element.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transform.html".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "text-overflow",
            description: "How hidden overflow content is signaled to users.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "text-shadow",
            description: "Adds shadow effects around a text.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Color],

        },
        PropertyInfo {
            name: "top",
            description: "Top distance from the element's box during layout.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#positioning".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "transform-origin",
            description: "Origin for the rotate, translate, and scale transforms.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transform.html".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "transition",
            description: "Shorthand for transition-delay, transition-duration, transition-property, transition-timing-function",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transitions.html".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Multiple(vec![ValueType::Keyword, ValueType::Number])],
            
        },
        PropertyInfo {
            name: "transition-delay",
            description: "Delay before a transition starts.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transitions.html".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "transition-duration",
            description: "Duration of a transition.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transitions.html".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "transition-property",
            description: "CSS properties that should transition.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transitions.html".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "transition-timing-function",
            description: "Timing function for a transition.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transitions.html".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "translate",
            description: "Translation to apply to the element.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transform.html".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length],

        },
        PropertyInfo {
            name: "visibility",
            description: "Specifies whether or not an element is visible.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#appearance".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "white-space",
            description: "How white space inside an element is handled.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "width",
            description: "Fixed width of an element for the layout phase.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#box-model".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Length, ValueType::Keyword],

        },
        PropertyInfo {
            name: "word-spacing",
            description: "Increases or decreases the space between words in text.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: true,
            value_types: vec![ValueType::Length],

        },
    ];
    
    for prop in standard_props {
        properties.insert(prop.name, prop);
    }
    
    properties
}

/// Create Unity-specific properties
pub fn create_unity_properties() -> HashMap<&'static str, PropertyInfo> {
    let mut properties = HashMap::new();
    
    // Unity-specific properties
    let unity_props = [
        PropertyInfo {
            name: "-unity-background-image-tint-color",
            description: "Tinting color for the background image.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-background".to_string(),
            inherited: false,
            animatable: true,
            value_types: vec![ValueType::Color],
            
        },
        PropertyInfo {
            name: "-unity-background-scale-mode",
            description: "How the background image is scaled in the element's box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-background".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "-unity-editor-text-rendering-mode",
            description: "Text rendering mode for the editor.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "-unity-font",
            description: "Font to use to display text.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-font".to_string(),
            inherited: true,
            animatable: false,
            value_types: vec![ValueType::Resource],
            
        },
        PropertyInfo {
            name: "-unity-font-definition",
            description: "Font asset to use to display text.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-font".to_string(),
            inherited: true,
            animatable: false,
            value_types: vec![ValueType::Resource],
            
        },
        PropertyInfo {
            name: "-unity-font-style",
            description: "Font style and weight (normal, bold, italic, or bold-and-italic).",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-font".to_string(),
            inherited: true,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "-unity-overflow-clip-box",
            description: "How content overflows are clipped.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#appearance".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "-unity-paragraph-spacing",
            description: "Space between paragraphs.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#appearance".to_string(),
            inherited: true,
            animatable: true,
            value_types: vec![ValueType::Length],

        },
        PropertyInfo {
            name: "-unity-slice-bottom",
            description: "Size of the 9-slice's bottom edge when painting an element's background image.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-slice".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "-unity-slice-left",
            description: "Size of the 9-slice's left edge when painting an element's background image.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-slice".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "-unity-slice-right",
            description: "Size of the 9-slice's right edge when painting an element's background image.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-slice".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "-unity-slice-scale",
            description: "Scaling of the 9-slice when painting an element's background image.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-slice".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "-unity-slice-top",
            description: "Size of the 9-slice's top edge when painting an element's background image.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-slice".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Number],
            
        },
        PropertyInfo {
            name: "-unity-slice-type",
            description: "Type of 9-slice when painting an element's background image.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-slice".to_string(),
            inherited: false,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "-unity-text-align",
            description: "Horizontal and vertical text alignment in the element's box.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "-unity-text-generator",
            description: "Text generator to use for text rendering.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
        PropertyInfo {
            name: "-unity-text-outline",
            description: "Text outline properties.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: true,
            value_types: vec![ValueType::Length],

        },
        PropertyInfo {
            name: "-unity-text-outline-color",
            description: "Color of the text outline.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: true,
            value_types: vec![ValueType::Color],
            
        },
        PropertyInfo {
            name: "-unity-text-outline-width",
            description: "Width of the text outline.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: true,
            value_types: vec![ValueType::Length],

        },
        PropertyInfo {
            name: "-unity-text-overflow-position",
            description: "Position where text overflow occurs.",
            documentation_url: "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html#unity-text".to_string(),
            inherited: true,
            animatable: false,
            value_types: vec![ValueType::Keyword],
            
        },
    ];
    
    for prop in unity_props {
        properties.insert(prop.name, prop);
    }
    
    properties
}