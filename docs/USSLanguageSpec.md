# Unity Style Sheet (USS) Language Specification

## Introduction
Unity Style Sheet (USS) is a styling language for Unity's UI Toolkit, inspired by CSS, used to define the appearance of UI elements. This specification outlines the syntax and grammar for a language server to parse, validate, and provide features like autocompletion for USS files (`.uss` extension).

## Syntax Overview
USS follows the same basic structure as CSS:
```
selector {
    property: value;
}
```
- **Selectors**: Target UI elements.
- **Declaration blocks**: Contain property-value pairs, terminated by semicolons (`;`).

## Selectors
USS supports a subset of CSS selectors, adapted for Unity's UI elements.

### Simple Selectors
- **Type selectors**: Match UI element types, can be user defined types, without namespace (e.g., `Button`, `Label`).
- **Class selectors**: Match elements with USS classes (e.g., `.class-name`).
- **Name selectors**: Match elements by name (e.g., `#element-name`), similar to CSS ID selectors.
- **Universal selector**: Matches any element (e.g., `*`).

### Complex Selectors
- **Descendant selectors**: Match descendants (e.g., `selector1 selector2`), same as CSS.
- **Child selectors**: Match direct children (e.g., `selector1 > selector2`), same as CSS.
- **Multiple selectors**: Match elements satisfying all selectors (e.g., `.class1.class2`), same as CSS.

### Selector Lists
- Comma-separated selectors apply styles to each (e.g., `selector1, selector2`), same as CSS.

### Pseudo-classes
USS supports specific pseudo-classes for UI states:
- `:hover` - The cursor is positioned over the element
- `:active` - A user interacts with the element
- `:inactive` - A user stops to interact with the element
- `:focus` - The element has focus
- `:selected` - USS doesn't support this pseudo-state. Use `:checked` instead
- `:disabled` - The element is in a disabled state
- `:enabled` - The element is in an enabled state
- `:checked` - The element is a Toggle or RadioButton element and it's selected
- `:root` - The element is the highest-level element in the visual tree that has the stylesheet applied

**Chaining pseudo-classes**: You can chain pseudo-classes together to apply styles for multiple concurrent states (e.g., `Toggle:checked:hover`).

**Note**: Not all CSS pseudo-classes are supported; only those listed above apply. You cannot extend pseudo-classes or create custom ones.

## At-Rules
USS supports a limited subset of CSS at-rules. Currently, Unity only supports the `@import` at-rule for including other USS files.

### @import
The `@import` at-rule allows you to import styles from other USS files. This is useful for organizing styles across multiple files and creating reusable style libraries.

**Syntax**:
```uss
@import "path/to/stylesheet.uss";
@import url("path/to/stylesheet.uss");
```

**Example**:
```uss
/* Import a common styles file */
@import "common-styles.uss";

/* Import from a subdirectory */
@import "components/button-styles.uss";

/* Using url() function */
@import url("themes/dark-theme.uss");

/* Your specific styles */
Button {
    background-color: blue;
}
```

**Important Notes**:
- `@import` statements must appear at the beginning of the USS file, before any other rules
- Paths are relative to the current USS file location
- Circular imports are not supported and will cause errors
- Only USS files can be imported (`.uss` extension)

### Characters and case (from CSS spec but is the same for USS)
The following rules always hold:

All CSS syntax is case-insensitive within the ASCII range (i.e., [a-z] and [A-Z] are equivalent), except for parts that are not under the control of CSS. For example, the case-sensitivity of values of the HTML attributes "id" and "class", of font names, and of URIs lies outside the scope of this specification. Note in particular that element names are case-insensitive in HTML, but case-sensitive in XML.

In CSS, identifiers (including element names, classes, and IDs in selectors) can contain only the characters [a-zA-Z0-9] and ISO 10646 characters U+00A0 and higher, plus the hyphen (-) and the underscore (_); they cannot start with a digit, two hyphens, or a hyphen followed by a digit. Identifiers can also contain escaped characters and any ISO 10646 character as a numeric code (see next item). For instance, the identifier "B&W?" may be written as "B\&W\?" or "B\26 W\3F".
Note that Unicode is code-by-code equivalent to ISO 10646 (see [UNICODE] and [ISO10646]).

