TODO list:
- [x] Improve hover on for import statement in uss, because now it doesn't have any hover.
- [x] Improve hover for url path in uss, so that we show a clickable link for a file that does exist
- [ ] Improve hover for property to include syntax and examples and better description to make the hover doc for property better, also as for property value that is keyword, we should also include what that keyword does, hover docs is the most efficient way for user to get what he needed, so it must be good
- [x] Add auto completion for properties and values
- [ ] Add support for tss files, also note that "theme://default" (or something like that) is actually valid url in tss import statement, it is a special case. Should not create an error for that.
- [ ] Let our uss file and tss file use the same icon as css file in vs code
- [ ] Format document and format selection(both uss and tss), try to figure out a way to use exising css format ability in VS Code or what ever that can avoid to do it ourselves, because format is the same for uss as css
- [ ] Basic refactor feature to rename id selectors and class selector in a uss file
- [x] Add auto completion to actual VisualElement types
- [x] Add url argument auto completion(in which case we should detect what is actually on the filesytem to help user type faster, also we should add query string to includ guid file id and such to make the url complete)(for query parameters, we may not be able to do it perfectly because in the case of sprites, we may not be able to show user the sprites inside of the file)
- [ ] Add auto completion for @import when user just typed an @ , we assume they typing an import statement, and give user a basic structure complete with quotes and semicolon. Give completion items to show a relative path or a absolute path or with project scheme.   If possible put cursor before closing quote so user can keep typing the path.
- [x] Add Auto completion for pseudo classes
- [ ] Make sure our hover will cover everything that needs some docs, including pseudo classes!
- [ ] Add a warning for duplicate property in same block, that is probably a mistake
- [ ] Add docs when auto completing pseudo classes

TODO for consideration:
- [ ] (We should do this later, it is complex because it involves Unity Editor) Add code action for url when url does include a guid, but file doesn't exist or guid doesn't match, then we can offer a code action to fix it, typically it is because user moved a file, which involves messaging Unity Editor, because we need to locate the asset, which could fail due to non existence of the guid or Unity Editor is busy
- [ ] (Not needed now because it usually doesn't happen in practice)Add additional validation for url in uss that includes guid, if file path exists but guid (the query parater in url) doesn't match, show an warning, though this should be rare in practice
