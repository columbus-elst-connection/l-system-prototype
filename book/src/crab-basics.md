# Turtle (Crab) Rendering

Ok, so we have a Crab that's brandishing a pen, so how do we actually work with this thing?
The `crab` API is pretty simple and straightforward. It's pretty easy to implement each individual rendering instruction. The complexity of how multiple instructions are combined to draw interesting shapes will be left to the L-system.



## Simple instructions

- `Forward`: Moves the crab forward by the `step`, drawing a straight line along the way.
- `RotateLeft`, `RotateRight`: Rotates the crab in place. "Right" means clockwise, and "left" means counter-clockwise
