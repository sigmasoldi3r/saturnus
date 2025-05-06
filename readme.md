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
cargo run --bin saturnc -- run -i examples/simple_get.st
```

You should see something akin to:

```
Starting request
Done, response should arrive soon
Entry [1] = 'delectus aut autem'
```

## How does it look?

> [!TIP]
> Take a look at the `/examples` folder in this repository!

A simple, yet classy (No pun intended) hello world:

```js
// Simple hello world with OOP
class Greeter {
    static fn new(name) = Greeter.'{ name };
    fn greet() = print("Hello " ++ self.name ++ "!");
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

- **[ONGOING]**: Implement a robust runtime that can handle async programming and web services.
- Add coroutine sugar syntax `{ a, b ~> "This is a coroutine" }`
- Fix basic missing features (Unary ops).
- Parse process and compile macro declarations `macro! foo {...}`
- Parse and compile macro calls `some!("macro")`
- Parse and compile runtime decorators `@some("thing") class Foo {...}`
- Parse and compile macro decorators `@some!("thing") class Foo {...}`
