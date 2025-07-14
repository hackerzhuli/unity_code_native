## properties source data

Properties source data coverted to txt from Unity's docs. Each property is a single line.

There are 4 fields in them. the first is property name, which doesn't contain spaces.

Second is whether it is inherited, which is either Yes or No.

Third is whether it is animatable, which is either Fully animatable, Discrete or Non-animatable.

Fourth is the description, which is the last.

eg.

```
margin	No	Fully animatable	Shorthand for margin-top, margin-right, margin-bottom, margin-left
margin-bottom	No	Fully animatable	Space reserved for the bottom edge of the margin during the layout phase.
margin-left	No	Fully animatable	Space reserved for the left edge of the margin during the layout phase.
margin-right	No	Fully animatable	Space reserved for the right edge of the margin during the layout phase.
```