In CSS 2.1, a backslash (\) character can indicate one of three types of character escape. Inside a CSS comment, a backslash stands for itself, and if a backslash is immediately followed by the end of the style sheet, it also stands for itself (i.e., a DELIM token).

First, inside a string, a backslash followed by a newline is ignored (i.e., the string is deemed not to contain either the backslash or the newline). Outside a string, a backslash followed by a newline stands for itself (i.e., a DELIM followed by a newline).

Second, it cancels the meaning of special CSS characters. Any character (except a hexadecimal digit, linefeed, carriage return, or form feed) can be escaped with a backslash to remove its special meaning. For example, "\"" is a string consisting of one double quote. Style sheet preprocessors must not remove these backslashes from a style sheet since that would change the style sheet's meaning.

Third, backslash escapes allow authors to refer to characters they cannot easily put in a document. In this case, the backslash is followed by at most six hexadecimal digits (0..9A..F), which stand for the ISO 10646 ([ISO10646]) character with that number, which must not be zero. (It is undefined in CSS 2.1 what happens if a style sheet does contain a character with Unicode codepoint zero.) If a character in the range [0-9a-fA-F] follows the hexadecimal number, the end of the number needs to be made clear. There are two ways to do that:

with a space (or other white space character): "\26 B" ("&B"). In this case, user agents should treat a "CR/LF" pair (U+000D/U+000A) as a single white space character.
by providing exactly 6 hexadecimal digits: "\000026B" ("&B")
In fact, these two methods may be combined. Only one white space character is ignored after a hexadecimal escape. Note that this means that a "real" space after the escape sequence must be doubled.

If the number is outside the range allowed by Unicode (e.g., "\110000" is above the maximum 10FFFF allowed in current Unicode), the UA may replace the escape with the "replacement character" (U+FFFD). If the character is to be displayed, the UA should show a visible symbol, such as a "missing character" glyph (cf. 15.2, point 5).

Note: Backslash escapes are always considered to be part of an identifier or a string (i.e., "\7B" is not punctuation, even though "{" is, and "\32" is allowed at the start of a class name, even though "2" is not).
The identifier "te\st" is exactly the same identifier as "test".

## Properties
This section introduces the common USS properties, their syntax and accepted values, and differences from CSS. <mcreference link="https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-SupportedProperties.html" index="1">1</mcreference> For a complete list of USS properties, see the Properties Reference Table below.

### All
The `all` property resets all properties to their default value. This property doesn't apply to the custom USS properties. <mcreference link="https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-SupportedProperties.html" index="1">1</mcreference>

```uss
all: initial
```

### Box Model

#### Dimensions
```uss
width: <length> | auto
height: <length> | auto
min-width: <length> | auto
min-height: <length> | auto
max-width: <length> | none
max-height: <length> | none
```

The `width` and `height` specify the size of the element. If width isn't specified, the width is based on the width of the element's contents. If height isn't specified, the height is based on the height of the element's contents. <mcreference link="https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-SupportedProperties.html" index="1">1</mcreference>

#### Margins
```uss
margin-left: <length> | auto
margin-top: <length> | auto
margin-right: <length> | auto
margin-bottom: <length> | auto
/* Shorthand */
margin: [<length> | auto]{1,4}
```

#### Borders
```uss
border-left-width: <length>
border-top-width: <length>
border-right-width: <length>
border-bottom-width: <length>
/* Shorthand */
border-width: <length>{1,4}
```

#### Padding
```uss
padding-left: <length>
padding-top: <length>
padding-right: <length>
padding-bottom: <length>
/* Shorthand */
padding: <length>{1,4}
```

#### Differences from CSS
The alternative box model that USS uses is different from the standard CSS box model. In the standard CSS box model, width and height define the size of the content box. An element's rendered size is the sum of its padding, border-width, and width / height values.

Unity's model is equivalent to setting the CSS `box-sizing` property to `border-box`. <mcreference link="https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-SupportedProperties.html" index="1">1</mcreference>

### Flex Layout
UI Toolkit includes a layout engine that positions visual elements based on layout and styling properties. The layout engine implements a subset of Flexbox, an HTML/CSS layout system. <mcreference link="https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-SupportedProperties.html" index="1">1</mcreference>

