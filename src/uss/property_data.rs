//! USS Property Data
//!
//! Contains the actual property definitions for USS properties.
//! This module is separated from definitions.rs to improve maintainability.

use crate::uss::definitions::{PropertyInfo, PropertyAnimation};
use crate::uss::value_spec::{ValueType, ValueSpec, ValueFormat, ValueEntry};
use std::collections::HashMap;

/// Create all standard CSS properties supported by USS
pub fn create_standard_properties() -> HashMap<&'static str, PropertyInfo> {
    let mut properties = HashMap::new();
    
    let supported_properties_url = "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html";

    let css_url = "https://developer.mozilla.org/en-US/docs/Web/CSS";

    let transform_url = "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transform.html";

    let transitions_url = "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transitions.html";

    let standard_props = [
        PropertyInfo {
            name: "align-content",
            description: "Alignment of the whole area of children on the cross axis if they span over multiple lines in this container.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["flex-start", "flex-end", "center", "space-between", "space-around", "stretch"]),
        },
        PropertyInfo {
            name: "align-items",
            description: "Alignment of children on the cross axis of this container.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["flex-start", "flex-end", "center", "baseline", "stretch"]),
        },
        PropertyInfo {
            name: "align-self",
            description: "Similar to align-items, but only for this specific element.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["auto", "flex-start", "flex-end", "center", "baseline", "stretch"]),
        },
        PropertyInfo {
            name: "all",
            description: "Allows resetting all properties with the initial keyword. Does not apply to custom USS properties.",
            documentation_url: format!("{supported_properties_url}#all"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::keywords(&["initial"]),
        },
        PropertyInfo {
            name: "background-color",
            description: "Background color to paint in the element's box.",
            documentation_url: format!("{supported_properties_url}#unity-background"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "background-image",
            description: "Background image to paint in the element's box.",
            documentation_url: format!("{supported_properties_url}#unity-background"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::one_of(vec![ValueType::Asset, ValueType::Keyword("none")]),
        },
        PropertyInfo {
            name: "background-position",
            description: "Background image position value.",
            documentation_url: format!("{css_url}/background-position"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::sequence(vec![ValueType::Length, ValueType::Length]),
        },
        PropertyInfo {
            name: "background-position-x",
            description: "Background image x position value.",
            documentation_url: format!("{css_url}/background-position-x"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "background-position-y",
            description: "Background image y position value.",
            documentation_url: format!("{css_url}/background-position-y"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "background-repeat",
            description: "Background image repeat value.",
            documentation_url: format!("{css_url}/background-repeat"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["repeat", "repeat-x", "repeat-y", "no-repeat", "space", "round"]),
        },
        PropertyInfo {
            name: "background-size",
            description: "Background image size value. Transitions are fully supported only when using size in pixels or percentages, such as pixel-to-pixel or percentage-to-percentage transitions.",
            documentation_url: format!("{css_url}/background-size"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::multiple_formats(vec![
                ValueFormat { entries: vec![ValueEntry { types: vec![ValueType::Keyword("cover"), ValueType::Keyword("contain"), ValueType::Keyword("auto")] }] }, // cover, contain, auto
                ValueFormat { entries: vec![ValueEntry { types: vec![ValueType::Length] }] }, // single length
                ValueFormat { entries: vec![
                    ValueEntry { types: vec![ValueType::Length] },
                    ValueEntry { types: vec![ValueType::Length] }
                ] }, // width height
            ]),
        },
        PropertyInfo {
            name: "border-bottom-color",
            description: "Color of the element's bottom border.",
            documentation_url: format!("{supported_properties_url}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "border-bottom-left-radius",
            description: "The radius of the bottom-left corner when a rounded rectangle is drawn in the element's box.",
            documentation_url: format!("{supported_properties_url}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "border-bottom-right-radius",
            description: "The radius of the bottom-right corner when a rounded rectangle is drawn in the element's box.",
            documentation_url: format!("{supported_properties_url}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "border-bottom-width",
            description: "Space reserved for the bottom edge of the border during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "border-color",
            description: "Shorthand for border-top-color, border-right-color, border-bottom-color, border-left-color",
            documentation_url: format!("{supported_properties_url}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::Color, 1, 4),
        },
        PropertyInfo {
            name: "border-left-color",
            description: "Color of the element's left border.",
            documentation_url: format!("{supported_properties_url}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "border-left-width",
            description: "Space reserved for the left edge of the border during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "border-radius",
            description: "Shorthand for border-top-left-radius, border-top-right-radius, border-bottom-right-radius, border-bottom-left-radius",
            documentation_url: format!("{supported_properties_url}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::Length, 1, 4),
        },
        PropertyInfo {
            name: "border-right-color",
            description: "Color of the element's right border.",
            documentation_url: format!("{supported_properties_url}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "border-top-left-radius",
            description: "The radius of the top-left corner when a rounded rectangle is drawn in the element's box.",
            documentation_url: format!("{supported_properties_url}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "border-top-color",
            description: "Color of the element's top border.",
            documentation_url: format!("{supported_properties_url}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "border-right-width",
            description: "Space reserved for the right edge of the border during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "border-top-right-radius",
            description: "The radius of the top-right corner when a rounded rectangle is drawn in the element's box.",
            documentation_url: format!("{supported_properties_url}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "border-top-width",
            description: "Space reserved for the top edge of the border during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "border-width",
            description: "Shorthand for border-top-width, border-right-width, border-bottom-width, border-left-width",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::Length, 1, 4),
        },
        PropertyInfo {
            name: "bottom",
            description: "Bottom distance from the element's box during layout.",
            documentation_url: format!("{supported_properties_url}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "color",
            description: "Color to use when drawing the text of an element.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "cursor",
            description: "Mouse cursor to display when the mouse pointer is over an element.",
            documentation_url: format!("{supported_properties_url}#cursor"),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::multiple_formats(vec![
                // Built-in cursor keywords
                ValueFormat::keywords(&[
                    "arrow", "text", "resize-vertical", "resize-horizontal",
                    "link", "slide-arrow", "resize-up-right", "resize-up-left",
                    "move-arrow", "rotate-arrow", "scale-arrow", "arrow-plus",
                    "arrow-minus", "pan", "orbit", "zoom", "fps", "split-resize-up-down",
                    "split-resize-left-right"
                ]),
                // Custom cursor: resource/url + optional hotspot coordinates
                ValueFormat::sequence(vec![ValueType::Asset, ValueType::Number, ValueType::Number]),
                // Custom cursor: resource/url only
                ValueFormat::single(ValueType::Asset)
            ]),
        },
        PropertyInfo {
            name: "display",
            description: "Defines how an element is displayed in the layout.",
            documentation_url: format!("{supported_properties_url}#appearance"),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&["flex", "none"]),
        },
        PropertyInfo {
            name: "flex",
            description: "Shorthand for flex-grow, flex-shrink, flex-basis.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::multiple_formats(vec![
                ValueFormat::keywords(&["none"]), 
                ValueFormat::single(ValueType::Number), // flex-grow only
                ValueFormat::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]), // flex-basis only
                ValueFormat::sequence(vec![ValueType::Number, ValueType::Number]), // flex-grow flex-shrink
                ValueFormat::sequence(vec![ValueType::Number, ValueType::Length]), // flex-grow flex-basis (length)
                ValueFormat::sequence(vec![ValueType::Number, ValueType::Keyword("auto")]), // flex-grow flex-basis (auto)
                ValueFormat::sequence(vec![ValueType::Number, ValueType::Number, ValueType::Length]), // flex-grow flex-shrink flex-basis (length)
                ValueFormat::sequence(vec![ValueType::Number, ValueType::Number, ValueType::Keyword("auto")]), // flex-grow flex-shrink flex-basis (auto)
            ]),
        },
        PropertyInfo {
            name: "flex-basis",
            description: "Initial main size of a flex item, on the main flex axis. The final layout might be smaller or larger, according to the flex shrinking and growing determined by the other flex properties.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "flex-direction",
            description: "Direction of the main axis to layout children in a container.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["row", "row-reverse", "column", "column-reverse"]),
        },
        PropertyInfo {
            name: "flex-grow",
            description: "Specifies how the item will grow relative to the rest of the flexible items inside the same container.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "flex-shrink",
            description: "Specifies how the item will shrink relative to the rest of the flexible items inside the same container.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "flex-wrap",
            description: "Placement of children over multiple lines if not enough space is available in this container.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["nowrap", "wrap", "wrap-reverse"]),
        },
        PropertyInfo {
            name: "font-size",
            description: "Font size to draw the element's text, specified in point size.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "height",
            description: "Fixed height of an element for the layout.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "justify-content",
            description: "Justification of children on the main axis of this container.",
            documentation_url: format!("{supported_properties_url}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["flex-start", "flex-end", "center", "space-between", "space-around"]),
        },
        PropertyInfo {
            name: "left",
            description: "Left distance from the element's box during layout.",
            documentation_url: format!("{supported_properties_url}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "letter-spacing",
            description: "Increases or decreases the space between characters.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "margin",
            description: "Shorthand for margin-top, margin-right, margin-bottom, margin-left",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::Length, 1, 4),
        },
        PropertyInfo {
            name: "margin-bottom",
            description: "Space reserved for the bottom edge of the margin during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "margin-left",
            description: "Space reserved for the left edge of the margin during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "margin-right",
            description: "Space reserved for the right edge of the margin during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "margin-top",
            description: "Space reserved for the top edge of the margin during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "max-height",
            description: "Maximum height for an element, when it is flexible or measures its own size.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "max-width",
            description: "Maximum width for an element, when it is flexible or measures its own size.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("none")]),
        },
        PropertyInfo {
            name: "min-height",
            description: "Minimum height for an element, when it is flexible or measures its own size.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "min-width",
            description: "Minimum width for an element, when it is flexible or measures its own size.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "opacity",
            description: "Specifies the transparency of an element and of its children.",
            documentation_url: format!("{supported_properties_url}#opacity"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "overflow",
            description: "How a container behaves if its content overflows its own box.",
            documentation_url: format!("{supported_properties_url}#appearance"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["visible", "hidden", "scroll"]),
        },
        PropertyInfo {
            name: "padding",
            description: "Shorthand for padding-top, padding-right, padding-bottom, padding-left",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::Length, 1, 4),
        },
        PropertyInfo {
            name: "padding-bottom",
            description: "Space reserved for the bottom edge of the padding during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "padding-left",
            description: "Space reserved for the left edge of the padding during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "padding-right",
            description: "Space reserved for the right edge of the padding during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "padding-top",
            description: "Space reserved for the top edge of the padding during the layout phase.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "position",
            description: "Element's positioning in its parent container.",
            documentation_url: format!("{supported_properties_url}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["relative", "absolute"]),
        },
        PropertyInfo {
            name: "right",
            description: "Right distance from the element's box during layout.",
            documentation_url: format!("{supported_properties_url}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "rotate",
            description: "A rotation transformation.",
            documentation_url: transform_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Angle),
        },
        PropertyInfo {
            name: "scale",
            description: "A scaling transformation.",
            documentation_url: transform_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "text-overflow",
            description: "The element's text overflow mode.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["clip", "ellipsis"]),
        },
        PropertyInfo {
            name: "text-shadow",
            description: "Drop shadow of the text.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::sequence(vec![ValueType::Length, ValueType::Length, ValueType::Length, ValueType::Color]),
        },
        PropertyInfo {
            name: "top",
            description: "Top distance from the element's box during layout.",
            documentation_url: format!("{supported_properties_url}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "transform-origin",
            description: "The transformation origin is the point around which a transformation is applied.",
            documentation_url: transform_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::sequence(vec![ValueType::Length, ValueType::Length]),
        },
        PropertyInfo {
            name: "transition",
            description: "Shorthand for transition-delay, transition-duration, transition-property, transition-timing-function",
            documentation_url: transitions_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::multiple_formats(vec![
                ValueFormat { entries: vec![
                    ValueEntry { types: vec![ValueType::PropertyName] },
                    ValueEntry { types: vec![ValueType::Time] },
                    ValueEntry { types: vec![ValueType::Keyword("ease"), ValueType::Keyword("linear"), ValueType::Keyword("ease-in"), ValueType::Keyword("ease-out"), ValueType::Keyword("ease-in-out")] },
                    ValueEntry { types: vec![ValueType::Time] }
                ] },
                ValueFormat { entries: vec![
                    ValueEntry { types: vec![ValueType::PropertyName] },
                    ValueEntry { types: vec![ValueType::Time] },
                    ValueEntry { types: vec![ValueType::Keyword("ease"), ValueType::Keyword("linear"), ValueType::Keyword("ease-in"), ValueType::Keyword("ease-out"), ValueType::Keyword("ease-in-out")] }
                ] },
                ValueFormat { entries: vec![
                    ValueEntry { types: vec![ValueType::PropertyName] },
                    ValueEntry { types: vec![ValueType::Time] }
                ] },
                ValueFormat { entries: vec![ValueEntry { types: vec![ValueType::Keyword("all"), ValueType::Keyword("none")] }] }
            ]),
        },
        PropertyInfo {
            name: "transition-delay",
            description: "Duration to wait before starting a property's transition effect when its value changes.",
            documentation_url: transitions_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::single(ValueType::Time),
        },
        PropertyInfo {
            name: "transition-duration",
            description: "Time a transition animation should take to complete.",
            documentation_url: transitions_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::single(ValueType::Time),
        },
        PropertyInfo {
            name: "transition-property",
            description: "Properties to which a transition effect should be applied.",
            documentation_url: transitions_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&["all", "none"]),
        },
        PropertyInfo {
            name: "transition-timing-function",
            description: "Determines how intermediate values are calculated for properties modified by a transition effect.",
            documentation_url: transitions_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&["ease", "ease-in", "ease-out", "ease-in-out", "linear"]),
        },
        PropertyInfo {
            name: "translate",
            description: "A translate transformation.",
            documentation_url: transform_url.to_string(),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::sequence(vec![ValueType::Length, ValueType::Length]),
        },
        PropertyInfo {
            name: "-unity-background-image-tint-color",
            description: "Tinting color for the element's backgroundImage.",
            documentation_url: format!("{supported_properties_url}#unity-background"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "-unity-background-scale-mode",
            description: "Background image scaling in the element's box.",
            documentation_url: format!("{supported_properties_url}#unity-background"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["stretch-to-fill", "scale-and-crop", "scale-to-fit"]),
        },
        PropertyInfo {
            name: "-unity-editor-text-rendering-mode",
            description: "TextElement editor rendering mode.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&["legacy", "distance-field"]),
        },
        PropertyInfo {
            name: "-unity-font",
            description: "Font to draw the element's text, defined as a Font object.",
            documentation_url: format!("{supported_properties_url}#unity-font"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::single(ValueType::Asset),
        },
        PropertyInfo {
            name: "-unity-font-definition",
            description: "Font to draw the element's text, defined as a FontDefinition structure. It takes precedence over -unity-font.",
            documentation_url: format!("{supported_properties_url}#unity-font"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::single(ValueType::Asset),
        },
        PropertyInfo {
            name: "-unity-font-style",
            description: "Font style and weight (normal, bold, italic) to draw the element's text.",
            documentation_url: format!("{supported_properties_url}#unity-font"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["normal", "bold", "italic", "bold-and-italic"]),
        },
        PropertyInfo {
            name: "-unity-overflow-clip-box",
            description: "Specifies which box the element content is clipped against.",
            documentation_url: format!("{supported_properties_url}#appearance"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["padding-box", "content-box"]),
        },
        PropertyInfo {
            name: "-unity-paragraph-spacing",
            description: "Increases or decreases the space between paragraphs.",
            documentation_url: format!("{supported_properties_url}#appearance"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "-unity-slice-bottom",
            description: "Size of the 9-slice's bottom edge when painting an element's background image.",
            documentation_url: format!("{supported_properties_url}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "-unity-slice-left",
            description: "Size of the 9-slice's left edge when painting an element's background image.",
            documentation_url: format!("{supported_properties_url}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "-unity-slice-right",
            description: "Size of the 9-slice's right edge when painting an element's background image.",
            documentation_url: format!("{supported_properties_url}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "-unity-slice-scale",
            description: "Scale applied to an element's slices.",
            documentation_url: format!("{supported_properties_url}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "-unity-slice-top",
            description: "Size of the 9-slice's top edge when painting an element's background image.",
            documentation_url: format!("{supported_properties_url}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "-unity-slice-type",
            description: "Specifies the type of sclicing.",
            documentation_url: format!("{supported_properties_url}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["stretch", "tile", "mirror"]),
        },
        PropertyInfo {
            name: "-unity-text-align",
            description: "Horizontal and vertical text alignment in the element's box.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["upper-left", "middle-left", "lower-left", "upper-center", "middle-center", "lower-center", "upper-right", "middle-right", "lower-right"]),
        },
        PropertyInfo {
            name: "-unity-text-generator",
            description: "Switches between Unity's standard and advanced text generator",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&["standard", "advanced"]),
        },
        PropertyInfo {
            name: "-unity-text-outline",
            description: "Outline width and color of the text.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Color]),
        },
        PropertyInfo {
            name: "-unity-text-outline-color",
            description: "Outline color of the text.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "-unity-text-outline-width",
            description: "Outline width of the text.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
        PropertyInfo {
            name: "-unity-text-overflow-position",
            description: "The element's text overflow position.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["start", "middle", "end"]),
        },
        PropertyInfo {
            name: "visibility",
            description: "Specifies whether or not an element is visible.",
            documentation_url: format!("{supported_properties_url}#appearance"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["visible", "hidden"]),
        },
        PropertyInfo {
            name: "white-space",
            description: "Word wrap over multiple lines if not enough space is available to draw the text of an element.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["normal", "nowrap"]),
        },
        PropertyInfo {
            name: "width",
            description: "Fixed width of an element for the layout.",
            documentation_url: format!("{supported_properties_url}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Length, ValueType::Keyword("auto")]),
        },
        PropertyInfo {
            name: "word-spacing",
            description: "Increases or decreases the space between words.",
            documentation_url: format!("{supported_properties_url}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Length),
        },
    ];
    
    // Add 'initial' keyword support to all properties (since Unity supports this value for all properties)
    let mut enhanced_props = Vec::new();
    for mut prop in standard_props {
        // Determine how to add 'initial' based on the property's value specification
        if prop.value_spec.is_single_format_and_entry() {
            // For single format and entry properties, add 'initial' to the entry
            if let Some(format) = prop.value_spec.formats.get_mut(0) {
                if let Some(entry) = format.entries.get_mut(0) {
                    if !entry.types.contains(&ValueType::Keyword("initial")) {
                        entry.types.push(ValueType::Keyword("initial"));
                    }
                }
            }
        } else {
            // For other properties, add a separate format that accepts only 'initial'
            let initial_format = ValueFormat {
                entries: vec![ValueEntry {
                    types: vec![ValueType::Keyword("initial")],
                }],
            };
            prop.value_spec.formats.push(initial_format);
        }
        
        enhanced_props.push(prop);
    }

    for prop in enhanced_props {
        properties.insert(prop.name, prop);
    }
    
    properties
}