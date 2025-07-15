//! USS Property Data
//!
//! Contains the actual property definitions for USS properties.
//! This module is separated from definitions.rs to improve maintainability.

use crate::uss::definitions::{PropertyAnimation, PropertyInfo};
use crate::uss::flexible_format::FlexibleFormatBuilder;
use crate::uss::value_spec::{ValueEntry, ValueFormat, ValueSpec, ValueType};
use std::collections::HashMap;
const SUPPORTED_PROPERTIES_URL: &str =
    "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-USS-SupportedProperties.html";

const CSS_URL: &str = "https://developer.mozilla.org/en-US/docs/Web/CSS";

const TRANSFORM_URL: &str =
    "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transform.html";

const TRANSITIONS_URL: &str =
    "https://docs.unity3d.com/{version}/Documentation/Manual/UIE-Transitions.html";

const TIMING_FUN: [&'static str; 23] = [
    "ease",
    "ease-in",
    "ease-out",
    "ease-in-out",
    "linear",
    "ease-in-sine",
    "ease-out-sine",
    "ease-in-out-sine",
    "ease-in-cubic",
    "ease-out-cubic",
    "ease-in-out-cubic",
    "ease-in-circ",
    "ease-out-circ",
    "ease-in-out-circ",
    "ease-in-elastic",
    "ease-out-elastic",
    "ease-in-out-elastic",
    "ease-in-back",
    "ease-out-back",
    "ease-in-out-back",
    "ease-in-bounce",
    "ease-out-bounce",
    "ease-in-out-bounce",
];