By default, all items are vertically placed in their container.

```uss
/* Items */
flex-grow: <number>
flex-shrink: <number>
flex-basis: <length> | auto
flex: none | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]
align-self: auto | flex-start | flex-end | center | stretch

/* Containers */
flex-direction: row | row-reverse | column | column-reverse
flex-wrap: nowrap | wrap | wrap-reverse
align-content: flex-start | flex-end | center | stretch

/* The default value is `stretch`.
`auto` sets `align-items` to `flex-end`. */
align-items: auto | flex-start | flex-end | center | stretch

justify-content: flex-start | flex-end | center | space-between | space-around
```

### Positioning
```uss
/* The default value is `relative` which positions the element based on its parent.
If sets to `absolute`, the element leaves its parent layout and values are specified based on the parent bounds.*/
position: absolute | relative

/* The distance from the parent edge or the original position of the element. */
left: <length> | auto
top: <length> | auto
right: <length> | auto
bottom: <length> | auto
```

### Background
```uss
background-color: <color>
background-image: <resource> | <url> | none
-unity-background-scale-mode: stretch-to-fill | scale-and-crop | scale-to-fit
-unity-background-image-tint-color: <color>
```

### Border Color
```uss
border-left-color: <color>
border-top-color: <color>
border-right-color: <color>
border-bottom-color: <color>
/* Shorthand */
border-color: <color>{1,4}
```

### Border Radius
```uss
border-top-left-radius: <length>
border-top-right-radius: <length>
border-bottom-left-radius: <length>
border-bottom-right-radius: <length>
/* Shorthand */
border-radius: <length>{1,4}
```

Border radius properties work almost the same in USS and CSS. <mcreference link="https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-SupportedProperties.html" index="1">1</mcreference>

### Transform Properties
The transform properties apply a 2D transformation to a visual element. You can use them to rotate, scale, or move a visual element. <mcreference link="https://docs.unity3d.com/Manual//UIE-Transform.html" index="1">1</mcreference>

```uss
transform-origin: <length> <length> | <percentage> <percentage> | center | top | bottom | left | right
translate: <length> <length> | <percentage> <percentage>
scale: <number> <number> | none
rotate: <angle>
```

All transformations are performed in the following order: Scale, Rotate, Translate. <mcreference link="https://docs.unity3d.com/Manual//UIE-Transform.html" index="1">1</mcreference>

**Note**: Transform is best used for changes and animations rather than static positioning. For static positioning, use regular layout properties like `top` and `left`. <mcreference link="https://docs.unity3d.com/Manual//UIE-Transform.html" index="1">1</mcreference>

### Text Properties
Text properties set the color, font, font size, and Unity-specific properties for font resource, font style, alignment, word wrap, and clipping. <mcreference link="https://docs.unity3d.com/6000.1/Documentation/Manual/UIE-USS-SupportedProperties.html" index="2">2</mcreference> Unlike most USS style properties, text style properties propagate to child elements. <mcreference link="https://docs.unity3d.com/6000.1/Documentation/Manual/UIB-styling-ui-text.html" index="1">1</mcreference>

```uss
color: <color>
-unity-font: <resource> | <url>
-unity-font-definition: <resource> | <url>
font-size: <number>
-unity-font-style: normal | italic | bold | bold-and-italic
-unity-text-align: upper-left | middle-left | lower-left | upper-center | middle-center | lower-center | upper-right | middle-right | lower-right
-unity-text-overflow-position: start | middle | end
white-space: normal | nowrap
-unity-text-outline-width: <length>
-unity-text-outline-color: <color>
/* Shorthand */
-unity-text-outline: <length> | <color>
-unity-text-generator: standard | advanced
/* The text overflow mode. */
text-overflow: clip | ellipsis
text-shadow: <x-offset> <y-offset> <blur-radius> <color>
letter-spacing: <length>
word-spacing: <length>
-unity-paragraph-spacing: <length>
```

**Note**: When you set up the font in UI Builder, the Font control sets `-unity-font`, and the Font Asset control sets `-unity-font-definition`. Because `-unity-font-definition` takes precedence over `-unity-font`, to use a font from the Font list, select None from Font Asset. <mcreference link="https://docs.unity3d.com/6000.1/Documentation/Manual/UIE-USS-SupportedProperties.html" index="2">2</mcreference>

