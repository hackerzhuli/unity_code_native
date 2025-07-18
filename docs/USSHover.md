# Uss Hover
## Order
the hover logic should try to find hover documenation in the same order as discribed in this document. If something if found, we should just show that, and stop the process.

## No Error
When we show a hover for a node, we must first make sure the node and all parents up until the root node are not error nodes.

## functions
If mouse is over a supported function, like a url function, a rgb function, something like that, we should documentation for the function.

## Unit
If mouse is over a supported unit, like px, %, something like that, we should documentation for the unit.

## import statement
If mouse is on import statement, we show the documentation for import statement.

## tag selector
if mouse is over a tag selector, and the it is a known tag(ie. a known UXML element), then we just show the full name of the uxml element(ie. class name including namespace).

## pseudo class selector
If mouse is over a pseudo class selector, and the it is a valid pseudo class selector, then we just show the documentation for the pseudo class.

## declaration
If it is known property, show the property's documentation, when mouse is hover over the declaration node. But if the property is not known, always show nothing, even if mouse if over a known keyword value inside of the declartion node. This is the general case.

If mouse is over a value node inside of the declaration(ie. inside a direct children of the declaration, the value nodes), if the value is a known keyword or a property, then show the documentation for the keyword or property.

How to decide if the identifier value is a keyword or property?

if the name of the property of the declaration node is `transition` or `transition-property`, then we will see if the identifier is an animatable property, if it is, we show the documentation for the property.

In other cases, we will assume the identifier is a keyword.

If they are not known property names or keyword names, we just show the default documentation for the declaration node, same as general case.

