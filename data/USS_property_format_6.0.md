
Position element with the layout engine

USS common properties
=====================

This page introduces the common USS properties, their syntax and accepted values, and differences from CSS. For a complete list of USS properties, see [USS properties reference](UIE-USS-Properties-Reference.html).

All
---

The `all` property resets all properties to their default value. This property doesn’t apply to the custom USS properties.

    all: initial
    

Box model
---------

![Box model](../uploads/Main/style-box-model.png)

Box model

### Dimensions

    width: <length> | auto
    height: <length> | auto
    min-width: <length> | auto
    min-height: <length> | auto
    max-width: <length> | none
    max-height: <length> | none
    

The `width` and `height` specify the size of the element. If `width` isn’t specified, the width is based on the width of the element’s contents. If `height` isn’t specified, the height is based on the height of the element’s contents.

### Margins

    margin-left: <length> | auto;
    margin-top: <length> | auto
    margin-right: <length> | auto
    margin-bottom: <length> | auto
    /* Shorthand */
    margin: [<length> | auto]{1,4}
    

### Borders

    border-left-width: <length>
    border-top-width: <length>
    border-right-width: <length>
    border-bottom-width: <length>
    /* Shorthand */
    border-width: <length>{1,4}
    

### Padding

    padding-left: <length>
    padding-top: <length>
    padding-right: <length>
    padding-bottom: <length>
    /* Shorthand */
    padding: <length>{1,4}
    

### Differences from CSS

