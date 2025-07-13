# Auto Completion for Urls in Unity Project
## Overview
This is auto completion feature for Urls seen in uss/uxml, Unity's UIToolkit.

## Example
for uss url can appear inside a url function or inside a import statement, here is an example:

```uss
@import "var.uss";    
@import url("project:///Assets/UI/test.uss")
@import "project://AABBCC/Assets/UI/test.uss?fileID=7433441132597879392&amp;guid=f05797c550ffb7e4fa0d5346ee5edf95&amp;type=3#test";
 
.haha Button {
    background-image: url("PROJECT:///Assets/UI/1.png");
}
```

for uxml url is for importing stylesheets, example:
```uxml
<ui:UXML xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:ui="UnityEngine.UIElements" xmlns:uie="UnityEditor.UIElements" noNamespaceSchemaLocation="../../UIElementsSchema/UIElements.xsd" editor-extension-mode="False">
    <Style src="project://database/Assets/UI/MyStyle.uss?fileID=7433441132597879392&amp;guid=1ae8690cfe60b1c4789c4d4546355806&amp;type=3#MyStyle" />
</ui:UXML>
```

## Some basic rules
Urls can be written as an absolute path, either full with `project` scheme or start with `/` that is absolute path.

Otherwise it is a relative path, relative to the source file this url is from. Relative paths are fully supported and can use standard relative path notation:
- `../` to navigate to parent directory
- `./` to reference current directory
- Multiple `../` can be chained (e.g., `../../`) to navigate multiple levels up
- Direct filename or directory name (e.g., `components.uss`, `subfolder/file.png`) - relative to current directory

### Relative Path Examples
```uss
/* From Assets/UI/Styles/main.uss, reference a file in Assets/UI/ */
@import "../components.uss";

/* From Assets/UI/Styles/main.uss, reference a file in Assets/Resources/ */
background-image: url("../../Resources/background.png");

/* From Assets/UI/main.uss, reference a file in the same directory */
@import "./variables.uss";
@import "variables.uss"; /* Same as above - no dot prefix needed */

/* From Assets/UI/main.uss, reference a file in a subdirectory */
background-image: url("Images/icon.png");
```

## Limitation
Our auto completion feature should support both languages, uss and uxml as their url syntax is the same.

Limitation: since uss have escape sequences, we don't support that in our url auto completion, it is very uncommon for user to write escape sequences in url, .ie, use baskslash to escape things.

Of course we do support percent encoding and whatever is standard in a url string.

Also, the string must be quoted, otherwise we don't support autocompletion. eg. `url(image.png)` is valid css/uss, but we don't support it, the url argument must be quoted, either single quote or double quote.


## Completion Logic for path part of url
Once we only deal with pure url that is quoted, without escape sequences(backslash based), we can write a general url completion that works for both languages.

The specific completion logic is, when user is typing scheme or authority, we don't do auto completion, our main focus is the path portion of the url.

After user type a `/` (with or without a scheme) in the path part, we start doing auto completion. For relative paths, completion also works after typing `../` or `./`. We will look at what is inside of the target directory and give user a list of all of the items, including directories and files(but ignore `.meta` files). As user type more characters that is not `/`, we narrow down the list to what matches.

Relative path completion resolves the target directory based on the current file's location and the relative path components, then provides completions for that resolved directory.

## Completion logic for query and fragment part of url
urls actually allow query that will allow user to specify asset guid and fileId, etc, which will make the url more robust, because even if user moved a file, the guid/fileId will still be valid. So nothing will break.

We only support completion for query part of url, if the url is explicitly `project` scheme (meaning the `project` scheme is present in the source string, this means it can't be a relative path), and the path is a valid and existing asset(and not a directory) in the project, then we will allow completion for query part of url.

we ONLY do auto completion when user just typed `?`(if user keeps typing after that, we don't do auto completion for query), we will give users a list of possible entries, each represents an subasset of the asset the path is point at.

Most asset is a single asset, that is one file is one asset, not subassets. But some asset types can have subassets, like an image, in Unity image can be divided into Sprites, allow user to specify a Sprite, eg. for an background image in uss.

here is an example of the full url with query string `"project://database/Assets/UI/test.uss?fileID=7433441132597879392&amp;guid=f05797c550ffb7e4fa0d5346ee5edf95&amp;type=3#test"`, fileID is the subasset id inside of the asset, guid is the guid of the asset, type is the type for the subasset(or the asset if there is no subasset). The fragment part is just a name of the subasset, or the name of the file if there is no subasset.

The info about these file id, guid can be read from the `.meta` file of the asset, which we will detail on a seperate document.