/// Create all standard CSS properties supported by USS
pub fn create_standard_properties() -> HashMap<&'static str, PropertyInfo> {
    let mut properties = HashMap::new();

    let spec_length_auto = ValueSpec::one_of(vec![ValueType::LengthOrPercentage, ValueType::Keyword("auto")]);
    
    let standard_props = [
        PropertyInfo {
            name: "align-content",
            description: "Alignment of the whole area of children on the cross axis if they span over multiple lines in this container.",
            examples_unity: None,
            examples_mozilla: None,
            format: "flex-start | flex-end | center | stretch",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["flex-start", "flex-end", "center", "stretch"]),
        },
        PropertyInfo {
            name: "align-items",
            description: "Alignment of children on the cross axis of this container.",
            examples_unity: None,
            examples_mozilla: None,
            format: "auto | flex-start | flex-end | center | stretch",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&[
                "auto",
                "flex-start",
                "flex-end",
                "center",
                "stretch",
            ]),
        },
        PropertyInfo {
            name: "align-self",
            description: "Similar to align-items, but only for this specific element.",
            examples_unity: None,
            examples_mozilla: None,
            format: "auto | flex-start | flex-end | center | stretch",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&[
                "auto",
                "flex-start",
                "flex-end",
                "center",
                "stretch",
            ]),
        },
        PropertyInfo {
            name: "all",
            description: "Allows resetting all properties with the initial keyword. Does not apply to custom USS properties.",
            examples_unity: Some("all: initial"),
            examples_mozilla: None,
            format: "initial",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#all"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::keywords(&["initial"]),
        },
        PropertyInfo {
            name: "background-color",
            description: "Background color to paint in the element's box.",
            examples_unity: Some("background-color: red;\nbackground-color: blue;"),
            examples_mozilla: None,
            format: "<color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-background"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "background-image",
            description: "Background image to paint in the element's box.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<resource> | <url> | none",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-background"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::one_of(vec![ValueType::Asset, ValueType::Keyword("none")]),
        },
        PropertyInfo {
            name: "background-position",
            description: "Background image position value.",
            examples_unity: None,
            examples_mozilla: Some("background-position: top;\nbackground-position: bottom;\nbackground-position: left;\nbackground-position: right;\nbackground-position: center;\nbackground-position: 25% 75%;\nbackground-position: 0 0;\nbackground-position: 1cm 2cm;\nbackground-position: 10ch 8em;\nbackground-position:\nbackground-position: bottom 10px right 20px;\nbackground-position: right 3em bottom 10px;\nbackground-position: bottom 10px right;\nbackground-position: top right 10px;\nbackground-position: inherit;\nbackground-position: initial;\nbackground-position: revert;\nbackground-position: revert-layer;\nbackground-position: unset;\nbackground-position:"),
            format: 
                "[ left | center | right | top | bottom | <length> ]  |  [ left | center | right | <length> ] [ top | center | bottom | <length> ]  |  [ center | [ left | right ] <length>? ] && [ center | [ top | bottom ] <length>? ] ",
            documentation_url: format!("{CSS_URL}/background-position"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::new(create_formats_for_background_position()),
        },
        PropertyInfo {
            name: "background-position-x",
            description: "Background image x position value.",
            examples_unity: None,
            examples_mozilla: Some("background-position-x: left;\nbackground-position-x: center;\nbackground-position-x: right;\nbackground-position-x: 25%;\nbackground-position-x: 0px;\nbackground-position-x: 1cm;\nbackground-position-x: 8em;\nbackground-position-x: right 3px;\nbackground-position-x: left 25%;\nbackground-position-x: 0px, center;\nbackground-position-x: inherit;\nbackground-position-x: initial;\nbackground-position-x: revert;\nbackground-position-x: revert-layer;\nbackground-position-x: unset;\nbackground-position-x: center;\nbackground-position-x: right 20px;"),
            format: "[ center | [ [ left | right | x-start | x-end ]? <length>? ]! ]#",
            documentation_url: format!("{CSS_URL}/background-position-x"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::new(create_formats_for_background_position_x()),
        },
        PropertyInfo {
            name: "background-position-y",
            description: "Background image y position value.",
            examples_unity: None,
            examples_mozilla: Some("background-position-y: top;\nbackground-position-y: center;\nbackground-position-y: bottom;\nbackground-position-y: 25%;\nbackground-position-y: 0px;\nbackground-position-y: 1cm;\nbackground-position-y: 8em;\nbackground-position-y: bottom 3px;\nbackground-position-y: bottom 10%;\nbackground-position-y: 0px, center;\nbackground-position-y: inherit;\nbackground-position-y: initial;\nbackground-position-y: revert;\nbackground-position-y: revert-layer;\nbackground-position-y: unset;\nbackground-position-y: bottom;\nbackground-position-y: bottom 10px;"),
            format: "[ center | [ [ top | bottom | y-start | y-end ]? <length>? ]! ]#",
            documentation_url: format!("{CSS_URL}/background-position-y"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::new(create_formats_for_background_position_y()),
        },
        PropertyInfo {
            name: "background-repeat",
            description: "Background image repeat value.",
            examples_unity: None,
            examples_mozilla: Some("background-repeat: repeat;\nbackground-repeat: repeat-x;\nbackground-repeat: repeat-y;\nbackground-repeat: space;\nbackground-repeat: round;\nbackground-repeat: no-repeat;\nbackground-repeat: repeat space;\nbackground-repeat: repeat repeat;\nbackground-repeat: round space;\nbackground-repeat: no-repeat round;\nbackground-repeat: inherit;\nbackground-repeat: initial;\nbackground-repeat: revert;\nbackground-repeat: revert-layer;\nbackground-repeat: unset;\nbackground-repeat: no-repeat;\nbackground-repeat: repeat;\nbackground-repeat: repeat-x;\nbackground-repeat: repeat-y;\nbackground-repeat: space;\nbackground-repeat: round;\nbackground-repeat: repeat-x, repeat-y;"),
            format: "repeat-x | repeat-y | [ repeat | space | round | no-repeat ]{1,2}",
            documentation_url: format!("{CSS_URL}/background-repeat"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::new(create_formats_for_background_repeat()),
        },
        PropertyInfo {
            name: "background-size",
            description: "Background image size value. Transitions are fully supported only when using size in pixels or percentages, such as pixel-to-pixel or percentage-to-percentage transitions.",
            examples_unity: None,
            examples_mozilla: Some("background-size: cover;\nbackground-size: contain;\nbackground-size: 50%;\nbackground-size: 3.2em;\nbackground-size: 12px;\nbackground-size: auto;\nbackground-size: 50% auto;\nbackground-size: 3em 25%;\nbackground-size: auto 6px;\nbackground-size: auto auto;\nbackground-size: auto, auto; /* Not to be confused with `auto auto` */\nbackground-size: 50%, 25%, 25%;\nbackground-size: 6px, auto, contain;\nbackground-size: inherit;\nbackground-size: initial;\nbackground-size: revert;\nbackground-size: revert-layer;\nbackground-size: unset;\nbackground-size: 150px;"),
            format: "[ <length> | auto ]{1,2} | cover | contain",
            documentation_url: format!("{CSS_URL}/background-size"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::new(create_formats_for_background_size()),
        },
        PropertyInfo {
            name: "border-bottom-color",
            description: "Color of the element's bottom border.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "border-bottom-left-radius",
            description: "The radius of the bottom-left corner when a rounded rectangle is drawn in the element's box.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "border-bottom-right-radius",
            description: "The radius of the bottom-right corner when a rounded rectangle is drawn in the element's box.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "border-bottom-width",
            description: "Space reserved for the bottom edge of the border during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "border-color",
            description: "Shorthand for border-top-color, border-right-color, border-bottom-color, border-left-color",
            examples_unity: None,
            examples_mozilla: None,
            format: "<color>{1,4}",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::Color, 1, 4),
        },
        PropertyInfo {
            name: "border-left-color",
            description: "Color of the element's left border.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "border-left-width",
            description: "Space reserved for the left edge of the border during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "border-radius",
            description: "Shorthand for border-top-left-radius, border-top-right-radius, border-bottom-right-radius, border-bottom-left-radius",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>{1,4}",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::LengthOrPercentage, 1, 4),
        },
        PropertyInfo {
            name: "border-right-color",
            description: "Color of the element's right border.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "border-top-left-radius",
            description: "The radius of the top-left corner when a rounded rectangle is drawn in the element's box.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "border-top-color",
            description: "Color of the element's top border.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#border-color"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "border-right-width",
            description: "Space reserved for the right edge of the border during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "border-top-right-radius",
            description: "The radius of the top-right corner when a rounded rectangle is drawn in the element's box.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#drawing-borders"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "border-top-width",
            description: "Space reserved for the top edge of the border during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "border-width",
            description: "Shorthand for border-top-width, border-right-width, border-bottom-width, border-left-width",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>{1,4}",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::LengthOrPercentage, 1, 4),
        },
        PropertyInfo {
            name: "bottom",
            description: "Bottom distance from the element's box during layout.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "color",
            description: "Color to use when drawing the text of an element.",
            examples_unity: Some("background-color: red;\nbackground-color: blue;\ncolor: black;\ncolor: red;"),
            examples_mozilla: None,
            format: "<color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "cursor",
            description: "Mouse cursor to display when the mouse pointer is over an element.",
            examples_unity: None,
            examples_mozilla: None,
            format: 
                "[ [ <resource> | <url> ] [ <integer> <integer>]? , ] [ arrow | text | resize-vertical | resize-horizontal | link | slide-arrow | resize-up-right | resize-up-left | move-arrow | rotate-arrow | scale-arrow | arrow-plus | arrow-minus | pan | orbit | zoom | fps | split-resize-up-down | split-resize-left-right ]",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#cursor"),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::new(vec![
                // Built-in cursor keywords
                ValueFormat::keywords(&[
                    "arrow",
                    "text",
                    "resize-vertical",
                    "resize-horizontal",
                    "link",
                    "slide-arrow",
                    "resize-up-right",
                    "resize-up-left",
                    "move-arrow",
                    "rotate-arrow",
                    "scale-arrow",
                    "arrow-plus",
                    "arrow-minus",
                    "pan",
                    "orbit",
                    "zoom",
                    "fps",
                    "split-resize-up-down",
                    "split-resize-left-right",
                ]),
                // Custom cursor: resource/url + optional hotspot coordinates
                ValueFormat::sequence(vec![
                    ValueType::Asset,
                    ValueType::Integer,
                    ValueType::Integer,
                ]),
                // Custom cursor: resource/url only
                ValueFormat::single(ValueType::Asset),
            ]),
        },
        PropertyInfo {
            name: "display",
            description: "Defines how an element is displayed in the layout.",
            examples_unity: None,
            examples_mozilla: None,
            format: "flex | none",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#appearance"),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&["flex", "none"]),
        },
        PropertyInfo {
            name: "flex",
            description: "Shorthand for flex-grow, flex-shrink, flex-basis.",
            examples_unity: None,
            examples_mozilla: None,
            format: "none | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::new(create_flex_formats()),
        },
        PropertyInfo {
            name: "flex-basis",
            description: "Initial main size of a flex item, on the main flex axis. The final layout might be smaller or larger, according to the flex shrinking and growing determined by the other flex properties.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "flex-direction",
            description: "Direction of the main axis to layout children in a container.",
            examples_unity: None,
            examples_mozilla: None,
            format: "row | row-reverse | column | column-reverse",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["row", "row-reverse", "column", "column-reverse"]),
        },
        PropertyInfo {
            name: "flex-grow",
            description: "Specifies how the item will grow relative to the rest of the flexible items inside the same container.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<number>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "flex-shrink",
            description: "Specifies how the item will shrink relative to the rest of the flexible items inside the same container.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<number>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "flex-wrap",
            description: "Placement of children over multiple lines if not enough space is available in this container.",
            examples_unity: None,
            examples_mozilla: None,
            format: "nowrap | wrap | wrap-reverse",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["nowrap", "wrap", "wrap-reverse"]),
        },
        PropertyInfo {
            name: "font-size",
            description: "Font size to draw the element's text, specified in point size. Actually, the format is `<length>`.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<number>", // this is wrong, actual format is `<length>`, but we will keep it the same as offcial docs
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "height",
            description: "Fixed height of an element for the layout.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "justify-content",
            description: "Justification of children on the main axis of this container.",
            examples_unity: None,
            examples_mozilla: None,
            format: "flex-start | flex-end | center | space-between | space-around",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#flex-layout"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&[
                "flex-start",
                "flex-end",
                "center",
                "space-between",
                "space-around",
            ]),
        },
        PropertyInfo {
            name: "left",
            description: "Left distance from the element's box during layout.",
            examples_unity: Some("left: 20px;\nleft: 50px;\nleft: 0px;"),
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "letter-spacing",
            description: "Increases or decreases the space between characters.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "margin",
            description: "Shorthand for margin-top, margin-right, margin-bottom, margin-left",
            examples_unity: None,
            examples_mozilla: None,
            format: "[<length> | auto]{1,4}",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::new(
                FlexibleFormatBuilder::new()
                    .range(
                        ValueEntry::new(vec![ValueType::LengthOrPercentage, ValueType::Keyword("auto")]),
                        1,
                        4,
                    )
                    .build(),
            ),
        },
        PropertyInfo {
            name: "margin-bottom",
            description: "Space reserved for the bottom edge of the margin during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "margin-left",
            description: "Space reserved for the left edge of the margin during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "margin-right",
            description: "Space reserved for the right edge of the margin during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "margin-top",
            description: "Space reserved for the top edge of the margin during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "max-height",
            description: "Maximum height for an element, when it is flexible or measures its own size.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | none",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::LengthOrPercentage, ValueType::Keyword("none")]),
        },
        PropertyInfo {
            name: "max-width",
            description: "Maximum width for an element, when it is flexible or measures its own size.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | none",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::LengthOrPercentage, ValueType::Keyword("none")]),
        },
        PropertyInfo {
            name: "min-height",
            description: "Minimum height for an element, when it is flexible or measures its own size.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "min-width",
            description: "Minimum width for an element, when it is flexible or measures its own size.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "opacity",
            description: "Specifies the transparency of an element and of its children.",
            examples_unity: Some("opacity: 0.5;"),
            examples_mozilla: None,
            format: "<number>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#opacity"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Number),
        },
        PropertyInfo {
            name: "overflow",
            description: "How a container behaves if its content overflows its own box.",
            examples_unity: None,
            examples_mozilla: None,
            format: "hidden | visible",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#appearance"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["visible", "hidden"]),
        },
        PropertyInfo {
            name: "padding",
            description: "Shorthand for padding-top, padding-right, padding-bottom, padding-left",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>{1,4}",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::repeat(ValueType::LengthOrPercentage, 1, 4),
        },
        PropertyInfo {
            name: "padding-bottom",
            description: "Space reserved for the bottom edge of the padding during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "padding-left",
            description: "Space reserved for the left edge of the padding during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "padding-right",
            description: "Space reserved for the right edge of the padding during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "padding-top",
            description: "Space reserved for the top edge of the padding during the layout phase.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "position",
            description: "Element's positioning in its parent container.",
            examples_unity: None,
            examples_mozilla: None,
            format: "absolute | relative",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["relative", "absolute"]),
        },
        PropertyInfo {
            name: "right",
            description: "Right distance from the element's box during layout.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "rotate",
            description: "A rotation transformation.",
            examples_unity: Some("rotate: 45deg;\nrotate: -100grad;\nrotate: -3.14rad;\nrotate: 0.75turn;\nrotate: none;"),
            examples_mozilla: None,
            format: "none | [ x | y | z | <number>{3} ] && <angle> | <angle>",
            documentation_url: TRANSFORM_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::one_of(vec![ValueType::Angle, ValueType::Keyword("none")]),
        },
        PropertyInfo {
            name: "scale",
            description: "A scaling transformation.",
            examples_unity: Some("scale: 2.5;\nscale: -1 1;\nscale: none;"),
            examples_mozilla: None,
            format: "none | <number>{1,3}",
            documentation_url: TRANSFORM_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::new(create_scale_formats()),
        },
        PropertyInfo {
            name: "text-overflow",
            description: "The element's text overflow mode.",
            examples_unity: None,
            examples_mozilla: None,
            format: "clip | ellipsis",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["clip", "ellipsis"]),
        },
        PropertyInfo {
            name: "text-shadow",
            description: "Drop shadow of the text.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<x-offset> <y-offset> <blur-radius> <color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::sequence(vec![
                ValueType::LengthOrPercentage,
                ValueType::LengthOrPercentage,
                ValueType::LengthOrPercentage,
                ValueType::Color,
            ]),
        },
        PropertyInfo {
            name: "top",
            description: "Top distance from the element's box during layout.",
            examples_unity: Some("top: 20px;"),
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#positioning"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "transform-origin",
            description: "The transformation origin is the point around which a transformation is applied.",
            examples_unity: Some("transform-origin: 0% 100%;\ntransform-origin: 20px 10px;\ntransform-origin: 0px 100%;\ntransform-origin: 60% 10px;"),
            examples_mozilla: None,
            format: "[ <length> | <percentage> | left | center | right | top | bottom ] | [ [ <length> | <percentage>  | left | center | right ] && [ <length> | <percentage>  | top | center | bottom ] ] <length>?",
            documentation_url: TRANSFORM_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec ::new(create_transform_origin_formats()),
        },
        PropertyInfo {
            name: "transition",
            description: "Shorthand for transition-delay, transition-duration, transition-property, transition-timing-function",
            examples_unity: Some("transition: width 2s ease-out;\ntransition: margin-right 4s, color 1s;"),
            examples_mozilla: None,
            format: "[<property> <duration> <timing-function>? <delay>?]#", // Can't find a definition, so this is my best guess according to Unity docs
            documentation_url: TRANSITIONS_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::new(FlexibleFormatBuilder::new().required(ValueEntry::new(vec![ValueType::PropertyName])).range(ValueEntry::new(vec![ValueType::Time]), 1, 2).optional(ValueEntry::keywords(&TIMING_FUN)).build()),
        },
        PropertyInfo {
            name: "transition-delay",
            description: "Duration to wait before starting a property's transition effect when its value changes.",
            examples_unity: Some("transition-delay: 0s;\ntransition-delay: 300ms;\ntransition-delay: 2s, 650ms, 2.75s;\ntransition-delay: initial;\ntransition-delay: 0s, 600ms;\ntransition-delay: 0s, 1s;\ntransition-delay: 1s, 2s, 3s, 4s, 5s, 6s, 7s;"),
            examples_mozilla: None,
            format: "<time>#", // This format doesn't exist in official Unity docs but it is shown in Unity's USS warnings
            documentation_url: TRANSITIONS_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::single(ValueType::Time),
        },
        PropertyInfo {
            name: "transition-duration",
            description: "Time a transition animation should take to complete.",
            examples_unity: Some("transition-duration: 2s;\ntransition-duration: 800ms;\ntransition-duration: 3s, 1500ms, 1.75s;\ntransition-duration: initial;\ntransition-duration: 1s, 2s, 3s;\ntransition-duration: 4s, 3s;\ntransition-duration: 500ms, 1s;\ntransition-duration: 1s, 2s;"),
            examples_mozilla: None,
            format: "<time>#", // This format doesn't exist in official Unity docs but it is shown in Unity's USS warnings
            documentation_url: TRANSITIONS_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::single(ValueType::Time),
        },
        PropertyInfo {
            name: "transition-property",
            description: "Properties to which a transition effect should be applied.",
            examples_unity: Some("transition-property: color, rotate;\ntransition-property: left;\ntransition-property: scale;\ntransition-property: translate, all, rotate;\ntransition-property: initial;\ntransition-property: none;\ntransition-property: scale, all, rotate;\ntransition-property: scale, transform-origin;\ntransition-property: all, translate;\ntransition-property: scale, rotate, translate;"),
            examples_mozilla: None,
            format: "[all | none | initial | ignored | <property>]#", // Can't find a definition, so this is my best guess according to Unity docs
            documentation_url: TRANSITIONS_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::one_of(vec![
                ValueType::PropertyName,
                ValueType::Keyword("none"),
                ValueType::Keyword("all"),
                ValueType::Keyword("initial"),
                ValueType::Keyword("ignored"),
            ]),
        },
        PropertyInfo {
            name: "transition-timing-function",
            description: "Determines how intermediate values are calculated for properties modified by a transition effect.",
            examples_unity: Some("transition-timing-function: ease-in-out-sine;\ntransition-timing-function: linear;\ntransition-timing-function: ease-in, ease-out-circ, ease-in-out-cubic;\ntransition-timing-function: initial;\ntransition-timing-function: ease-in-sine, ease-out-elastic;\ntransition-timing-function: linear, ease-in;"),
            examples_mozilla: None,
            format: 
                "<easing-function>#", // This format doesn't exist in official Unity docs but it is shown in Unity's USS warnings
            documentation_url: TRANSITIONS_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&TIMING_FUN),
        },
        PropertyInfo {
            name: "translate",
            description: "A translate transformation.",
            examples_unity: Some("translate: 80%;\ntranslate: 35px;\ntranslate: 5% 10px;\ntranslate: 24px 0%;"),
            examples_mozilla: None,
            format: "none | [<length> | <percentage>] [ [<length> | <percentage>] <length>? ]?",
            documentation_url: TRANSFORM_URL.to_string(),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::new(create_translate_formats()),
        },
        PropertyInfo {
            name: "-unity-background-image-tint-color",
            description: "Tinting color for the element's backgroundImage.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-background"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "-unity-background-scale-mode",
            description: "Background image scaling in the element's box.",
            examples_unity: None,
            examples_mozilla: None,
            format: "stretch-to-fill | scale-and-crop | scale-to-fit",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-background"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["stretch-to-fill", "scale-and-crop", "scale-to-fit"]),
        },
        PropertyInfo {
            name: "-unity-editor-text-rendering-mode",
            description: "TextElement editor rendering mode.",
            examples_unity: None,
            examples_mozilla: None,
            format: "legacy | distance-field",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&["legacy", "distance-field"]),
        },
        PropertyInfo {
            name: "-unity-font",
            description: "Font to draw the element's text, defined as a Font object.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<resource> | <url>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-font"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::one_of(vec![ValueType::Asset]),
        },
        PropertyInfo {
            name: "-unity-font-definition",
            description: "Font to draw the element's text, defined as a FontDefinition structure. It takes precedence over -unity-font.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<resource> | <url>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-font"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::one_of(vec![ValueType::Asset]),
        },
        PropertyInfo {
            name: "-unity-font-style",
            description: "Font style and weight (normal, bold, italic) to draw the element's text.",
            examples_unity: None,
            examples_mozilla: None,
            format: "normal | italic | bold | bold-and-italic",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-font"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["normal", "bold", "italic", "bold-and-italic"]),
        },
        PropertyInfo {
            name: "-unity-overflow-clip-box",
            description: "Specifies which box the element content is clipped against.",
            examples_unity: None,
            examples_mozilla: None,
            format: "padding-box | content-box",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#appearance"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["padding-box", "content-box"]),
        },
        PropertyInfo {
            name: "-unity-paragraph-spacing",
            description: "Increases or decreases the space between paragraphs.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#appearance"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "-unity-slice-bottom",
            description: "Size of the 9-slice's bottom edge when painting an element's background image.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<integer>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Integer),
        },
        PropertyInfo {
            name: "-unity-slice-left",
            description: "Size of the 9-slice's left edge when painting an element's background image.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<integer>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Integer),
        },
        PropertyInfo {
            name: "-unity-slice-right",
            description: "Size of the 9-slice's right edge when painting an element's background image.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<integer>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Integer),
        },
        PropertyInfo {
            name: "-unity-slice-scale",
            description: "Scale applied to an element's slices.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "-unity-slice-top",
            description: "Size of the 9-slice's top edge when painting an element's background image.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<integer>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::Integer),
        },
        PropertyInfo {
            name: "-unity-slice-type",
            description: "Specifies the type of sclicing.",
            examples_unity: None,
            examples_mozilla: None,
            format: "sliced | tiled",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-slice"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["sliced", "tiled"]),
        },
        PropertyInfo {
            name: "-unity-text-align",
            description: "Horizontal and vertical text alignment in the element's box.",
            examples_unity: None,
            examples_mozilla: None,
            format: 
                "upper-left | middle-left | lower-left | upper-center | middle-center | lower-center | upper-right | middle-right | lower-right",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&[
                "upper-left",
                "middle-left",
                "lower-left",
                "upper-center",
                "middle-center",
                "lower-center",
                "upper-right",
                "middle-right",
                "lower-right",
            ]),
        },
        PropertyInfo {
            name: "-unity-text-generator",
            description: "Switches between Unity's standard and advanced text generator",
            examples_unity: None,
            examples_mozilla: None,
            format: "standard | advanced",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::None,
            value_spec: ValueSpec::keywords(&["standard", "advanced"]),
        },
        PropertyInfo {
            name: "-unity-text-outline",
            description: "Outline width and color of the text. Actually, the format is `<length> || <color>`.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | <color>", // this is acutally wrong, the actual format is <length> || <color>, but we will keep it the same as offcial docs
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::new(FlexibleFormatBuilder::any_order().optional(ValueEntry::new(vec![ValueType::LengthOrPercentage])).optional(ValueEntry::new(vec![ValueType::Color])).build()),
        },
        PropertyInfo {
            name: "-unity-text-outline-color",
            description: "Outline color of the text.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<color>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::color(),
        },
        PropertyInfo {
            name: "-unity-text-outline-width",
            description: "Outline width of the text.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
        },
        PropertyInfo {
            name: "-unity-text-overflow-position",
            description: "The element's text overflow position.",
            examples_unity: None,
            examples_mozilla: None,
            format: "start | middle | end",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: false,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["start", "middle", "end"]),
        },
        PropertyInfo {
            name: "visibility",
            description: "Specifies whether or not an element is visible.",
            examples_unity: None,
            examples_mozilla: None,
            format: "visible | hidden",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#appearance"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["visible", "hidden"]),
        },
        PropertyInfo {
            name: "white-space",
            description: "Word wrap over multiple lines if not enough space is available to draw the text of an element.",
            examples_unity: None,
            examples_mozilla: None,
            format: "normal | nowrap",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Discrete,
            value_spec: ValueSpec::keywords(&["normal", "nowrap"]),
        },
        PropertyInfo {
            name: "width",
            description: "Fixed width of an element for the layout.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length> | auto",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#box-model"),
            inherited: false,
            animatable: PropertyAnimation::Animatable,
            value_spec: spec_length_auto.clone(),
        },
        PropertyInfo {
            name: "word-spacing",
            description: "Increases or decreases the space between words.",
            examples_unity: None,
            examples_mozilla: None,
            format: "<length>",
            documentation_url: format!("{SUPPORTED_PROPERTIES_URL}#unity-text"),
            inherited: true,
            animatable: PropertyAnimation::Animatable,
            value_spec: ValueSpec::single(ValueType::LengthOrPercentage),
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
                    if !entry.options.contains(&ValueType::Keyword("initial")) {
                        entry.options.push(ValueType::Keyword("initial"));
                    }
                }
            }
        } else {
            // For other properties, add a separate format that accepts only 'initial'
            let initial_format = ValueFormat {
                entries: vec![ValueEntry {
                    options: vec![ValueType::Keyword("initial")],
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

fn create_translate_formats() -> Vec<ValueFormat> {
    // format
    // none | [<length> | <percentage>] [ [<length> | <percentage>] <length>? ]?
    let mut r = vec![ValueFormat::keywords(&vec!["none"])];
    let format2= FlexibleFormatBuilder::new().range(ValueEntry::new(vec![ValueType::LengthOrPercentage]), 1, 2).build();
    let format3 = FlexibleFormatBuilder::new().range(ValueEntry::new(vec![ValueType::LengthOrPercentage]), 2, 2).optional(ValueEntry::new(vec![ValueType::Length])).build();
    r.extend(format2);
    r.extend(format3);
    r
}

fn create_transform_origin_formats() -> Vec<ValueFormat> {
    // format
    // [ <length> | <percentage> | left | center | right | top | bottom ] | [ [ <length> | <percentage>  | left | center | right ] && [ <length> | <percentage>  | top | center | bottom ] ] <length>?
    let mut r = vec![ValueFormat{entries: vec![ValueEntry{
        options: vec![ValueType::LengthOrPercentage, ValueType::Keyword("left"), ValueType::Keyword("center"), ValueType::Keyword("right"), ValueType::Keyword("top"), ValueType::Keyword("bottom")],
    }]}];
    let x_entry = ValueEntry::new(
        vec![ValueType::LengthOrPercentage, ValueType::Keyword("left"), ValueType::Keyword("center"), ValueType::Keyword("right")]);
    let y_entry = ValueEntry::new(
        vec![ValueType::LengthOrPercentage, ValueType::Keyword("top"), ValueType::Keyword("center"), ValueType::Keyword("bottom")]);
    let z_entry = ValueEntry::new(vec![ValueType::Length]); // there is an optional length here I assume it is for z
    let format2 = FlexibleFormatBuilder::new().required(x_entry.clone()).required(y_entry.clone()).optional(z_entry.clone()).build();
    let format3 = FlexibleFormatBuilder::new().required(y_entry.clone()).required(x_entry.clone()).optional(z_entry.clone()).build();

    r.extend(format2);
    r.extend(format3);
    r
}

fn create_scale_formats() -> Vec<ValueFormat> {
    // format
    // <number> | <number> <number> | none
    let mut r = vec![ValueFormat::keywords(&vec!["none"])];

    let format1 = FlexibleFormatBuilder::new()
        .range(ValueEntry::new(vec![ValueType::Number]), 1, 2)
        .build();

    r.extend(format1.into_iter());
    r
}

fn create_flex_formats() -> Vec<ValueFormat> {
    // format
    // none | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]
    // flex-basis: <length> | auto
    // flex-grow: <number>
    // flex-shrink: <number>

    let mut r = vec![ValueFormat::keywords(&["none"])];

    let format2 = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::Number])) // flex-grow
        .optional(ValueEntry::new(vec![ValueType::Number])) // flex-shrink
        .required(ValueEntry::new(vec![
            ValueType::LengthOrPercentage,
            ValueType::Keyword("auto"),
        ])) // flex-basis
        .build();

    let format3 = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![
            ValueType::LengthOrPercentage,
            ValueType::Keyword("auto"),
        ])) // flex-basis
        .required(ValueEntry::new(vec![ValueType::Number])) // flex-grow
        .optional(ValueEntry::new(vec![ValueType::Number])) // flex-shrink
        .build();

    let format4 = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![
            ValueType::LengthOrPercentage,
            ValueType::Keyword("auto"),
        ])) // flex-basis
        .build();

    let format5 = FlexibleFormatBuilder::new()
        .required(ValueEntry::new(vec![ValueType::Number])) // flex-grow
        .optional(ValueEntry::new(vec![ValueType::Number])) // flex-shrink
        .build();

    r.extend(format2.into_iter());
    r.extend(format3.into_iter());
    r.extend(format4.into_iter());
    r.extend(format5.into_iter());
    r
}