The alternative box model that USS uses is different from the [standard CSS box model](https://developer.mozilla.org/en-US/docs/Learn/CSS/Building_blocks/The_box_model#What_is_the_CSS_box_model). In the standard CSS box model, `width` and `height` define the size of the content box. An element’s rendered size is the sum of its `padding`, `border-width`, and `width` / `height` values.

Unity’s model is equivalent to setting the CSS `box-sizing` property to `border-box`. See the [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/CSS/box-sizing) for details.

Flex layout
-----------

UI Toolkit includes a [layout engine](UIE-LayoutEngine.html) that positions **visual elements**A node of a visual tree that instantiates or derives from the C# [`VisualElement`](../ScriptReference/UIElements.VisualElement.html) class. You can style the look, define the behaviour, and display it on screen as part of the UI. [More info](UIE-VisualTree.html)  
See in [Glossary](Glossary.html#Visualelement) based on layout and styling properties. The layout engine implements a subset of Flexbox, an HTML/CSS layout system.

By default, all items are vertically placed in their container.

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
    

Positioning
-----------

    /* The default value is `relative` which positions the element based on its parent.
    If sets to `absolute`, the element leaves its parent layout and values are specified based on the parent bounds.*/
    position: absolute | relative
    
    /* The distance from the parent edge or the original position of the element. */
    left: <length> | auto
    top: <length> | auto
    right: <length> | auto
    bottom: <length> | auto
    

Background
----------

    background-color: <color>
    background-image: <resource> | <url> | none
    -unity-background-scale-mode: stretch-to-fill | scale-and-crop | scale-to-fit
    -unity-background-image-tint-color: <color>
    

For more information about setting background images, refer to [Set background images](UIB-styling-ui-backgrounds.html).

Slicing
-------

When assigning a background image, you draw it with respect to a simplified 9-slice specification:

    -unity-slice-left: <integer>
    -unity-slice-top: <integer>
    -unity-slice-right: <integer>
    -unity-slice-bottom: <integer>
    -unity-slice-scale: <length>
    -unity-slice-type: sliced | tiled
    

**Note**: For **sprites**A 2D graphic objects. If you are used to working in 3D, Sprites are essentially just standard textures but there are special techniques for combining and managing sprite textures for efficiency and convenience during development. [More info](sprite/sprite-landing.html)  
See in [Glossary](Glossary.html#Sprite), Unity adjusts the `-unity-slice-scale` by the sprite’s `pixels-per-unit` value in relation to the panel’s `reference sprite pixels per unit value`, which is by default `100`. For example, if the sprite’s `pixels-per-unit` is `16`, the scale is adjusted by `16/100 = 0.16`. Therefore, if you set the scale to `2px`, the final scale is `2px * 0.16px = 0.32px`. For texture and vector images, Unity doesn’t make additional adjustments to the slice scale value you set.

For more information about 9-slice, refer to [9-Slice images with UI Toolkit](UIB-styling-ui-backgrounds.html#9-slice-images-with-ui-toolkit).

Border color
------------

    border-left-color: <color>
    border-top-color: <color>
    border-right-color: <color>
    border-bottom-color: <color>
    /* Shorthand */
    border-color: <color>{1,4}
    

Border radius
-------------

    border-top-left-radius: <length>
    border-top-right-radius: <length>
    border-bottom-left-radius: <length>
    border-bottom-right-radius: <length>
    /* Shorthand */
    border-radius: <length>{1,4}
    

### Differences from CSS

Border radius properties work almost the same in USS and CSS. For detailed information about `border-radius`, refer to the [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/CSS/border-radius).

However, there are two main differences:

*   Unity doesn’t support the second-radius shorthand (`border-radius: (first radius values) / (second radius values);`) used to create elliptical corners.
*   Unity reduces border radius values to half of the element’s size in **pixels**The smallest unit in a computer image. Pixel size depends on your screen resolution. Pixel lighting is calculated at every screen pixel. [More info](ShadowPerformance.html)  
    See in [Glossary](Glossary.html#pixel). For example, for a 100px x 100px element, any `border-radius` value greater than 50px is reduced to 50px. If you use percentage (`%`) values for border-radius properties, Unity first resolves the percentage to pixels and then reduces the `border-radius` value to half of the resolved pixel value. If you have different radius values for two or more corners, Unity reduces any values greater than half of the element’s size to half of the element’s size.

Appearance
----------

    overflow: hidden | visible
    -unity-overflow-clip-box: padding-box | content-box
    -unity-paragraph-spacing: <length>
    visibility: visible | hidden
    display: flex | none
    

The `-unity-overflow-clip-box` defines the clipping rectangle for the element content. The default is `padding-box`, the rectangle outside the padding area (the green rectangle in the box model image above); `content-box` represents the value inside the padding area (the blue rectangle in the box model image above).

The `display` default value is `flex`. Set `display` to `none` to remove the element from the UI. Set the `visibility` to `hidden` to hide the element, but the element still occupies space in the layout.

The `overflow` property controls the clipping of an element’s content. The default value is `visible`, which means the element’s content isn’t clipped to the element’s bounds. If you set `overflow` to `hidden`, the element’s content is clipped to the element’s bounds. You can use `overflow` to [make a masking effect](UIE-masking.html).

### Differences from CSS

The USS `display` property supports only a small subset of the CSS `display` property’s available keyword values. The USS version supports keywords that work with the Yoga layout engine.

*   For more information about Yoga, refer to [Flexible Layouts with Yoga](https://yogalayout.com/).
*   For more information about the CSS `display` property, refer to the [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/CSS/display).

Text properties
---------------

Text properties set the color, font, font size, and Unity-specific properties for font resource, font style, alignment, word wrap, and clipping.

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
    /* clip: Trims any text that extends beyond the boudaries of its container. */
    /* ellipsis: Truncates any text that extends beyong the boudaries of its container with an ellipsis. */
    text-overflow: clip | ellipsis
    
    text-shadow: <x-offset> <y-offset> <blur-radius> <color>;
    
    letter-spacing: <length>
    word-spacing: <length>
    -unity-paragraph-spacing: <length>
    

**Note**: When you set up the font in UI Builder, the **Font** control in the ****Inspector**A Unity window that displays information about the currently selected GameObject, asset or project settings, allowing you to inspect and edit the values. [More info](UsingTheInspector.html)  
See in [Glossary](Glossary.html#Inspector)** sets `-unity-font`, and the **Font Asset** control sets `-unity-font-definition`. Because `-unity-font-definition` takes precedence over `-unity-font`, to use a font from the **Font** list, select **None** from **Font Asset**. Otherwise, the font you selected doesn’t take effect.

For more information about text shadow and text outline, refer to [Text effects](UIE-text-effects.html).

Cursor
------

The `cursor` property specifies the mouse cursor to be displayed when the mouse pointer is over an element.

    cursor: [ [ <resource> | <url> ] [ <integer> <integer>]? , ] [ arrow | text | resize-vertical | resize-horizontal | link | slide-arrow | resize-up-right | resize-up-left | move-arrow | rotate-arrow | scale-arrow | arrow-plus | arrow-minus | pan | orbit | zoom | fps | split-resize-up-down | split-resize-left-right ]
    

**Note**: Cursor keywords are only available in the Editor UI. Cursor keywords don’t work in runtime UI. In runtime UI, you must use a texture for custom cursors.

### Differences from CSS

In CSS, you can specify multiple optional custom cursors and a mandatory keyword in a single `cursor` style declaration. When you specify multiple cursors, they form a fallback chain. If the browser can’t load the first custom cursor, it tries each of the others in sequence until one of them loads or there are no more custom cursors to try. If the browser can’t load any custom cursors, it uses the keyword.

In USS, custom cursors and keywords are mutually exclusive. A `cursor` style declaration can only have one custom cursor or one keyword.

For detailed information about the CSS `cursor` property, see the [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/CSS/cursor).

Opacity
-------

    opacity: <number>
    

### Differences from CSS

USS opacity is similar to [CSS opacity](https://developer.mozilla.org/en-US/docs/Web/CSS/opacity). The opacity of parent elements affects the **perceived opacity** of child elements. The perceivability is different between USS opacity and CSS opacity.

In the following USS example, the blue square is a child of the red square. The red square has an `opacity` of `0.5`.

    .red {
        background-color: red;
        opacity: 0.5;
    }
    
    .blue {
        background-color: blue;
        left: 20px;
        top: 20px;
    }
    

Although the blue square doesn’t have an opacity value, it has a perceived opacity of `0.5` from the red square. You can see the red square through the blue square.



USS transform
=============

The transform properties apply a 2D transformation to a **visual element**A node of a visual tree that instantiates or derives from the C# [`VisualElement`](../ScriptReference/UIElements.VisualElement.html) class. You can style the look, define the behaviour, and display it on screen as part of the UI. [More info](UIE-VisualTree.html)  
See in [Glossary](Glossary.html#Visualelement). You can use them to rotate, scale, or move a visual element.

If you change the layout of an element, Unity recalculates the layout of other elements in the same hierarchy. This recalculation might reduce an animation’s frame rate. Applying transform to an element reduces recalculations because it doesn’t change the layout of other elements in the hierarchy.

It’s possible to use transform to define the static appearance of a visual element. However, transform is best used for changes and animations. For example, if you want to make a visual element shake when an event happens in an application, set the position of the visual element using the regular layout properties such as `top` and `left`, and then use `translate` to align an offset relative to the initial position.

Transform includes the following properties:

  

**Property**

**USS syntax**

**Description**

**Transform Origin**

`transform-origin`

Represents the point of origin where rotation, scaling, and translation occur.

**Translate**

`translate`

Repositions the visual element in horizontal or vertical directions.

**Scale**

`scale`

Changes the apparent size, padding, border, and margins of a visual element. Negative values flip visual elements along the scale axis.

**Rotate**

`rotate`

Rotates a visual element. Positive values represent clockwise rotation and negative values represent counterclockwise rotation. You can set rotation with `deg`, `grad`, `rad`, or `turn` units. For more information on these units, see [MDN Web Docs’s page on the `<angle>` CSS data type](https://developer.mozilla.org/en-US/docs/Web/CSS/angle).

**Note**: All transformations are performed in the following order:

1.  Scale
2.  Rotate
3.  Translate

You can set transform properties for a visual element using the controls in the [UI Builder](UIBuilder.html), within a [USS](UIE-USS.html) file, or using a C# script.

Transform controls in the UI Builder
------------------------------------

You can use the controls in the **Transform** section of the ****Inspector**A Unity window that displays information about the currently selected GameObject, asset or project settings, allowing you to inspect and edit the values. [More info](UsingTheInspector.html)  
See in [Glossary](Glossary.html#Inspector)** in the UI Builder to set the transform properties of a visual element.

### Pivot Origin

The **Pivot Origin** widget sets the transform origin property. To use it, do one of the following:

*   Click a point in the widget to set the origin to a corner, the center of an edge, or the center. You can also define the values using the keyboard. When the widget is in focus, use the arrow keys to specify a point in the widget.
*   Enter values for **X** and **Y** and specify the unit.

**Tip**: You can enter `%` or `px` after values. This automatically changes the displayed unit in the unit selector. You can also drag to define the values in the **X** and **Y** boxes.

**Note**: The default value for the transform origin is center.

If you use percentages for both the X and Y values, the widget shows the new origin point when you edit the X and Y text boxes.

If you specify a transform origin point outside the element, such as having a value less than 0% or greater than 100%, the widget shows the directions of the X and Y axes.

![A transform origin with an X value less than 0, and a Y value greater than 100. The pivot origin is highlighted in the bottom left corner.](../uploads/Main/TransformOriginWgt.png)

A transform origin with an X value less than 0, and a Y value greater than 100. The pivot origin is highlighted in the bottom left corner.

### Translate

The **Translate** control sets the translate property. To use it, enter values in the **X** and **Y** boxes and specify the unit.

**Tip**: You can enter `%` or `px` after values. This automatically changes the displayed unit in the unit selector. You can also drag to define the values in the **X** and **Y** boxes.

### Scale

The **Scale** control sets the scale property. To use it, enter values in the **X** and **Y** boxes and specify the unit.

**Tip**: You can enter `%` or `px` after values. This automatically changes the displayed unit in the unit selector. You can also drag to define the values in the **X** and **Y** boxes.

### Rotate

The **Rotate** control sets the rotate property. To use it, enter a value and specify the unit.

**Tip**: You can type `deg`, `grad`, `rad`, or `turn` after a value in the **Rotate** box. This automatically changes the displayed unit in the unit selector.

USS transform properties
------------------------

You can use styling rules to set the transform properties for a visual element. You can set the styling rules within a USS file or inline in a UXML file.

### `transform-origin`

The `transform-origin` property sets the transform origin along the X and Y axes in **pixels**The smallest unit in a computer image. Pixel size depends on your screen resolution. Pixel lighting is calculated at every screen pixel. [More info](ShadowPerformance.html)  
See in [Glossary](Glossary.html#pixel) or percentages.

You can also use keywords to set the value of the `transform-origin` attribute. These keywords match the dots in the widget in the UI Builder. The following keywords are supported:

 

**Pivot point**

**Keywords**

**Center**

*   `center`
*   `center center` (This is the default value)

**Center of left edge**

*   `left`
*   `left center`
*   `center left`

**Center of right edge**

*   `right`
*   `right center`
*   `center right`

**Center of top edge**

*   `top`
*   `top center`
*   `center top`

**Center of bottom edge**

*   `bottom`
*   `bottom center`
*   `center bottom`

**Top-left corner**

*   `top left`
*   `left top`

**Top-right corner**

*   `top right`
*   `right top`

**Bottom-left corner**

*   `bottom left`
*   `left bottom`

**Bottom-right corner**

*   `bottom right`
*   `right bottom`

**Examples**

    transform-origin: 0% 100%;
    transform-origin: 20px 10px;
    transform-origin: 0px 100%;
    transform-origin: 60% 10px;
    

### `translate`

The `translate` property sets the translation along the X and Y axes in pixels or percentages relative to the size of this visual element. You can omit Y if it equals X.

**Examples**

    translate: 80%;
    translate: 35px;
    translate: 5% 10px;
    translate: 24px 0%;
    

### `scale`

The `scale` property sets the scale along the X and Y axes in pixels or percentages. You can omit Y if it equals X.

The keyword `none` sets no scale.

**Examples**

    scale: 2.5;
    scale: -1 1;
    scale: none;
    

### `rotate`

The `rotate` property sets the rotation using a number or a unit.

The keyword `none` sets no rotation.

**Examples**

    rotate: 45deg;
    rotate: -100grad;
    rotate: -3.14rad;
    rotate: 0.75turn;
    rotate: none;
    

Transform C# properties
-----------------------

You can set the transform properties for a visual element in a C# script.

### `IStyle.transformOrigin`

The [`IStyle.transformOrigin`](../ScriptReference/UIElements.IStyle-transformOrigin.html) property sets the transform origin.

The `transformOrigin` property of the [`style`](../ScriptReference/UIElements.VisualElement-style.html) is of type [`StyleTransformOrigin`](../ScriptReference/UIElements.StyleTransformOrigin.html). Its [constructor](../ScriptReference/UIElements.StyleTransformOrigin-ctor.html) takes a [`TransformOrigin`](../ScriptReference/UIElements.TransformOrigin.html) as an argument. You can construct a new [`TransformOrigin`](../ScriptReference/UIElements.TransformOrigin.html) using an X value and a Y value. The X value and the Y value are of type [`Length`](../ScriptReference/UIElements.Length.html).

**Examples**

    //This example sets the transform origin of the element to be 100 pixels from the left edge and 50% of the way down from the top edge.
    element.style.transformOrigin = new StyleTransformOrigin(new TransformOrigin(new Length(100f, LengthUnit.Pixel), new Length(50f, LengthUnit.Percent)));
    

You can simplify the above code as follows using implicit conversions:

    element.style.transformOrigin = new TransformOrigin(100, Length.Percent(50));
    

### `IStyle.translate`

The [`IStyle.translate`](../ScriptReference/UIElements.IStyle-translate.html) property sets the translation.

[`IStyle.translate`](../ScriptReference/UIElements.IStyle-translate.html) is of type [`StyleTranslate`](../ScriptReference/UIElements.StyleTranslate.html). Its constructor takes a [`Translate`](../ScriptReference/UIElements.Translate.html) as an argument. You can construct a new [`Translate`](../ScriptReference/UIElements.Translate.html) using an X value and a Y value. The X value and the Y value are of type [`Length`](../ScriptReference/UIElements.Length.html).

**Examples**

    //This example sets the translation of the element. The X-axis is 10% and the Y-axis is 50 pixels.
    element.style.translate = new StyleTranslate(new Translate(new Length(10f, LengthUnit.Percent), new Length(50f, LengthUnit.Pixel)));
    

You can simplify the above code as follows using implicit conversions:

    element.style.translate = new Translate(Length.Percent(10), 50);
    

### `IStyle.scale`

The [`IStyle.scale`](../ScriptReference/UIElements.IStyle-scale.html) property sets the scale.

[`IStyle.scale`](../ScriptReference/UIElements.IStyle-scale.html) is of type [`StyleScale`](../ScriptReference/UIElements.StyleScale.html). [`StyleScale`](../ScriptReference/UIElements.StyleScale.html)’s [constructor](../ScriptReference/UIElements.StyleScale-ctor.html) takes a [Scale](../ScriptReference/UIElements.Scale.html) as an argument. You can [construct](../ScriptReference/UIElements.Scale-ctor.html) this `Scale` with a `Vector2`.

**Examples**

    element.style.scale = new StyleScale(new Scale(new Vector2(0.5f, -1f)));
    

You can simplify the code above as follows using implicit conversions:

    element.style.scale = new Scale(new Vector2(0.5f, -1));
    

### `IStyle.rotate`

The [`IStyle.rotate`](../ScriptReference/UIElements.IStyle-rotate.html) property sets the rotation.

The [`IStyle.rotate`](../ScriptReference/UIElements.IStyle-rotate.html) property is of type [`StyleRotate`](../ScriptReference/UIElements.StyleRotate.html). The [`StyleRotate`](../ScriptReference/UIElements.StyleRotate.html)’s [constructor](../ScriptReference/UIElements.StyleRotate-ctor.html) takes a [`Rotate`](../ScriptReference/UIElements.Rotate.html) as an argument. You can [construct](../ScriptReference/UIElements.Rotate-ctor.html) this `Rotate` with an [`Angle`](../ScriptReference/UIElements.Angle.html). You can [construct](../ScriptReference/UIElements.Angle-ctor.html) an `Angle` with a float and an optional [`AngleUnit`](../ScriptReference/UIElements.AngleUnit.html) enum, or you can use static methods [`Degrees()`](../ScriptReference/UIElements.Angle.Degrees.html), [`Gradians()`](../ScriptReference/UIElements.Angle.Gradians.html), [`Radians()`](../ScriptReference/UIElements.Angle.Radians.html), and [`Turns()`](../ScriptReference/UIElements.Angle.Turns.html).

**Examples**

    //Rotate by 180 degrees
    elements[0].style.rotate = new StyleRotate(new Rotate(new Angle(180f, AngleUnit.Degree)));
    //Rotate by 200 gradians
    elements[1].style.rotate = new StyleRotate(new Rotate(new Angle(200f, AngleUnit.Gradian)));
    //Rotate by pi radians
    elements[2].style.rotate = new StyleRotate(new Rotate(new Angle(Mathf.PI, AngleUnit.Radian)));
    //Rotate by half a turn
    elements[3].style.rotate = new StyleRotate(new Rotate(new Angle(0.5f, AngleUnit.Turn)));
    

You can simplify the above code as follows:

    //Rotate by 180 degrees
    elements[0].style.rotate = new Rotate(180);
    //Rotate by 200 gradians
    elements[1].style.rotate = new Rotate(Angle.Gradians(200));
    //Rotate by pi radians
    elements[2].style.rotate = new Rotate(Angle.Radians(Mathf.PI));
    //Rotate by half a turn
    elements[3].style.rotate = new Rotate(Angle.Turns(0.5f));
    

USS transition
==============

USS transitions are similar to [CSS transitions](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Transitions). A USS transition changes property values over a given duration. You can use a USS transition to create an animation that changes the appearance of a **visual element**A node of a visual tree that instantiates or derives from the C# [`VisualElement`](../ScriptReference/UIElements.VisualElement.html) class. You can style the look, define the behaviour, and display it on screen as part of the UI. [More info](UIE-VisualTree.html)  
See in [Glossary](Glossary.html#Visualelement). For example, you can use a UI transition to make UI elements that change size or color when a user hovers their cursor over them.

You can use the controls in the [UI Builder](UIBuilder.html), a [USS file](UIE-USS.html), or a C# script to set the transition properties for a visual element.

The following table lists the transition properties and their corresponding C# methods:

  

**Property**

**C# method**

**Description**

`transition-property`

[`IStyle.transitionProperty`](../ScriptReference/UIElements.IStyle-transitionProperty.html)

Which USS properties the transition applies to.

`transition-duration`

[`IStyle.transitionDuration`](../ScriptReference/UIElements.IStyle-transitionDuration.html)

How long the transition takes to complete.

`transition-timing-function`

[`IStyle.transitionTimingFunction`](../ScriptReference/UIElements.IStyle-transitionTimingFunction.html)

How the property moves between values over time.

`transition-delay`

[`IStyle.transitionDelay`](../ScriptReference/UIElements.IStyle-transitionDelay.html)

When the transition starts.

`transition`

Shorthand for `transition-property`, `transition-duration`, `transition-timing-function`, and `transition-delay`.

Starting of a transition
------------------------

A transition triggers if you set a transition duration on the style and the style value changes. You can use pseudo-classes, C# methods, or events to trigger the transition.

**Note**: A transition animation on a frame is triggered when the property’s current state is different from the previous state. The first frame in a **scene**A Scene contains the environments and menus of your game. Think of each unique Scene file as a unique level. In each Scene, you place your environments, obstacles, and decorations, essentially designing and building your game in pieces. [More info](CreatingScenes.html)  
See in [Glossary](Glossary.html#Scene) has no previous state. You must start a transition animation after the first frame.

The following transition example changes the color and rotation of the label when you hover over it. The transition has a duration of 2 seconds.

![A transition example](../uploads/Main/UIBuilder/transition-gif.gif)

A transition example

To implement this example, do the following:

1.  Set the transition properties for the visual element.
2.  Set the transition duration.
3.  Set the start and end style values.

The example USS looks like this:

    /* Set the transition properties, duration, and start style values. */
    .labelClass {
        transition-property: color, rotate;
        transition-duration: 2s;
        color: black;
    }
    
    /* The Label:hover triggers the transition. Set the end values for the trigger. */
    .labelClass:hover {
        rotate: 10deg;
        color: red;
    } 
    

**Note**: The example sets the transition on the element rather than the `:hover`. If you set the transition on the `:hover`, the transition doesn’t work if the cursor leaves the element.

To learn how to trigger a transition with C# events, refer to [Create a simple transition with UI Builder and C# scripts](UIE-transition-example.html).

### Match the value units

For properties that you set with a value and unit, make sure the units match. Pay special attention to transitions to or from default values. For example, the default value of the `translate` attribute is `0px`. If you try to transition from this value to a percentage value, the transition doesn’t work.

The following transition example offsets the visual element to the left by `50px` over 2 seconds. The default value of the `left` property is `auto`. You must explicitly set the unit to `0px` for the transition to work.

![An offset to left transition example](../uploads/Main/UIBuilder/transition-offset-left.gif)

An offset to left transition example

The example USS looks like this:

    .boxClass:hover {
        left: 50px;
    }
    
    .boxClass {
        transition-property: left;
        transition-duration: 2s;
        transition-timing-function: ease-in-out-sine;
        left: 0px;
    }
    

### Transitions for an inherited property

For visual elements in a hierarchy, USS transitions behave the same as [CSS transitions](https://drafts.csswg.org/css-transitions/#starting). If you set transitions for an inherited property, such as `color`, transitions applied to the parent elements cascade to the child elements. To find out which property is inherited, refer to [USS property reference](UIE-USS-Properties-Reference.html).

### Interrupt transitions

When an incomplete transition is `interruptedSame`, USS transitions behave the same as CSS transitions. The reverse transition might be faster. For more information, refer to [Faster reversing of interrupted transitions](https://drafts.csswg.org/css-transitions/#reversing)

### Transition events

[Transition events](UIE-Transition-Events.html) are generated by transitions. You can use them to detect when a transition starts and ends. For an example, refer to [Create a transition event](UIE-transition-event-example.html).

### Usage hints

The **Usage Hints** offers a set of [properties](../ScriptReference/UIElements.UsageHints.html) for optimizations. You can use it to reduce draw calls and geometry regeneration.

**Note**: Set the usage hints at edit time or before you add the element to a panel. When the transition starts, the system might automatically add missing relevant usage hints to avoid regenerating geometry in every frame. However, this causes a one-frame performance penalty because the rendering data for the VisualElement and its descendants is regenerated.

Transition controls in the UI Builder
-------------------------------------

You can use the controls in the **Transition Animations** section of the ****Inspector**A Unity window that displays information about the currently selected GameObject, asset or project settings, allowing you to inspect and edit the values. [More info](UsingTheInspector.html)  
See in [Glossary](Glossary.html#Inspector)** in the UI Builder to set transition rules for a visual element. You can set multiple transitions on a visual element. To add another transition, select **Add Transition**. To remove a transition, select the **Remove (−)** button.

![This transition causes a visual element to adjust its scale over 500 milliseconds in a linear fashion after a delay of 20 milliseconds.](../uploads/Main/UIBuilder/example-transition.png)

This transition causes a visual element to adjust its scale over 500 milliseconds in a linear fashion after a delay of 20 milliseconds.

Transition property
-------------------

The transition property defines which USS properties the transition applies to.

### Keywords

The transition property supports the following keywords:

*   `all`: Applies transitions to all properties and overrides any preceding transitions. This is the default value.
*   `initial`: Applies transitions to all properties.
*   `none`: Ignores transitions for all properties.
*   `ignored`: Ignores transitions for the specified duration, delay, and easing function.

### Animatability

You can apply transitions to most USS properties. However, the animatability for certain properties is different. The animatability of USS properties falls into the following categories:

*   **Fully animatable**: Supports transition from the start value to the end value, at a speed that follows the easing function and transition duration.
*   **Discrete**: Supports transition between values in a single step from the start value to the end value.
*   **Non-animatable**: Doesn’t support transition.

To find out the animatability of each property, see [USS property reference](UIE-USS-Properties-Reference.html).

**Note**: It’s recommended that you use transitions with the [USS transform properties](UIE-Transform.html). Although you can use transitions on other USS properties, it might result in animations with low frame rates because value changes on these properties might cause layout recalculations. Layout recalculations in each frame can slow down the frame rate of your transition animation. All color properties, such as `color`, `background-color`, tint, and `opacity`, also have a fast pass that prevents the regeneration of the geometry.

### USS examples

You can supply a single USS property, a keyword, or a comma-separated list of them to `transition-property`.

    transition-property: scale;
    transition-property: translate, all, rotate;
    transition-property: initial;
    transition-property: none;
    

### C# examples

The [`IStyle.transitionProperty`](../ScriptReference/UIElements.IStyle-transitionProperty.html) property is of type `StyleList<StylePropertyName>`. [`StylePropertyName`](../ScriptReference/UIElements.StylePropertyName.html) is a struct that you can [construct](../ScriptReference/UIElements.StylePropertyName-ctor.html) from a string. [`StyleList`](../ScriptReference/UIElements.StyleList_1.html) is a struct you can [construct](../ScriptReference/UIElements.StyleList_1-ctor.html) from a list of `StylePropertyName`.

    //Create a list that contains the rotate property, and use it to set transitionProperty.
    List<StylePropertyName> properties = new List<StylePropertyName>();
    properties.Add(new StylePropertyName("rotate"));
    //Given a VisualElement named "element"...
    element.style.transitionProperty = new StyleList<StylePropertyName>(properties);
    

You can use implicit conversions to simplify the above code as follows:

    //Given a VisualElement named "element"...
    element.style.transitionProperty = new List<StylePropertyName> { "rotate" };
    

Transition duration
-------------------

The transition duration sets how long the transition takes to complete.

### Keywords

The transition duration supports the following keywords:

*   `initial`: Sets the duration to `0s`. This is the default value.

### USS examples

You can supply a number with a unit, a keyword, or a comma-separated list of numbers and units to `transition-duration`.

    transition-duration: 2s;
    transition-duration: 800ms;
    transition-duration: 3s, 1500ms, 1.75s;
    transition-duration: initial;
    

If you supply multiple values, each value applies to the corresponding property specified in `transition-property`. In the following example, the original duration for scale is `1s`, but `all` overrides it to `2s`.

    transition-property: scale, all, rotate;
    transition-duration: 1s, 2s, 3s;
    

### C# examples

The [`IStyle.transitionDuration`](../ScriptReference/UIElements.IStyle-transitionDuration.html) property is of type `StyleList<TimeValue>`. [`TimeValue`](../ScriptReference/UIElements.TimeValue.html) is a struct that you can [construct](../ScriptReference/UIElements.TimeValue-ctor.html) from a number and a [`TimeUnit`](../ScriptReference/UIElements.TimeUnit.html) enum. [`StyleList`](../ScriptReference/UIElements.StyleList_1.html) is a struct you can [construct](../ScriptReference/UIElements.StyleList_1-ctor.html) from a list of `TimeValue`.

    //Create a list that contains durations 2s and 500ms and use it to set transitionDuration.
    List<TimeValue> durations = new List<TimeValue>();
    durations.Add(new TimeValue(2f, TimeUnit.Second));
    durations.Add(new TimeValue(500f, TimeUnit.Millisecond));
    //Given a VisualElement named "element"...
    element.style.transitionDuration = new StyleList<TimeValue>(durations);
    

You can use implicit conversions to simplify the above code as follows:

    //Given a VisualElement named "element"...
    element.style.transitionDuration = new List<TimeValue> { 2, new (500, TimeUnit.Millisecond) };
    

Transition timing function
--------------------------

The transition timing function sets how the property moves between values over time.

### Keywords

The transition timing function supports the following keywords. The default value is `initial`, which sets the easing function to `ease`.

*   `initial`
*   `ease`
*   `ease-in`
*   `ease-out`
*   `ease-in-out`
*   `linear`
*   `ease-in-sine`
*   `ease-out-sine`
*   `ease-in-out-sine`
*   `ease-in-cubic`
*   `ease-out-cubic`
*   `ease-in-out-cubic`
*   `ease-in-circ`
*   `ease-out-circ`
*   `ease-in-out-circ`
*   `ease-in-elastic`
*   `ease-out-elastic`
*   `ease-in-out-elastic`
*   `ease-in-back`
*   `ease-out-back`
*   `ease-in-out-back`
*   `ease-in-bounce`
*   `ease-out-bounce`
*   `ease-in-out-bounce`

For detailed information about each function, refer to [MDN’s documentation for the `transition-timing-function` CSS attribute](https://developer.mozilla.org/en-US/docs/Web/CSS/transition-timing-function) or [easings.net](https://easings.net/).

### USS examples

You can supply an easing function, a keyword, or a comma-separated list of easing functions to `transition-timing-function`.

    transition-timing-function: linear;
    transition-timing-function: ease-in, ease-out-circ, ease-in-out-cubic;
    transition-timing-function: initial;
    

### C# examples

The [`IStyle.transitionTimingFunction`](../ScriptReference/UIElements.IStyle-transitionTimingFunction.html) property is of type `StyleList<EasingFunction>`. [`EasingFunction`](../ScriptReference/UIElements.EasingFunction.html) is a struct that you can [construct](../ScriptReference/UIElements.EasingFunction-ctor.html) from an [`EasingMode`](../ScriptReference/UIElements.EasingMode.html) enum.

    //Create a list that contains the Linear easing function, and use it to set transitionTimingFunction.
    List<EasingFunction> easingFunctions = new List<EasingFunction>();
    easingFunctions.Add(new EasingFunction(EasingMode.Linear));
    //Given a VisualElement named "element"...
    element.style.transitionTimingFunction = new StyleList<EasingFunction>(easingFunctions);
    

You can use implicit conversions to simplify the above code as follows:

    //Given a VisualElement named "element"...
    element.style.transitionTimingFunction = new List<EasingFunction> { EasingMode.Linear };
    

Transition delay
----------------

The transition delay sets when the transition starts.

### Keywords

The transition delay supports the following keywords:

*   `initial`: Sets the delay to `0s`. This is the default value.

### USS examples

You can supply a number with a unit, a keyword, or a comma-separated list of numbers and units to `transition-delay`.

    transition-delay: 0s;
    transition-delay: 300ms;
    transition-delay: 2s, 650ms, 2.75s;
    transition-delay: initial;
    

### C# examples

The [`IStyle.transitionDelay`](../ScriptReference/UIElements.IStyle-transitionDelay.html) property is of type `StyleList<TimeValue>`. [`TimeValue`](../ScriptReference/UIElements.TimeValue.html) is a struct that you can [construct](../ScriptReference/UIElements.TimeValue-ctor.html) from a number and a [`TimeUnit`](../ScriptReference/UIElements.TimeUnit.html) enum. [`StyleList`](../ScriptReference/UIElements.StyleList_1.html) is a struct you can [construct](../ScriptReference/UIElements.StyleList_1-ctor.html) from a list of `TimeValue`.

    //Create a list that contains delays 0.5s and 200ms, and use it to set transitionDelay.
    List<TimeValue> delays = new List<TimeValue>();
    delays.Add(new TimeValue(0.5f, TimeUnit.Second));
    delays.Add(new TimeValue(200f, TimeUnit.Millisecond));
    //Given a VisualElement named "element"...
    element.style.transitionDelay = new StyleList<TimeValue>(delays);
    

You can use implicit conversions to simplify the above code as follows:

    //Given a VisualElement named "element"...
    element.style.transitionDelay = new List<TimeValue> { 0.5f, new(200, TimeUnit.Millisecond) };
    

USS `transition`
----------------

You can supply one transition, a keyword, or a comma-separated list of transitions to `transition`. You separate properties within a transition by a space in the following order:

1.  `transition-property`
2.  `transition-delay`
3.  `transition-duration`
4.  `transition-timing-function`

### Keywords

`transition` only supports the keyword `initial`, which represents the initial value of each transition property:

*   `transition-delay`: `0s`
*   `transition-duration`: `0s`
*   `transition-property`: `all`
*   `transition-timing-function`: `ease`

### USS examples

    /*One transition*/
    transition: width 2s ease-out;
    
    /*Two transitions*/
    transition: margin-right 4s, color 1s;
    

Transition on multiple property examples
----------------------------------------

This section includes examples that apply transitions on multiple properties.

### Example 1

This example applies transitions on the `scale` and `transform-origin` properties:

*   The first transition is on the `scale` property. It has a duration of `4s`, a delay of `0s`, and the `ease-in-sine` easing function.
*   The second transition is on the `transform-origin` property. It has a duration of `3s`, a delay of `600ms`, and the `ease-out-elastic` easing function.

    .classA {
        transition-property: scale, transform-origin;
        transition-duration: 4s, 3s;
        transition-delay: 0s, 600ms;
        transition-timing-function: ease-in-sine, ease-out-elastic;
    }
    

### Example 2

In this example, the later transitions override earlier transitions, including transitions with the `all` keyword:

*   The first transition is on all properties. It applies a duration of 500 milliseconds, a delay of zero seconds, and the `linear` easing function.
*   The second transition is on the `translate` property. It overrides the transition with a duration of `1s`, a delay of `1s`, and the `ease-in` easing function. All other properties still have a duration of `500ms`, a delay of `0s`, and the `linear` easing function.

    .classB {
        transition-property: all, translate;
        transition-duration: 500ms, 1s;
        transition-delay: 0s, 1s;
        transition-timing-function: linear, ease-in;
    }
    

### Example 3

This example shows what happens when property value lists are of different lengths. If any property’s list of values is shorter than that for `transition-property`, Unity repeats its values to make them match. Similarly, if any property’s value list is longer than `transition-property`, Unity truncates it.

    .classC {
        transition-property: scale, rotate, translate;
        transition-duration: 1s, 2s;
        transition-delay: 1s, 2s, 3s, 4s, 5s, 6s, 7s;
    }
    

The following table shows the final results for the example above:

   

**Property**

**Duration**

**Delay**

**Easing function**

`scale`

`1s`

`1s`

`ease`

`rotate`

`2s`

`2s`

`ease`

`translate`

`1s`

`3s`

`ease`

**Important**: `transition-property`, `transition-duration`, `transition-delay`, and `transition-timing-function` are separate USS properties. If you leave any of them undefined, it’s possible that they are defined elsewhere, such as in another USS rule or inline on the UXML element.
