# Copies relavent property docs from mozilla


background-position
===================

Baseline Widely available

This feature is well established and works across many devices and browser versions. It’s been available across browsers since July 2015.

*   [Learn more](/en-US/docs/Glossary/Baseline/Compatibility)
*   [See full compatibility](#browser_compatibility)
*   [Report feedback](https://survey.alchemer.com/s3/7634825/MDN-baseline-feedback?page=%2Fen-US%2Fdocs%2FWeb%2FCSS%2Fbackground-position&level=high)

The **`background-position`** [CSS](/en-US/docs/Web/CSS) property sets the initial position for each background image. The position is relative to the position layer set by [`background-origin`](/en-US/docs/Web/CSS/background-origin).

[Try it](#try_it)
-----------------

    background-position: top;
    

    background-position: left;
    

    background-position: center;
    

    background-position: 25% 75%;
    

    background-position: bottom 50px right 100px;
    

    background-position: right 35% bottom 45%;
    

    <section class="display-block" id="default-example">
      <div class="transition-all" id="example-element"></div>
    </section>
    

    #example-element {
      background-color: navajowhite;
      background-image: url("/shared-assets/images/examples/star.png");
      background-repeat: no-repeat;
      height: 100%;
    }
    

[Syntax](#syntax)
-----------------

css

    /* Keyword values */
    background-position: top;
    background-position: bottom;
    background-position: left;
    background-position: right;
    background-position: center;
    
    /* <percentage> values */
    background-position: 25% 75%;
    
    /* <length> values */
    background-position: 0 0;
    background-position: 1cm 2cm;
    background-position: 10ch 8em;
    
    /* Multiple images */
    background-position:
      0 0,
      center;
    
    /* Edge offsets values */
    background-position: bottom 10px right 20px;
    background-position: right 3em bottom 10px;
    background-position: bottom 10px right;
    background-position: top right 10px;
    
    /* Global values */
    background-position: inherit;
    background-position: initial;
    background-position: revert;
    background-position: revert-layer;
    background-position: unset;
    

The `background-position` property is specified as one or more `<position>` values, separated by commas.

### [Values](#values)

[`<position>`](#position)

A [`<position>`](/en-US/docs/Web/CSS/position_value). A position defines an x/y coordinate, to place an item relative to the edges of an element's box. It can be defined using one to four values. If two non-keyword values are used, the first value represents the horizontal position and the second represents the vertical position. If only one value is specified, the second value is assumed to be `center`. If three or four values are used, the length-percentage values are offsets for the preceding keyword value(s).

**1-value syntax:** The value may be:

*   The keyword value `center`, which centers the image.
*   One of the keyword values `top`, `left`, `bottom`, or `right`. This specifies an edge against which to place the item. The other dimension is then set to 50%, so the item is placed in the middle of the edge specified.
*   A [`<length>`](/en-US/docs/Web/CSS/length) or [`<percentage>`](/en-US/docs/Web/CSS/percentage). This specifies the X coordinate relative to the left edge, with the Y coordinate set to 50%.

**2-value syntax:** One value defines X and the other defines Y. Each value may be:

*   One of the keyword values `top`, `left`, `bottom`, or `right`. If `left` or `right` is given, then this defines X and the other given value defines Y. If `top` or `bottom` is given, then this defines Y and the other value defines X.
*   A [`<length>`](/en-US/docs/Web/CSS/length) or [`<percentage>`](/en-US/docs/Web/CSS/percentage). If the other value is `left` or `right`, then this value defines Y, relative to the top edge. If the other value is `top` or `bottom`, then this value defines X, relative to the left edge. If both values are `<length>` or `<percentage>` values, then the first defines X and the second Y.
*   Note that: If one value is `top` or `bottom`, then the other value may not be `top` or `bottom`. If one value is `left` or `right`, then the other value may not be `left` or `right`. This means, e.g., that `top top` and `left right` are not valid.
*   Order: When pairing keywords, placement is not important as the browser can reorder it; the values `top left` and `left top` will yield the same result. When pairing [`<length>`](/en-US/docs/Web/CSS/length) or [`<percentage>`](/en-US/docs/Web/CSS/percentage) with a keyword, the placement is important: the value defining X should come first followed by Y, so for example the value `right 20px` is valid while `20px right` is invalid. The values `left 20%` and `20% bottom` are valid as X and Y values are clearly defined and the placement is correct.
*   The default value is `left top` or `0% 0%`.

**3-value syntax:** Two values are keyword values, and the third is the offset for the preceding value:

*   The first value is one of the keyword values `top`, `left`, `bottom`, `right`, or `center`. If `left` or `right` are given here, then this defines X. If `top` or `bottom` are given, then this defines Y and the other keyword value defines X.
*   The [`<length>`](/en-US/docs/Web/CSS/length) or [`<percentage>`](/en-US/docs/Web/CSS/percentage) value, if it is the second value, is the offset for the first value. If it is the third value, it is the offset for the second value.
*   The single length or percentage value is an offset for the keyword value that precedes it. The combination of one keyword with two [`<length>`](/en-US/docs/Web/CSS/length) or [`<percentage>`](/en-US/docs/Web/CSS/percentage) values is not valid.

**4-value syntax:** The first and third values are keyword values defining X and Y. The second and fourth values are offsets for the preceding X and Y keyword values:

*   The first and third values are equal to one of the keyword values `top`, `left`, `bottom`, or `right`. If `left` or `right` is given for the first value, then this defines X and the other value defines Y. If `top` or `bottom` is given for the first value, then this defines Y and the other keyword value defines X.
*   The second and fourth values are [`<length>`](/en-US/docs/Web/CSS/length) or [`<percentage>`](/en-US/docs/Web/CSS/percentage) values. The second value is the offset for the first keyword. The fourth value is the offset for the second keyword.

### [Regarding Percentages](#regarding_percentages)

The percentage offset of the given background image's position is relative to the container. A value of 0% means that the left (or top) edge of the background image is aligned with the corresponding left (or top) edge of the container, or the 0% mark of the image will be on the 0% mark of the container. A value of 100% means that the _right_ (or _bottom_) edge of the background image is aligned with the _right_ (or _bottom_) edge of the container, or the 100% mark of the image will be on the 100% mark of the container. Thus a value of 50% horizontally or vertically centers the background image as the 50% of the image will be at the 50% mark of the container. Similarly, `background-position: 25% 75%` means the spot on the image that is 25% from the left and 75% from the top will be placed at the spot of the container that is 25% from the container's left and 75% from the container's top.

Essentially what happens is the background image dimension is _subtracted_ from the corresponding container dimension, and then a percentage of the resulting value is used as the direct offset from the left (or top) edge.

(container width - image width) \* (position x%) = (x offset value)
(container height - image height) \* (position y%) = (y offset value)

Using the X axis for an example, let's say we have an image that is 300px wide and we are using it in a container that is 100px wide, with `background-size` set to auto:

100px - 300px = -200px (container & image difference)

So that with position percentages of -25%, 0%, 50%, 100%, 125%, we get these image-to-container edge offset values:

\-200px \* -25% = 50px
-200px \* 0% = 0px
-200px \* 50% = -100px
-200px \* 100% = -200px
-200px \* 125% = -250px

So with these resultant values for our example, the **left edge** of the **image** is offset from the **left edge** of the **container** by:

*   \+ 50px (putting the left image edge in the center of the 100-pixel-wide container)
*   0px (left image edge coincident with the left container edge)
*   \-100px (left image edge 100px to the left of the container, in this example that means the middle 100px image area is centered in the container)
*   \-200px (left image edge 200px to the left of the container, in this example that means the right image edge is coincident with the right container edge)
*   \-250px (left image edge 250px to the left of the container, in this example that puts the right edge of the 300px-wide image in the center of the container)

It's worth mentioning that if your `background-size` is equal to the container size for a given axis, then a _percentage_ position for that axis will have no effect because the "container-image difference" will be zero. You will need to offset using absolute values.

[Formal definition](#formal_definition)
---------------------------------------

[Initial value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#initial_value)

`0% 0%`

Applies to

all elements. It also applies to [`::first-letter`](/en-US/docs/Web/CSS/::first-letter) and [`::first-line`](/en-US/docs/Web/CSS/::first-line).

[Inherited](/en-US/docs/Web/CSS/CSS_cascade/Inheritance)

no

Percentages

refer to the size of the background positioning area minus size of background image; size refers to the width for horizontal offsets and to the height for vertical offsets

[Computed value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#computed_value)

as each of the properties of the shorthand:  

*   [`background-position-x`](/en-US/docs/Web/CSS/background-position-x): A list, each item consisting of: an offset given as a combination of an absolute length and a percentage, plus an origin keyword
*   [`background-position-y`](/en-US/docs/Web/CSS/background-position-y): A list, each item consisting of: an offset given as a combination of an absolute length and a percentage, plus an origin keyword

[Animation type](/en-US/docs/Web/CSS/CSS_animated_properties)

a repeatable list

[Formal syntax](#formal_syntax)
-------------------------------

background-position =   
  <bg-position>[#](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#hash_mark "Hash mark: the entity is repeated one or several times, each occurrence separated by a comma")    
  
<bg-position> =   
  [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") left [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") center [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") right [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") top [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") bottom [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") <length-percentage> [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")  [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") left [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") center [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") right [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") <length-percentage> [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") top [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") center [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") bottom [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") <length-percentage> [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")  [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") center [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") left [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") right [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") <length-percentage>[?](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#question_mark "Question mark: the entity is optional") [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") [&&](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#double_ampersand "Double ampersand: all of the entities must be present, in any order") [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") center [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") top [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") bottom [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") <length-percentage>[?](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#question_mark "Question mark: the entity is optional") [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")    
  
<length-percentage> =   
  [<length>](/en-US/docs/Web/CSS/length)      [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  [<percentage>](/en-US/docs/Web/CSS/percentage)    

Sources: [CSS Backgrounds and Borders Module Level 3](https://drafts.csswg.org/css-backgrounds-3/), [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/)

[Examples](#examples)
---------------------

### [Positioning background images](#positioning_background_images)

Each of these three examples uses the [`background`](/en-US/docs/Web/CSS/background) property to create a yellow, rectangular element containing a star image. In each example, the star is in a different position. The third example illustrates how to specify positions for two different background images within one element.

#### HTML

html

    <div class="example-one">Example One</div>
    <div class="example-two">Example Two</div>
    <div class="example-three">Example Three</div>
    

#### CSS

css

    /* Shared among all <div>s */
    div {
      background-color: #ffee99;
      background-repeat: no-repeat;
      width: 300px;
      height: 80px;
      margin-bottom: 12px;
    }
    
    /* These examples use the `background` shorthand property */
    .example-one {
      background: url("star-transparent.gif") #ffee99 2.5cm bottom no-repeat;
    }
    .example-two {
      background: url("star-transparent.gif") #ffee99 left 4em bottom 1em no-repeat;
    }
    
    /* Multiple background images: Each image is matched with the
       corresponding position, from first specified to last. */
    .example-three {
      background-image: url("star-transparent.gif"), url("cat-front.png");
      background-position:
        0px 0px,
        right 3em bottom 2em;
    }
    

#### Result

[Specifications](#specifications)
---------------------------------

Specification

[CSS Backgrounds and Borders Module Level 3  
\# background-position](https://drafts.csswg.org/css-backgrounds/#background-position)

[Browser compatibility](#browser_compatibility)
-----------------------------------------------

[See also](#see_also)
---------------------

*   [`background-position-x`](/en-US/docs/Web/CSS/background-position-x)
*   [`background-position-y`](/en-US/docs/Web/CSS/background-position-y)
*   [Using multiple backgrounds](/en-US/docs/Web/CSS/CSS_backgrounds_and_borders/Using_multiple_backgrounds)
*   [`transform-origin`](/en-US/docs/Web/CSS/transform-origin)



background-position-x
=====================

Baseline Widely available \*

This feature is well established and works across many devices and browser versions. It’s been available across browsers since September 2016.

\* Some parts of this feature may have varying levels of support.

*   [Learn more](/en-US/docs/Glossary/Baseline/Compatibility)
*   [See full compatibility](#browser_compatibility)
*   [Report feedback](https://survey.alchemer.com/s3/7634825/MDN-baseline-feedback?page=%2Fen-US%2Fdocs%2FWeb%2FCSS%2Fbackground-position-x&level=high)

The **`background-position-x`** [CSS](/en-US/docs/Web/CSS) property sets the initial horizontal position for each background image. The position is relative to the position layer set by [`background-origin`](/en-US/docs/Web/CSS/background-origin).

[Try it](#try_it)
-----------------

    background-position-x: left;
    

    background-position-x: center;
    

    background-position-x: 25%;
    

    background-position-x: 2rem;
    

    background-position-x: right 32px;
    

    <section class="display-block" id="default-example">
      <div class="transition-all" id="example-element"></div>
    </section>
    

    #example-element {
      background-color: navajowhite;
      background-image: url("/shared-assets/images/examples/star.png");
      background-repeat: no-repeat;
      height: 100%;
    }
    

The value of this property is overridden by any declaration of the [`background`](/en-US/docs/Web/CSS/background) or [`background-position`](/en-US/docs/Web/CSS/background-position) shorthand properties applied to the element after it.

[Syntax](#syntax)
-----------------

css

    /* Keyword values */
    background-position-x: left;
    background-position-x: center;
    background-position-x: right;
    
    /* <percentage> values */
    background-position-x: 25%;
    
    /* <length> values */
    background-position-x: 0px;
    background-position-x: 1cm;
    background-position-x: 8em;
    
    /* Side-relative values */
    background-position-x: right 3px;
    background-position-x: left 25%;
    
    /* Multiple values */
    background-position-x: 0px, center;
    
    /* Global values */
    background-position-x: inherit;
    background-position-x: initial;
    background-position-x: revert;
    background-position-x: revert-layer;
    background-position-x: unset;
    

The `background-position-x` property is specified as one or more values, separated by commas.

### [Values](#values)

[`left`](#left)

Aligns the left edge of the background image with the left edge of the background position layer.

[`center`](#center)

Aligns the center of the background image with the center of the background position layer.

[`right`](#right)

Aligns the right edge of the background image with the right edge of the background position layer.

[`<length>`](/en-US/docs/Web/CSS/length)

The offset of the given background image's left vertical edge from the background position layer's left vertical edge. (Some browsers allow assigning the right edge for offset).

[`<percentage>`](/en-US/docs/Web/CSS/percentage)

The offset of the given background image's horizontal position relative to the container. A value of 0% means that the left edge of the background image is aligned with the left edge of the container, and a value of 100% means that the _right_ edge of the background image is aligned with the _right_ edge of the container, thus a value of 50% horizontally centers the background image.

[Formal definition](#formal_definition)
---------------------------------------

[Initial value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#initial_value)

`0%`

Applies to

all elements. It also applies to [`::first-letter`](/en-US/docs/Web/CSS/::first-letter) and [`::first-line`](/en-US/docs/Web/CSS/::first-line).

[Inherited](/en-US/docs/Web/CSS/CSS_cascade/Inheritance)

no

Percentages

refer to width of background positioning area minus width of background image

[Computed value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#computed_value)

A list, each item consisting of: an offset given as a combination of an absolute length and a percentage, plus an origin keyword

[Animation type](/en-US/docs/Web/CSS/CSS_animated_properties)

a repeatable list

[Formal syntax](#formal_syntax)
-------------------------------

background-position-x =   
  [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") center [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") left [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") right [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") x-start [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") x-end [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")[?](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#question_mark "Question mark: the entity is optional") <length-percentage>[?](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#question_mark "Question mark: the entity is optional") [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")! [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")[#](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#hash_mark "Hash mark: the entity is repeated one or several times, each occurrence separated by a comma")    
  
<length-percentage> =   
  [<length>](/en-US/docs/Web/CSS/length)      [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  [<percentage>](/en-US/docs/Web/CSS/percentage)    

Sources: [CSS Backgrounds Module Level 4](https://drafts.csswg.org/css-backgrounds-4/), [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/)

[Examples](#examples)
---------------------

### [Basic example](#basic_example)

The following example shows a background image implementation, with background-position-x and background-position-y used to define the image's horizontal and vertical positions separately.

#### HTML

html

    <div></div>
    

#### CSS

css

    div {
      width: 300px;
      height: 300px;
      background-color: skyblue;
      background-image: url(https://mdn.dev/archives/media/attachments/2020/07/29/17350/3b4892b7e820122ac6dd7678891d4507/firefox.png);
      background-repeat: no-repeat;
      background-position-x: center;
      background-position-y: bottom;
    }
    

#### Result

### [Side-relative values](#side-relative_values)

The following example shows support for side-relative offset syntax, which allows the developer to offset the background from any edge.

#### HTML

html

    <div></div>
    

#### CSS

css

    div {
      width: 300px;
      height: 300px;
      background-color: seagreen;
      background-image: url(https://mdn.dev/archives/media/attachments/2020/07/29/17350/3b4892b7e820122ac6dd7678891d4507/firefox.png);
      background-repeat: no-repeat;
      background-position-x: right 20px;
      background-position-y: bottom 10px;
    }
    

#### Result

[Specifications](#specifications)
---------------------------------

Specification

[CSS Backgrounds Module Level 4  
\# background-position-longhands](https://drafts.csswg.org/css-backgrounds-4/#background-position-longhands)

[Browser compatibility](#browser_compatibility)
-----------------------------------------------

[See also](#see_also)
---------------------

*   [`background-position`](/en-US/docs/Web/CSS/background-position)
*   [`background-position-y`](/en-US/docs/Web/CSS/background-position-y)
*   [Using multiple backgrounds](/en-US/docs/Web/CSS/CSS_backgrounds_and_borders/Using_multiple_backgrounds)



background-position-y
=====================

Baseline Widely available \*

This feature is well established and works across many devices and browser versions. It’s been available across browsers since September 2016.

\* Some parts of this feature may have varying levels of support.

*   [Learn more](/en-US/docs/Glossary/Baseline/Compatibility)
*   [See full compatibility](#browser_compatibility)
*   [Report feedback](https://survey.alchemer.com/s3/7634825/MDN-baseline-feedback?page=%2Fen-US%2Fdocs%2FWeb%2FCSS%2Fbackground-position-y&level=high)

The **`background-position-y`** [CSS](/en-US/docs/Web/CSS) property sets the initial vertical position for each background image. The position is relative to the position layer set by [`background-origin`](/en-US/docs/Web/CSS/background-origin).

[Try it](#try_it)
-----------------

    background-position-y: top;
    

    background-position-y: center;
    

    background-position-y: 25%;
    

    background-position-y: 2rem;
    

    background-position-y: bottom 32px;
    

    <section class="display-block" id="default-example">
      <div class="transition-all" id="example-element"></div>
    </section>
    

    #example-element {
      background-color: navajowhite;
      background-image: url("/shared-assets/images/examples/star.png");
      background-repeat: no-repeat;
      height: 100%;
    }
    

The value of this property is overridden by any declaration of the [`background`](/en-US/docs/Web/CSS/background) or [`background-position`](/en-US/docs/Web/CSS/background-position) shorthand properties applied to the element after it.

[Syntax](#syntax)
-----------------

css

    /* Keyword values */
    background-position-y: top;
    background-position-y: center;
    background-position-y: bottom;
    
    /* <percentage> values */
    background-position-y: 25%;
    
    /* <length> values */
    background-position-y: 0px;
    background-position-y: 1cm;
    background-position-y: 8em;
    
    /* Side-relative values */
    background-position-y: bottom 3px;
    background-position-y: bottom 10%;
    
    /* Multiple values */
    background-position-y: 0px, center;
    
    /* Global values */
    background-position-y: inherit;
    background-position-y: initial;
    background-position-y: revert;
    background-position-y: revert-layer;
    background-position-y: unset;
    

The `background-position-y` property is specified as one or more values, separated by commas.

### [Values](#values)

[`top`](#top)

Aligns the top edge of the background image with the top edge of the background position layer.

[`center`](#center)

Aligns the vertical center of the background image with the vertical center of the background position layer.

[`bottom`](#bottom)

Aligns the bottom edge of the background image with the bottom edge of the background position layer.

[`<length>`](/en-US/docs/Web/CSS/length)

The offset of the given background image's horizontal edge from the corresponding background position layer's top horizontal edge. (Some browsers allow assigning the bottom edge for offset).

[`<percentage>`](/en-US/docs/Web/CSS/percentage)

The offset of the given background image's vertical position relative to the container. A value of 0% means that the top edge of the background image is aligned with the top edge of the container, and a value of 100% means that the _bottom_ edge of the background image is aligned with the _bottom_ edge of the container, thus a value of 50% vertically centers the background image.

[Formal definition](#formal_definition)
---------------------------------------

[Initial value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#initial_value)

`0%`

Applies to

all elements. It also applies to [`::first-letter`](/en-US/docs/Web/CSS/::first-letter) and [`::first-line`](/en-US/docs/Web/CSS/::first-line).

[Inherited](/en-US/docs/Web/CSS/CSS_cascade/Inheritance)

no

Percentages

refer to height of background positioning area minus height of background image

[Computed value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#computed_value)

A list, each item consisting of: an offset given as a combination of an absolute length and a percentage, plus an origin keyword

[Animation type](/en-US/docs/Web/CSS/CSS_animated_properties)

a repeatable list

[Formal syntax](#formal_syntax)
-------------------------------

background-position-y =   
  [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") center [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") top [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") bottom [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") y-start [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") y-end [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")[?](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#question_mark "Question mark: the entity is optional") <length-percentage>[?](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#question_mark "Question mark: the entity is optional") [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")! [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")[#](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#hash_mark "Hash mark: the entity is repeated one or several times, each occurrence separated by a comma")    
  
<length-percentage> =   
  [<length>](/en-US/docs/Web/CSS/length)      [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  [<percentage>](/en-US/docs/Web/CSS/percentage)    

Sources: [CSS Backgrounds Module Level 4](https://drafts.csswg.org/css-backgrounds-4/), [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/)

[Examples](#examples)
---------------------

### [Basic example](#basic_example)

The following example shows a background image implementation, with background-position-x and background-position-y used to define the image's horizontal and vertical positions separately.

#### HTML

html

    <div></div>
    

#### CSS

css

    div {
      width: 300px;
      height: 300px;
      background-color: skyblue;
      background-image: url(https://mdn.dev/archives/media/attachments/2020/07/29/17350/3b4892b7e820122ac6dd7678891d4507/firefox.png);
      background-repeat: no-repeat;
      background-position-x: center;
      background-position-y: bottom;
    }
    

#### Result

### [Side-relative values](#side-relative_values)

The following example shows support for side-relative offset syntax, which allows the developer to offset the background from any edge.

#### HTML

html

    <div></div>
    

#### CSS

css

    div {
      width: 300px;
      height: 300px;
      background-color: seagreen;
      background-image: url(https://mdn.dev/archives/media/attachments/2020/07/29/17350/3b4892b7e820122ac6dd7678891d4507/firefox.png);
      background-repeat: no-repeat;
      background-position-x: right 20px;
      background-position-y: bottom 10px;
    }
    

#### Result

[Specifications](#specifications)
---------------------------------

Specification

[CSS Backgrounds Module Level 4  
\# background-position-longhands](https://drafts.csswg.org/css-backgrounds-4/#background-position-longhands)

[Browser compatibility](#browser_compatibility)
-----------------------------------------------

[See also](#see_also)
---------------------

*   [`background-position`](/en-US/docs/Web/CSS/background-position)
*   [`background-position-x`](/en-US/docs/Web/CSS/background-position-x)
*   [Using multiple backgrounds](/en-US/docs/Web/CSS/CSS_backgrounds_and_borders/Using_multiple_backgrounds)



background-repeat
=================

Baseline Widely available

This feature is well established and works across many devices and browser versions. It’s been available across browsers since July 2015.

*   [Learn more](/en-US/docs/Glossary/Baseline/Compatibility)
*   [See full compatibility](#browser_compatibility)
*   [Report feedback](https://survey.alchemer.com/s3/7634825/MDN-baseline-feedback?page=%2Fen-US%2Fdocs%2FWeb%2FCSS%2Fbackground-repeat&level=high)

The **`background-repeat`** [CSS](/en-US/docs/Web/CSS) property sets how background images are repeated. A background image can be repeated along the horizontal and vertical axes, or not repeated at all.

[Try it](#try_it)
-----------------

    background-repeat: repeat-x;
    

    background-repeat: repeat;
    

    background-repeat: space;
    

    background-repeat: round;
    

    background-repeat: no-repeat;
    

    background-repeat: space repeat;
    

    <section id="default-example">
      <div id="example-element"></div>
    </section>
    

    #example-element {
      background: #ccc url("/shared-assets/images/examples/moon.jpg") center / 120px;
      min-width: 100%;
      min-height: 100%;
    }
    

[Syntax](#syntax)
-----------------

css

    /* Keyword values */
    background-repeat: repeat;
    background-repeat: repeat-x;
    background-repeat: repeat-y;
    background-repeat: space;
    background-repeat: round;
    background-repeat: no-repeat;
    
    /* Two-value syntax: horizontal | vertical */
    background-repeat: repeat space;
    background-repeat: repeat repeat;
    background-repeat: round space;
    background-repeat: no-repeat round;
    
    /* Global values */
    background-repeat: inherit;
    background-repeat: initial;
    background-repeat: revert;
    background-repeat: revert-layer;
    background-repeat: unset;
    

[Description](#description)
---------------------------

The property accepts a comma-separated list of two [`<repeat-style>`](#values) keyterms, or one keyterm as a shorthand for the two values. When two values are provided, the first value defines the horizontal repetition behavior and the second value defines the vertical behavior. Property values can be used to repeat only horizontally, vertically, or not at all.

The default value is `repeat repeat`. With this value, the background image maintains its intrinsic [aspect ratio](/en-US/docs/Glossary/Aspect_ratio), repeating both horizontally and vertically to cover the entire background paint area, with edge images being clipped to the size of the element. Which edges clipped depends on the value of the corresponding [`background-position`](/en-US/docs/Web/CSS/background-position) value. How many times they are repeated and how much the images on the edges are clipped depends on the size of the background painting area and the corresponding [`background-size`](/en-US/docs/Web/CSS/background-size) value.

The repeating images can be evenly spaced apart, ensuring the repeated image maintains its aspect ratio without being clipped. With the `space` value, if the background paint area has a different aspect ratio than the image or does not otherwise have a size that is a multiple of the background size in either direction, there will be areas not covered by the background image.

Alternatively, the repeated background image can be stretched to cover the entire area without clipping. With `round`, the repeated image is stretched to fill all the available space until there is room to add an additional repeated image if the aspect ratio of the background image is not the same as the paint area's aspect ratio. For example, given a background image that is 100px x 100px and a background paint area of 1099px x 750px, the image will be repeated 10 times in the horizontal direction and 7 times vertically, for a total of 70 repetitions, with each image stretched in both directions to be 109.9px x 105px, altering the image's aspect ratio and potentially distorting it. If the width of the paint area increases by 1px, becoming 1100px wide, an 11th image will fit horizontally for a total of 77 image repetitions, with each image being painted at 100px wide and 105px tall, stretched only in the vertical direction.

[Values](#values)
-----------------

The property accepts a comma-separated list of two `<repeat-style>` keyterms or one keyterm as a shorthand for the two values. The first value is the horizontal repetition. The second value is the vertical behavior. If only a single value is set to a value other than `repeat-x` or `repeat-y`, that value is applied the both vertices. The values include:

[`repeat`](#repeat)

The default value. The image is repeated as many times as needed to cover the entire background image painting area, with the edge image being clipped if the dimension of the painting area is not a multiple of the dimension of your background image.

[`no-repeat`](#no-repeat)

The image is not repeated (and hence the background image painting area will not necessarily be entirely covered). The position of the non-repeated background image is defined by the [`background-position`](/en-US/docs/Web/CSS/background-position) CSS property.

[`space`](#space)

The image is repeated as much as possible without clipping. The first and last images are pinned to either side of the element, and whitespace is distributed evenly between the images. The [`background-position`](/en-US/docs/Web/CSS/background-position) property is ignored unless only one image can be displayed without clipping. The only case where clipping happens using `space` is when there isn't enough room to display one image.

[`round`](#round)

As the allowed space increases in size, the repeated images will stretch (leaving no gaps) until there is room for another one to be added. This is the only `<repeat-style>` value that can lead to the distortion of the background image's [aspect ratio](/en-US/docs/Glossary/Aspect_ratio), which will occur if the aspect ratio of the background image differs from the aspect ratio of the background paint area.

[`repeat-x`](#repeat-x)

Shorthand for `repeat no-repeat`, the background image repeats horizontally only, with the edge image being clipped if the width of the paint area is not a multiple of the background image's width.

[`repeat-y`](#repeat-y)

Shorthand for `no-repeat repeat`, the background image repeats vertically only, with the edge image being clipped if the height of the paint area is not a multiple of the background image's height.

When one `<repeat-style>` keyterm is provided, the value is shorthand for the following two-value syntax:

Single value

Two-value equivalent

`repeat-x`

`repeat no-repeat`

`repeat-y`

`no-repeat repeat`

`repeat`

`repeat repeat`

`space`

`space space`

`round`

`round round`

`no-repeat`

`no-repeat no-repeat`

[Formal definition](#formal_definition)
---------------------------------------

[Initial value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#initial_value)

`repeat`

Applies to

all elements. It also applies to [`::first-letter`](/en-US/docs/Web/CSS/::first-letter) and [`::first-line`](/en-US/docs/Web/CSS/::first-line).

[Inherited](/en-US/docs/Web/CSS/CSS_cascade/Inheritance)

no

[Computed value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#computed_value)

a list, each item consisting of two keywords, one per dimension

[Animation type](/en-US/docs/Web/CSS/CSS_animated_properties)

discrete

[Formal syntax](#formal_syntax)
-------------------------------

background-repeat =   
  <repeat-style>[#](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#hash_mark "Hash mark: the entity is repeated one or several times, each occurrence separated by a comma")    
  
<repeat-style> =   
  repeat-x                                     [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  repeat-y                                     [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") repeat [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") space [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") round [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") no-repeat [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")[{1,2}](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#curly_braces "Curly braces: encloses two integers defining the minimal and maximal numbers of occurrences of the entity, or a single integer defining the exact number required")    

Sources: [CSS Backgrounds and Borders Module Level 3](https://drafts.csswg.org/css-backgrounds-3/)

[Examples](#examples)
---------------------

### [Setting background-repeat](#setting_background-repeat)

#### HTML

html

    <ol>
      <li>
        no-repeat
        <div class="one"></div>
      </li>
      <li>
        repeat
        <div class="two"></div>
      </li>
      <li>
        repeat-x
        <div class="three"></div>
      </li>
      <li>
        repeat-y
        <div class="four"></div>
      </li>
      <li>
        space
        <div class="five"></div>
      </li>
      <li>
        round
        <div class="six"></div>
      </li>
      <li>
        repeat-x, repeat-y (multiple images)
        <div class="seven"></div>
      </li>
    </ol>
    

#### CSS

css

    /* Shared for all DIVS in example */
    ol,
    li {
      margin: 0;
      padding: 0;
    }
    li {
      margin-bottom: 12px;
    }
    div {
      background-image: url(star-solid.gif);
      width: 160px;
      height: 70px;
    }
    
    /* Background repeats */
    .one {
      background-repeat: no-repeat;
    }
    .two {
      background-repeat: repeat;
    }
    .three {
      background-repeat: repeat-x;
    }
    .four {
      background-repeat: repeat-y;
    }
    .five {
      background-repeat: space;
    }
    .six {
      background-repeat: round;
    }
    
    /* Multiple images */
    .seven {
      background-image:
        url(star-solid.gif), url(/shared-assets/images/examples/favicon32.png);
      background-repeat: repeat-x, repeat-y;
      height: 144px;
    }
    

#### Result

In this example, each list item is matched with a different value of `background-repeat`.

[Specifications](#specifications)
---------------------------------

Specification

[CSS Backgrounds and Borders Module Level 3  
\# the-background-repeat](https://drafts.csswg.org/css-backgrounds/#the-background-repeat)

[Browser compatibility](#browser_compatibility)
-----------------------------------------------

[See also](#see_also)
---------------------

*   The other [`background`](/en-US/docs/Web/CSS/background) shorthand components: [`background-attachment`](/en-US/docs/Web/CSS/background-attachment), [`background-clip`](/en-US/docs/Web/CSS/background-clip), [`background-color`](/en-US/docs/Web/CSS/background-color), [`background-image`](/en-US/docs/Web/CSS/background-image), [`background-origin`](/en-US/docs/Web/CSS/background-origin), [`background-position`](/en-US/docs/Web/CSS/background-position) ([`background-position-x`](/en-US/docs/Web/CSS/background-position-x) and [`background-position-y`](/en-US/docs/Web/CSS/background-position-y)), and [`background-size`](/en-US/docs/Web/CSS/background-size)
*   [Using multiple backgrounds](/en-US/docs/Web/CSS/CSS_backgrounds_and_borders/Using_multiple_backgrounds)
*   [CSS backgrounds and borders](/en-US/docs/Web/CSS/CSS_backgrounds_and_borders/Using_multiple_backgrounds) module
*   [Understanding aspect ratios](/en-US/docs/Web/CSS/CSS_box_sizing/Understanding_aspect-ratio)


background-size
===============

Baseline Widely available

This feature is well established and works across many devices and browser versions. It’s been available across browsers since July 2015.

*   [Learn more](/en-US/docs/Glossary/Baseline/Compatibility)
*   [See full compatibility](#browser_compatibility)
*   [Report feedback](https://survey.alchemer.com/s3/7634825/MDN-baseline-feedback?page=%2Fen-US%2Fdocs%2FWeb%2FCSS%2Fbackground-size&level=high)

The **`background-size`** [CSS](/en-US/docs/Web/CSS) property sets the size of the element's background image. The image can be left to its natural size, stretched, or constrained to fit the available space.

[Try it](#try_it)
-----------------

    background-size: contain;
    

    background-size: contain;
    background-repeat: no-repeat;
    

    background-size: cover;
    

    background-size: 30%;
    

    background-size: 200px 100px;
    

    <section id="default-example">
      <div class="transition-all" id="example-element"></div>
    </section>
    

    #example-element {
      background-image: url("/shared-assets/images/examples/hand.jpg");
      min-width: 100%;
      min-height: 100%;
    }
    

Spaces not covered by a background image are filled with the [`background-color`](/en-US/docs/Web/CSS/background-color) property, and the background color will be visible behind background images that have transparency/translucency.

[Syntax](#syntax)
-----------------

css

    /* Keyword values */
    background-size: cover;
    background-size: contain;
    
    /* One-value syntax */
    /* the width of the image (height becomes 'auto') */
    background-size: 50%;
    background-size: 3.2em;
    background-size: 12px;
    background-size: auto;
    
    /* Two-value syntax */
    /* first value: width of the image, second value: height */
    background-size: 50% auto;
    background-size: 3em 25%;
    background-size: auto 6px;
    background-size: auto auto;
    
    /* Multiple backgrounds */
    background-size: auto, auto; /* Not to be confused with `auto auto` */
    background-size: 50%, 25%, 25%;
    background-size: 6px, auto, contain;
    
    /* Global values */
    background-size: inherit;
    background-size: initial;
    background-size: revert;
    background-size: revert-layer;
    background-size: unset;
    

The `background-size` property is specified in one of the following ways:

*   Using the keyword values `contain` or `cover`.
*   Using a width value only, in which case the height defaults to `auto`.
*   Using both a width and a height value, in which case the first sets the width and the second sets the height. Each value can be a [`<length>`](/en-US/docs/Web/CSS/length), a [`<percentage>`](/en-US/docs/Web/CSS/percentage), or `auto`.

To specify the size of multiple background images, separate the value for each one with a comma.

### [Values](#values)

[`contain`](#contain)

Scales the image as large as possible within its container without cropping or stretching the image. If the container is larger than the image, this will result in image tiling, unless the [`background-repeat`](/en-US/docs/Web/CSS/background-repeat) property is set to `no-repeat`.

[`cover`](#cover)

Scales the image (while preserving its ratio) to the smallest possible size to fill the container (that is: both its height and width completely _cover_ the container), leaving no empty space. If the proportions of the background differ from the element, the image is cropped either vertically or horizontally.

[`auto`](#auto)

Scales the background image in the corresponding direction such that its intrinsic proportions are maintained.

[`<length>`](/en-US/docs/Web/CSS/length)

Stretches the image in the corresponding dimension to the specified length. Negative values are not allowed.

[`<percentage>`](/en-US/docs/Web/CSS/percentage)

Stretches the image in the corresponding dimension to the specified percentage of the _background positioning area_. The background positioning area is determined by the value of [`background-origin`](/en-US/docs/Web/CSS/background-origin) (by default, the padding box). However, if the background's [`background-attachment`](/en-US/docs/Web/CSS/background-attachment) value is `fixed`, the positioning area is instead the entire [viewport](/en-US/docs/Glossary/Viewport). Negative values are not allowed.

### [Intrinsic dimensions and proportions](#intrinsic_dimensions_and_proportions)

The computation of values depends on the image's intrinsic dimensions (width and height) and intrinsic proportions (width-to-height ratio). These attributes are as follows:

*   A bitmap image (such as JPG) always has intrinsic dimensions and proportions.
*   A vector image (such as SVG) does not necessarily have intrinsic dimensions. If it has both horizontal and vertical intrinsic dimensions, it also has intrinsic proportions. If it has no dimensions or only one dimension, it may or may not have proportions.
*   CSS [`<gradient>`](/en-US/docs/Web/CSS/gradient)s have no intrinsic dimensions or intrinsic proportions.
*   Background images created with the [`element()`](/en-US/docs/Web/CSS/element) function use the intrinsic dimensions and proportions of the generating element.

**Note:** In Gecko, background images created using the [`element()`](/en-US/docs/Web/CSS/element) function are currently treated as images with the dimensions of the element, or of the background positioning area if the element is SVG, with the corresponding intrinsic proportion. This is non-standard behavior.

Based on the intrinsic dimensions and proportions, the rendered size of the background image is computed as follows:

*   **If both components of `background-size` are specified and are not `auto`:** The background image is rendered at the specified size.
    
*   **If the `background-size` is `contain` or `cover`:** While preserving its intrinsic proportions, the image is rendered at the largest size contained within, or covering, the background positioning area. If the image has no intrinsic proportions, then it's rendered at the size of the background positioning area.
    
*   **If the `background-size` is `auto` or `auto auto`:**
    
    *   If the image has both horizontal and vertical intrinsic dimensions, it's rendered at that size.
    *   If the image has no intrinsic dimensions and has no intrinsic proportions, it's rendered at the size of the background positioning area.
    *   If the image has no intrinsic dimensions but has intrinsic proportions, it's rendered as if `contain` had been specified instead.
    *   If the image has only one intrinsic dimension and has intrinsic proportions, it's rendered at the size corresponding to that one dimension. The other dimension is computed using the specified dimension and the intrinsic proportions.
    *   If the image has only one intrinsic dimension but has no intrinsic proportions, it's rendered using the specified dimension and the other dimension of the background positioning area.
    
    **Note:** SVG images have a [`preserveAspectRatio`](/en-US/docs/Web/SVG/Reference/Attribute/preserveAspectRatio) attribute that defaults to the equivalent of `contain`; an explicit `background-size` causes `preserveAspectRatio` to be ignored.
    
*   **If the `background-size` has one `auto` component and one non-`auto` component:**
    
    *   If the image has intrinsic proportions, it's stretched to the specified dimension. The unspecified dimension is computed using the specified dimension and the intrinsic proportions.
    *   If the image has no intrinsic proportions, it's stretched to the specified dimension. The unspecified dimension is computed using the image's corresponding intrinsic dimension, if there is one. If there is no such intrinsic dimension, it becomes the corresponding dimension of the background positioning area.

**Note:** Background sizing for vector images that lack intrinsic dimensions or proportions is not yet fully implemented in all browsers. Be careful about relying on the behavior described above, and test in multiple browsers to be sure the results are acceptable.

[Formal definition](#formal_definition)
---------------------------------------

[Initial value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#initial_value)

`auto auto`

Applies to

all elements. It also applies to [`::first-letter`](/en-US/docs/Web/CSS/::first-letter) and [`::first-line`](/en-US/docs/Web/CSS/::first-line).

[Inherited](/en-US/docs/Web/CSS/CSS_cascade/Inheritance)

no

Percentages

relative to the background positioning area

[Computed value](/en-US/docs/Web/CSS/CSS_cascade/Value_processing#computed_value)

as specified, but with relative lengths converted into absolute lengths

[Animation type](/en-US/docs/Web/CSS/CSS_animated_properties)

a repeatable list

[Formal syntax](#formal_syntax)
-------------------------------

background-size =   
  <bg-size>[#](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#hash_mark "Hash mark: the entity is repeated one or several times, each occurrence separated by a comma")    
  
<bg-size> =   
  [\[](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component") <length-percentage \[0,∞\]> [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present") auto [\]](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#brackets "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component")[{1,2}](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#curly_braces "Curly braces: encloses two integers defining the minimal and maximal numbers of occurrences of the entity, or a single integer defining the exact number required")  [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  cover                                      [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  contain                                      
  
<length-percentage> =   
  [<length>](/en-US/docs/Web/CSS/length)      [|](/en-US/docs/Web/CSS/CSS_Values_and_Units/Value_definition_syntax#single_bar "Single bar: exactly one of the entities must be present")  
  [<percentage>](/en-US/docs/Web/CSS/percentage)    

Sources: [CSS Backgrounds and Borders Module Level 3](https://drafts.csswg.org/css-backgrounds-3/), [CSS Values and Units Module Level 4](https://drafts.csswg.org/css-values-4/)

[Examples](#examples)
---------------------

### [Tiling a large image](#tiling_a_large_image)

Let's consider a large image, a 2982x2808 Firefox logo image. We want to tile four copies of this image into a 300x300-pixel element. To do this, we can use a fixed `background-size` value of 150 pixels.

#### HTML

html

    <div class="tiledBackground"></div>
    

#### CSS

css

    .tiledBackground {
      background-image: url(https://www.mozilla.org/media/img/logos/firefox/logo-quantum.9c5e96634f92.png);
      background-size: 150px;
      width: 300px;
      height: 300px;
      border: 2px solid;
      color: pink;
    }
    

#### Result

See [Resizing background images](/en-US/docs/Web/CSS/CSS_backgrounds_and_borders/Resizing_background_images) for more examples.

[Specifications](#specifications)
---------------------------------

Specification

[CSS Backgrounds and Borders Module Level 3  
\# the-background-size](https://drafts.csswg.org/css-backgrounds/#the-background-size)

[Browser compatibility](#browser_compatibility)
-----------------------------------------------

[See also](#see_also)
---------------------

*   [Resizing background images](/en-US/docs/Web/CSS/CSS_backgrounds_and_borders/Resizing_background_images)
*   [Scaling SVG backgrounds](/en-US/docs/Web/CSS/CSS_backgrounds_and_borders/Scaling_of_SVG_backgrounds)
*   [`object-fit`](/en-US/docs/Web/CSS/object-fit)
