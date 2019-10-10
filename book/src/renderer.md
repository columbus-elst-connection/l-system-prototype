# Renderer

We're going to use turtle graphics to render our l-system. Turtle graphics is an approach that uses an API that's conceptually based on a _pen_ that is controlled by giving it a sequence of instructions. For example, `go forward 2 steps, turn right, go forward 1 step, turn right, go forward 2 steps`. Of course a pen is an inanimate object and they're terrible at following instructions, so traditionally a turtle is in charge of the pen, though in our case it will be a crab instead. The crab will faithfully carry our each of the instructions in order to create our image.

## Renderer API

To start with, our renderer trait looks like this:

```rust
pub trait Renderer {
    /// Called at the very beginning of the entrypoint to our program so that the renderer can create a window if needed
    fn global_init() where Self: Sized {}

    /// Creates a new instance of the renderer with the given configuration
    fn new(renderer_config: RendererConfig) -> Self;

    /// Render the given instruction. This will get called repeatedly in order to create our images
    fn render(&mut self, instruction: RendererInstruction);

    /// Signals that the final instruction has been given and we can clean up and do any finalization that's required
    fn finish(&mut self) {}
}
```

The `global_init` function is the very first thing that's called when our application starts up. We'll use this function to create a window that we can draw into. Windowing and event handling apis can be relatively complex, and every platform has its own quirks. It's common for the operating system to require that window creation is done very early in the application lifecycle, so we'll always call this function right away in our application's main function.

After `global_init` is done, the application will focus on parsing it's arguments and the l-system. Parsing the l-system file successfully will result in a `RendererConfig` struct, which will be passed to `Renderer::new`. It's here that we can initialize things like the background color, the starting line width, and other such things. After this is done, we should be ready to execute render instructions to create our image.

The `render` function is where the magic happens. `RendererInstruction` is an enum with a variant for each instruction. Each time it's invoked, we'll use pattern matching to determine what to do. This would look something like the following:

```rust
fn render(&mut self, instruction: RendererInstruction) {
    match instruction {
        RendererInstruction::Forward => self.forward(),
        RendererInstruction::RotateLeft => self.rotate_left(),
        RendererInstruction::RotateRight => self.rotate_right(),
        RendererInstruction::NoOp => { /* no-op */ },
        /* .... remainder omitted for brevity */
    }
}
```

Since this pattern matching isn't particularly interesting, we'll update our `Renderer` trait to provide the implementation on the trait itself, which will just delegate to named functions for each instruction. We'll add a function to the `Renderer` trait for each instruction so that the code will be more readable. So instead of implementing `render` directly, we'll just implemet each of the specific functions like `forward`, `rotate_left`, `rotate_right`, `push`, `pop`, etc. So now our `Renderer` trait will look more like this:

```rust
pub trait Renderer {
    fn global_init() where Self: Sized {}

    fn new(renderer_config: RendererConfig) -> Self;

    fn render(&mut self, instruction: RendererInstruction) {
        match instruction {
            RendererInstruction::Push => self.push(),
            RendererInstruction::Pop => self.pop(),
            RendererInstruction::Forward => self.forward(),
            RendererInstruction::NoOp => { /* no-op */ },
            /* .... remainder omitted for brevity */
        }
    }

    fn forward(&mut self) {}
    fn push(&mut self) {}
    fn pop(&mut self) {}
    /* .... remainder of functions omitted for brevity */

    fn finish(&mut self) {}
}
```

### RendererConfig

Our renderer instructions are very simple and abstract. For example, to draw a straight line we would use the `Forward` instruction. The `Forward` instruction doesn't itself say _how far_ to move. That is instead specified in a separate struct as the `step` field. Whenever we execute the `Forward` instruction, `step` tells us how far to move. Why would these be separate? The answer is that the language we use to describe the L-system has no way to maintain state. We can provide instructions to _modify_ the state that we store, though. For example, we could have an instruction to multiply `step` by some global `step_multiplier`, and another instruction to divide it. This allows us to change the scale of our drawings, the line width, and things like that. The Renderer implementation is responsible for maintaining the state of the current `step`, `position`, `heading`, and `line_width`, but the initial values are specified in the `RendererConfig`. The `RendererConfig` also specifies the `pen_color`, `background_color`, and other things that will not be mutable in our implementation.
