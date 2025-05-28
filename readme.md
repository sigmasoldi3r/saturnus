# Saturnus

A general-purpose, high level programming language that aims to be
a direct replacement of _Lua_ scripts.

> [!WARNING]  
> Currently, saturnus is under a heavy re-write from scratch, so the
> repository is not ready yet, although functional.
>
> Main changes involve shifting from a PEG parsing crate to a LR one,
> among other changes like splitting modules, and a "rebranding" of the
> toolchain. Among others, Jaanus is now "Titan", which currently does
> not work.
>
> Also the rewrite comes with new benefits, such as compilation time
> optimizations, and new expressions that were deemed too hard to parse
> previously.

## Test it out

Clone the repository, then run inside:

```sh
cd examples
cargo run --bin titan -- compile
```

You should see something akin to:

```
titan warning: Dependency resolving is being worked on.
titan info: Compiling objects...
  Compiling src\example.st...
    Compiled src\example.st
  Compiling src\main.st...>                         ] 1/2                                                                                                                         
    Compiled src\main.st
 [=================================================>] 2/2                                                                                                                         
titan info: Linking objects...
      Linked target/objects\example.lua
      Linked target/objects\main.lua                                                                                                                                              
 [=================================================>] 2/2                                                                                                                         
titan info: Building Test
titan info: Done
```

And a file called `Test.lua` in the `target/` folder.

## How does it look?

> [!TIP]
> Take a look at the `/examples` folder in this repository!

A simple, yet classy (No pun intended) hello world:

_Extracted from [examples/hello_world.st example file](https://github.com/sigmasoldi3r/saturnus/blob/main/examples/hello_world.st)_

```js
// Simple hello world with OOP
class Greeter {
    // Self is available within the class context.
    // References this class.
    static fn new(name <- "Stranger") = Self.'{ name };
    fn greet() = print("Hello " ++ self.name ++ "!");
    // You could invoke new as:
    static fn default() = Self::new "unnamed";
}

let world = Greeter::new "World";

world.greet();

```

> [!CAUTION]
> Most notorious changes from the old syntax include:
>
> - Dynamic dispatch is now `.`, previously `->`.
> - Static dispatch is back to `::`.
> - Lambda syntax is now block oriented: `{ a, b => a + b }`, unlike the old one (`() => {}`).

_More syntax will be added to the docs as the project develops._

## Roadmap

> [!NOTE]
> Is more or less ordered by priority.

- **[ONGOING]**: Create a working toolset to compile and manage projects
- **[ONGOING]**: Fix basic missing features (Unary ops).
- Parse process and compile macro declarations `macro! foo {...}`
- Parse and compile macro calls `some!("macro")`
- Parse and compile runtime decorators `@some("thing") class Foo {...}`
- Parse and compile macro decorators `@some!("thing") class Foo {...}`
- Add coroutine sugar syntax `{ a, b ~> "This is a coroutine" }`
- Implement a robust runtime that can handle async programming and web services.
