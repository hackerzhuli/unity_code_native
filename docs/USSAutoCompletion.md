# USS Auto Completion

## Properties
After user typed `:` for a valid property, we give a list of most likely values for that property for user to select.

If user selected any item, then we should also remember to automatically insert a space before and a semicolon after;

Note that as user is typing after `:`, eg. typing `color:r` then we are going to look at what is our best guess of what is next and try to give a list of most likely values for that property.

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