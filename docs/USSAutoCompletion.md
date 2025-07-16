# USS Auto Completion

## Properties
After user typed `:` for a valid property, we give a list of most likely values for that property for user to select.

If user selected any item, then we should also remember to automatically insert a space before(if there are no space yet after the colon) and a `;` after(not for properties that support mulitple values, ie. comma seperated values, we don't add a `;` after).

Note that as user is typing after `:`, eg. typing `color:r` then we are going to look at what is our best guess of what is next and try to give a list of most likely values for that property.

Note that we have a limitation, if user just typed whitespaces after the colon and nothing else, we can't provide auto completion for that property. Only user just typed the colon or after user type some character that is not whitespace after colon.

We have special auto completion logic for color properties and `transition-property`(see below).

If the property is not these, then we will use a general logic, which is, find all single keywords that will work for this property, and show them. If no single keyword would work, we should not show any completion item.

### Color Properties
For color properties we should complete with color keywords along with other single keywords that will work for the property.

### transition-property
For `transition-property`, we should complete with all propeties that are animatable(including discrete), and also all single keywords that will work for the property.

### Single Keywords
What does it mean, when we say single keyword that will work (for a property)? Meaning that if the property only have one value that is that keyword, it is valid. eg. `flex-direction: row;`, just a single keyword, and this is valid.

### Comma seperated values
For properties that can have multiple values(ie. comma seperated values), then after each comma, the completion logic is the same as if we are after the first colon. Trigger the completion at the comma and then narrow down the list as user key typing.

## Selectors
Id selectors and class selectors, after user had typed the `#`  for id and `.` for class selector, we need to display a list of id or class selectors according to what is present in the same source file. And narrow down the list as user type.

## Pseudo classes
After user type `:`, we need to display a list of pseudo classes that is valid for that selector. As user type, we should also narrow down the list like other autocompletion features.

## Elements
Elements are defined in xsd files and we should read these and get all element names, and as user type an element selector (eg. `Button`) we should narrow down the list. I will not detail where the xsd files are and how to manage them here.

## url
See the [dedicated doc](./UrlCompletion.md)

## At rule or import statement
Since USS only have one at rule that is import, we should provide auto completion right when user typed `@`, and after that if user input still matches `@import`, after cursor leave `@import`, eg, user typed a space after that, no auto completion is provided.

We should give user a basic structure of import statement, here are the items we provide, each in one line(complete with semicolon, always have url function with quotes):

```css
@import url("project:///Assets"); /*the recommended way, use project scheme*/
@import url(""); /*empty so user an type anything, especially relative paths*/
@import url("/Assets"); /* absolute path, shorter but less used */
```

Note, if user picked an completion item, try to move the cursor to right before the closing quote, so that user can keep typing the path, if possible.