fn create_formats_for_background_position() -> Vec<ValueFormat> {
    // format
    // [ left | center | right | top | bottom | <length> ]  |  [ left | center | right | <length> ] [ top | center | bottom | <length> ]  |  [ center | [ left | right ] <length>? ] && [ center | [ top | bottom ] <length>? ]
    let mut result = vec![
        ValueFormat::one_of(vec![
            ValueType::Keyword("left"),
            ValueType::Keyword("center"),
            ValueType::Keyword("right"),
            ValueType::Keyword("top"),
            ValueType::Keyword("bottom"),
            ValueType::LengthOrPercentage,
        ]), // single value
        ValueFormat {
            entries: vec![
                ValueEntry {
                    options: vec![
                        ValueType::Keyword("left"),
                        ValueType::Keyword("center"),
                        ValueType::Keyword("right"),
                        ValueType::LengthOrPercentage,
                    ],
                },
                ValueEntry {
                    options: vec![
                        ValueType::Keyword("top"),
                        ValueType::Keyword("center"),
                        ValueType::Keyword("bottom"),
                        ValueType::LengthOrPercentage,
                    ],
                },
            ],
        },
        ValueFormat::sequence(vec![
            ValueType::Keyword("center"),
            ValueType::Keyword("center"),
        ]),
    ]; // center center
    let format2 = FlexibleFormatBuilder::new()
        .required(ValueEntry::keywords(&vec!["center"]))
        .required(ValueEntry::keywords(&vec!["top", "bottom"]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .build();
    let format3 = FlexibleFormatBuilder::new()
        .required(ValueEntry::keywords(&vec!["center"]))
        .required(ValueEntry::keywords(&vec!["left", "right"]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .build();
    let format4 = FlexibleFormatBuilder::new()
        .required(ValueEntry::keywords(&vec!["top", "bottom"]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .required(ValueEntry::keywords(&vec!["center"]))
        .build();
    let format5 = FlexibleFormatBuilder::new()
        .required(ValueEntry::keywords(&vec!["left", "right"]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .required(ValueEntry::keywords(&vec!["center"]))
        .build();
    let format6 = FlexibleFormatBuilder::new()
        .required(ValueEntry::keywords(&vec!["left", "right"]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .required(ValueEntry::keywords(&vec!["top", "bottom"]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .build();
    let format7 = FlexibleFormatBuilder::new()
        .required(ValueEntry::keywords(&vec!["top", "bottom"]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .required(ValueEntry::keywords(&vec!["left", "right"]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .build();
    result.extend(format2.into_iter());
    result.extend(format3.into_iter());
    result.extend(format4.into_iter());
    result.extend(format5.into_iter());
    result.extend(format6.into_iter());
    result.extend(format7.into_iter());
    result
}

fn create_formats_for_background_position_x() -> Vec<ValueFormat> {
    // format
    // [ center | [ [ left | right | x-start | x-end ]? <length>? ]! ]#
    let mut result = vec![
        ValueFormat::keywords(&vec!["center"]), // center
    ];
    let format2 = FlexibleFormatBuilder::new()
        .optional(ValueEntry::keywords(&vec![
            "left", "right", "x-start", "x-end",
        ]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .build();
    result.extend(format2.into_iter());
    result
}

fn create_formats_for_background_position_y() -> Vec<ValueFormat> {
    // format
    // [ center | [ [ top | bottom | y-start | y-end ]? <length>? ]! ]
    let mut result = vec![
        ValueFormat::keywords(&vec!["center"]), // center
    ];
    let format2 = FlexibleFormatBuilder::new()
        .optional(ValueEntry::keywords(&vec![
            "top", "bottom", "y-start", "y-end",
        ]))
        .optional(ValueEntry::new(vec![ValueType::LengthOrPercentage]))
        .build();
    result.extend(format2.into_iter());
    result
}

fn create_formats_for_background_repeat() -> Vec<ValueFormat> {
    // format
    // repeat-x | repeat-y | [ repeat | space | round | no-repeat ]{1,2}
    let mut r = vec![ValueFormat::keywords(&vec!["repeat-x", "repeat-y"])];
    let format2 = FlexibleFormatBuilder::new()
        .range(
            ValueEntry::keywords(&vec!["repeat", "space", "round", "no-repeat"]),
            1,
            2,
        )
        .build();
    r.extend(format2.into_iter());
    r
}

fn create_formats_for_background_size() -> Vec<ValueFormat> {
    // format
    // [ <length> | auto ]{1,2} | cover | contain
    let mut r = vec![
        ValueFormat::keywords(&vec!["cover", "contain"]), // single special keywords
    ];
    let format2 = FlexibleFormatBuilder::new()
        .range(
            ValueEntry::new(vec![ValueType::Keyword("auto"), ValueType::LengthOrPercentage]),
            1,
            2,
        )
        .build();
    r.extend(format2.into_iter());
    r
}