### Display Properties
The USS `display` property supports only a small subset of the CSS display property's available keyword values. <mcreference link="https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-SupportedProperties.html" index="1">1</mcreference>

```uss
display: flex | none
```

### Cursor Properties
The `cursor` property specifies the mouse cursor to be displayed when the mouse pointer is over an element. <mcreference link="https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-SupportedProperties.html" index="1">1</mcreference>

```uss
cursor: [ [ <resource> | <url> ] [ <integer> <integer>]? , ] [ arrow | text | resize-vertical | resize-horizontal | link | slide-arrow | resize-up-right | resize-up-left | move-arrow | rotate-arrow | scale-arrow | arrow-plus | arrow-minus | pan | orbit | zoom | fps | split-resize-up-down | split-resize-left-right ]
```

## Values
USS supports CSS-like data types with Unity-specific extensions:

### Basic Data Types
- **Length**: `px` (pixels, absolute) and `%` (percentage, relative to parent). Unit required except for `0`.
- **Numeric**: Floating point or integer literals (e.g., `flex: 1.0`).
- **Keywords**: Descriptive names like `auto`, `absolute`. All properties support `initial` to reset to default.
- **Color**: 
  - Hexadecimal: `#FFFF00`, `#0F0`
  - Functions: `rgb(255, 255, 0)`, `rgba(255, 255, 0, 1.0)`
  - Keywords: Standard CSS color names
- **Angle**: `deg`, `grad`, `rad`, `turn` (e.g., `90deg`, `1.5708rad`) - Used for rotation transforms

### Asset References
USS provides two functions for referencing Unity assets:
- **`url("path")`**: References assets by file path (relative to USS file or absolute from project root)
  - Relative: `url("../Resources/image.png")`
  - Absolute: `url("/Assets/Resources/image.png")` or `url("project:/Assets/Resources/image.png")` or `url("project:///Assets/Editor/Resources/thumb.png")`
- **`resource("name")`**: References assets in `Resources`(including `Editor Default Resources`) folders by name
  - eg. `resource("Images/my-image.png").png` (extension can be ommited here, which is different than `url`)

**Note**: The `resource()` function supports automatic loading of different image versions for different screen densities.

## Custom Properties
USS supports custom properties (variables) like CSS:
- Define: `--property: value;` (e.g., `--main-color: red;`)
- Use: `var(--property)` (e.g., `color: var(--main-color);`)
- Often declared in `:root` for global scope.

**Example**:
```uss
:root {
    --main-color: red;
}
Button {
    color: var(--main-color);
}
```

## Selector Precedence
USS resolves style conflicts with:
1. **C# styles**: Highest precedence, override all.
2. **UXML inline styles**: Override USS styles.
3. **USS styles**: Use CSS-like specificity:
   - Higher specificity wins (e.g., `#id` > `.class` > `type`).
   - Equal specificity: Last rule in file applies.
- **Note**: USS does **not** support `!important`.

**Example**:
```uss
Button {
    color: blue;  /* Lower specificity */
}
#myButton {
    color: red;   /* Higher specificity, applies */
}
```

## Additional Notes
- **Case sensitivity**: Selectors are case-sensitive(which is in offical docs). Property names are also case sensitive, no official docs found but tests shows they are case sensitive.
- **Escaping**: Special characters in selectors must be escaped (e.g., `#name\.with\.dots`).
- **Unity-specific behavior**:
  - `:root` matches stylesheet attachment points, not a fixed root element.
  - **Inheritance**: USS supports CSS-like inheritance (e.g., `color` inherits), but some properties may not due to Unity’s UI Toolkit structure.
  - **Style Application**: Styles apply in file order; later rules override earlier ones if specificity is equal.
- **Limitations**: No support for all CSS features (e.g., no `!important`, limited pseudo-classes, no nested rules, no `calc()`, `var()` can't be inside other functions as argument).

## Additional References
- [USS properties reference
](https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-USS-Properties-Reference.html)
- [USS color keywords
](https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-uss-color-keywords.html)
