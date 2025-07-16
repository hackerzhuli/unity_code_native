# TSS
tss is actually the same language as uss. Except that it has a different extension: `.tss`.

Also, in a tss file, we can only import other tss files. The same is true for uss, in a uss file, you can only import other uss files. This may or may not be a hard rule, it may be just a convention(that can only import the same type of file).

## special import url

tss support a special import url, that is invalid in other context:

``` css
@import url("unity-theme://default");
```

This (the url string itself) is valid and we should not show any diagnostics for it.

So, if the url uses this scheme `unity-theme://`, we just don't validate it, always assume it is valid.

We need to treat this type of url in a special way in our diagnostics.