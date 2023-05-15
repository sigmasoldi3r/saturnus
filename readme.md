# Saturnus

**Saturnus** is a programming language that aims to have a simplified mix of
[Rust programming language](https://www.rust-lang.org/) and [Lua](https://www.lua.org/).

The main target for Saturnus compiler is **Lua**, but multi-target compilation
will arrive in the future, so stay tuned if you like the language.

The original purpose of this language was to provide an easy-to-learn syntax,
and fast compilation times, to replace Lua scripts currently.

## Getting started

In order to compile your first file, you can check out the `/examples` folder,
and then, invoke the compiler from a terminal like:

```sh
./saturnus -i examples/hello_world_oop.saturn
```

(Or if you're using windows cmd)
```cmd
.\saturnus.exe -i examples\hello_world_oop.saturn
```

To get more help about the parameters, type:
```sh
./saturnus --help
```

### Where to get the binaries?

Currently the CD is disabled, however you can grab the latest [artifacts from
the nightly branch][nightly], **BUT!**

[nightly]: https://github.com/sigmasoldi3r/Saturnus/actions/workflows/build-artifacts.yml

**BUT...** beware that the artifacts will be surely outdated.

The safest way is just to clone this repository, and run:

```sh
cargo build --release
```

Then you will have the executable at `target/release/saturnus`. (You need the
[Rust tooling][rustup] to make that happen).

[rustup]: https://www.rust-lang.org/learn/get-started

## Why replace Lua?

I like many aspects of Lua, specially how fast and lightweight the VM is. But
original Lua syntax is nowadays a little bit old, and it needs some rework to
make the scripts less verbose and more easy to write.

Among other things, here are some key aspects that Saturnus changes:

- Function syntax is simpler, `fn` instead of `local function`.
- Lambdas are simpler yet familiar, Eg: `fn() 1 + 2 end`.
- More idiomatic class definitions: `class MyClass end` instead of [the classic one](https://www.lua.org/manual/2.4/node36.html).
- Decorators!
- A built-in prelude library for runtime type checks.
- Nice string interpolation.
- Terser loops.
- Built-in operator overloading.
- Custom operators.
- Some [RTTI](https://en.wikipedia.org/wiki/Run-time_type_information) (Which enables reflection).

## Some examples

Some little examples:

```rs
use println from "prelude";
use rti.Typed from "prelude";

class Greeter
  let who;

  // This will make the function panic if "who" is not a string!
  @Typed([rti.String])
  fn new(who)
    Greeter { who }
  end

  fn greet(self)
    return "Hello {self.who}!";
  end
end

// The classic OOP hello world:
let greeter = Greeter::new("Saturnus");
println(greeter.greet());
```

## Yet TODO:

- [ ] Implement a simple build system
- [ ] Match structure
- [ ] Add loops (for, while and "loop")
- [ ] Decorator code generation
- [ ] Operator overload
- [ ] Bitwise operators (This one is easy)
- [ ] Custom operator dispatch code generation
