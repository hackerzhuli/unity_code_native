# Formatter
## Rules
We only format if there is no error node inside of a syntax tree in that source range. Otherwise, do nothing.

### Actual format range
If the request is to format a range, then we will first narrow the range so that it contains whole top level nodes(eg. top level statements/rulesets/comments). If it starts or ends at the middle of some top level node, then we should not include these nodes. Also note that our range must not start in the middle of a line(ie. there are other nodes in the same line before our first node), neither should it end in the middle of the line(ie. there are other nodes in the same line after our last node), then we should narrow the range again, until we find some clean range. Once the actual range is decided, we can go ahead and format that.

If the reuqest is to format the whole document, then the acutal range is just the whole document.

### Do nothing if there are error nodes
If there are error nodes in the actual range we are trying to format, then we will do nothing.

### How to acutally format uss source
We leverage a crate [malva](https://docs.rs/malva/latest/malva/index.html) that does just that(css formatting, which should work for uss too), send the source of the actual range to it, and let it format that.
