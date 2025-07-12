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
when user is typing inside of a url function, we need to provide auto complete feature as they type.

we need to look at the path and check the file system and predict what user is trying to type according to real files and folders that exists.

Eg.

if user type `project:///Assets/F` then we look at all files and folder inside of `Assets` folder in project and see what starts with `F`(case sensitive) and if we find anything,  we should show them in auto complete.(Note `.meta` files should be ignored).

Also note that we should start showing a list of possible items starting with each `/` at the path, example, if user typed `project:///Assets/Folder/`, then we need to look at what is inside of `Assets/Folder` in the project and list all of them.

If the url function's syntax tree structure is invalid (eg. have an open `"` but forgot about a closing `"` or have 2 arguments), then we don't need to provide autocompletion for paths.

Also, I should add that if url argument is not quoted(ie. a plain node in syntax tree), we should also not provide auto completion even though this can be valid. (Because it can be error prone for us to get right and user should always quote the string anyway).
