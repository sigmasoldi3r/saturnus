# Saturnus

A generar-purpose, high level programming language that aims to be
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
