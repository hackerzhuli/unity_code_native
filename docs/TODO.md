TODO list:
- [ ] Add additional validation for url in uss that includes guid, if file path exists but guid (the query parater in url) doesn't match, show an warning, though this should be rare
- [ ] Improve hover on for import statement in uss, because now it doesn't have any hover.
- [ ] Improve hover for url path in uss, so that we show a clickable link for a file that does exist
- [ ] Improve hover for property to include syntax and examples and better description to make the hover doc for property better, also as for property value that is keyword, we should also include what that keyword does, hover docs is the most efficient way for user to get what he needed, so it must be good
- [ ] Add auto completion for properties, values and url/path(in which case we should detect what is actually on the filesytem to help user type faster, also we should add query string to includ guid file id and such to make the url complete)
- [ ] Add support for tss files, also note that "theme://default" (or something like that) is actually valid url in tss import statement, it is a special case. Should not create an error for that.

TODO for consideration:
- [ ] (We should do this later, it is complex because it involves Unity Editor) Add code action for url when url does include a guid, but file doesn't exist or guid doesn't match, then we can offer a code action to fix it, typically it is because user moved a file, which involves messaging Unity Editor, because we need to locate the asset, which could fail due to non existence of the guid or Unity Editor is busy