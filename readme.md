# Saturnus

![Saturnus provisional logo](saturnus_realist.png)

**Saturnus** is a programming language that aims to have a simplified mix of
[Rust programming language](https://www.rust-lang.org/) and [Lua](https://www.lua.org/).

The main target for Saturnus compiler is **Lua**, but multi-target compilation
will arrive in the future, so stay tuned if you like the language.

The original purpose of this language was to provide an easy-to-learn syntax,
and fast compilation times, to replace Lua scripts currently.

Wanna see how it looks? [Jump to Language Basics](#language-basics)!

## Getting started

The easiest way is to [install the tools](installation.md), and then run:

```sh
cd examples
janus build
```

To get more information on how to run the saturnus compiler or the Janus tool,
use the `--help` (Or `-h`) flag, like:

```sh
janus --help
saturnus --help
```

### Introducing Janus!

Now _Saturnus_ has a simple build system, meet Janus: The official _Saturnus_
programming language build toolkit.

To build a Janus workspace, you need to create a file named `Janus.toml`, here
is an example of a working project file:

```toml
[project]
name = "Helo Saturnus!"
author = "You & Me"
version = "1.0.0"

[build]
# Use default build flags
```

Tweak the fields according to your needs. Now by default, the only compile mode
available is the module-less, compile-in-place mode. This will produce `.lua`
files next to your `.saturn` files.

A little bit of Janus todo would be:

- [ ] Define more build modes
- [ ] Add flags to target other platforms
- [ ] Create different module system strategies
- [ ] File I/O directories customization
- [ ] Compiler flags

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

## Language Basics

> **Note**
>
> Some structures will be pretty similar to Rust, so this assumes you
> have some knowledge about OOP languages, and you're familiar with the Lua
> runtime.

> **Warning**
>
> An important remark about the syntax: Unlike Lua, here each
> statement that is **not** a block, it must end with a **semicolon** (`;`).

Declarations and function calls:

```rs
// Variables are easy stuff in Saturnus:
let a = "be something";
// period.

// Addition is more idiomatic than in plain Lua syntax:
count += 1;
strcat ++= "foo'd";

// Basic operators:
let a = b + c;
let a = b - c;
let a = b * c;
let a = b / c;
let a = b ** c; // Power op
let a = b % c; // Modulo
let a = b ++ c; // String concatenation
let rng = 1..10; // Range iterator build

// Collection types:

// Like in Javascript
let this_is_a_table = { a, b: "foo", c: 30-30 };
let this_is_a_vector = [1, 2, 3, (), "potato"];
// New structure, The tuple!
let this_is_a_tuple = ("foo", 10, "bar");
let foo = this_is_a_tuple._0;
let ten = this_is_a_tuple._1;
let bar = this_is_a_tuple._2;

// Array access can be weird compared to others (It has an extra dot):
let foo = bar.[key].value;
```

Now, function calls can be something more complex.

Imagine that you have static functions (aka do not belong to an object):

```rs
some_function(1, 2, 3);
let b = fetch("some url");
// Note that like in lua, you can pass [], {} and "" without parentheses:
let module = require! "Mods";
let bar = joining! [1, 2, 3];
let c = Foo { bar };
// etc
```

Those will be dispatched statically, and everyone will be safe & sound. But in
real world, you will have functions inside raw objects or class instances, here
is where things get tricky:

```rs
// Imagine a table with a static function like math's max()
// You have to use the static dispatch mode:
let max = math::max(1, 2);
// This is crucial, otherwise things will berak.

// BUT!
// If the object is an instance of a class, or an object that needs to access
// self's context, like, for example a "person" instance, you will have to use
// the dot (aka dynamic dispatch mode):
let name = person.get_name();
// As long as you remember that, you'll be safe & sound.
// Another side note, that obviously does not apply to fields of an object:
let things = that_do.not.care.about_dispatch;
// Because fields do not have the notion of "dispatching" (They're not functions
// at all!).
```

The loops:

In _Saturnus_ you can loop with four different structures: `while`, `while let`,
`for` and `loop` (See comments):

```rs
// The basic loop!
// Will repeat as long as the expression between "while" and "do" words is
// true-like (Can evaluate to "true").
while something() {
  print("Something is true!");
}

// This one is a sugar syntax introduced by Saturnus!
// Imagine you want to loop as long as you have a result that is not null, you
// could use iterators, reserve your own local variables and such, but we
// have a more idiomatic syntax sugar for you:
while let some = thing() {
  // The "some" variable is only visible within the loop, and you know that
  // will be a true-ish value (Like 1, true or something not null).
  print("Some is " ++ some);
}

// Numeric for? Easy sugar:
for i in 1..10 {
  print("i = " ++ i)
}

// Now, the classical foreach:
for entry in entries() {
  print(entry._0 ++ " = " ++ entry._1);
}
// Note: This is a raw iterator loop, and cannot be used in place of an
// iterator! This means that is no replacement for pairs function (and also
// it does NOT work well with it...)
// This assumes that what you have between "in" and "do" returns an iterator
// of a single entry value.
// To transform collections to iterators, you will need some prelude functions.

// And the final, the simplest and the dumbest:
loop {
  print("I'm looping forever...");
  if should_exit() {
    print("Or I am?");
    return true;
  }
}
// Note: Has no exit condition, you will have to either "break" or "return"!
```

That covers what _Saturnus_ can offer for now, in terms of looping.

Now, this follows conditions! We have `if`, `if else` and `else` at the moment:

```rs
// If statements are pretty close to Lua, as you can witness here:
if something() {
  print("Something was true!");
}

if a {
  print("A");
} else {
  print("Not A...");
}

// The only difference is that "else if" is separated with a space instead
// of being the word elseif.
if a {
  print("A");
} else if b {
  print("B");
} else {
  print("woops");
}
```

Functions!

Functions are declared like Lua ones, using `fn` keyword, but with a catch:
They are **always** local, never global (That is forbidden by design).

```rs
// Fair enough:
fn some_func(a, b) {
  return a + b;
}

// Oh, you can also have anonymous functions by the way!
let anon = fn(a, b) {
  return a + b;
};

// And if an anonymous function ONLY has one expression inside (Without ";"),
// that expression is an implicit return statement:
collections::reduce([1, 2, 3], fn(a, b) { a + b });
// Pretty cool
```

Time for some object oriented programming! Yes, _Saturnus_ has classes, of
course, but we forbid inheritance by design, which does not eliminate
at all polymorphism (see below).

```rs
class Person {
  // Fields (which are optional btw), are declared as variables:
  let name = "unnamed";

  // Methods, like normal functions, but remember that if the first (and only
  // the first) argument is "self", it will be a dynamic method, and if that is
  // absent, it will be compiled as a static method:
  fn get_name(self) {
    return self.name;
  }

  // Example of an static method, where the usage is shown later:
  fn greet(person) {
    print("Greetings " ++ person.name ++ "!");
  }
}

// Here you'll clearly see the difference:
let person = Person { name: "Mr. Foo" };
let name = person.get_name(); // Dynamic dispatch
Person::greet(person); // Static method dispatch!
```

Polymorphism example, altough if you're familiar with the term
["Duck Typing"][duck-type], you won't need this example:

[duck-type]: https://en.wikipedia.org/wiki/Duck_typing

```rs
class Foo {
  fn action(self) {
    return "At a distance"; // Einstein would complain...
  }
}

class Bar {
  fn action(self) {
    return "Quantum entanglement";
  }
}

class FooBarConsumer {
  fn consume(self, actor) {
    return "The action was: " ++ actor.action();
  }
}
let foo = Foo {};
let bar = Bar {};

let consumer = FooBarConsumer {};
print(consumer.consume(foo));
print(consumer.consume(bar));
```

## Why replace Lua?

I like many aspects of Lua, specially how fast and lightweight the VM is. But
original Lua syntax is nowadays a little bit old, and it needs some rework to
make the scripts less verbose and more easy to write.

Aside of the [Language Basics](#language-basics) section, there are other key
aspects of the language:

- Decorators!
- A built-in prelude library for runtime type checks.
- ~~Nice string interpolation.~~ (Maybe not?)
- Terser loops.
- Built-in operator overloading.
- Custom operators.
- Some [RTTI](https://en.wikipedia.org/wiki/Run-time_type_information) (Which enables reflection).

## The MVP release to-do list:

- [x] ~~Implement a simple build system~~ **Janus** comes to the rescue!
- [ ] Ennumeration structures
- [ ] Match structure
- [x] ~~Add loops (for, while and "loop")~~
- [ ] Decorator code generation
- [x] ~~Operator overload~~
- [ ] Bitwise operators (This one is easy)
- [ ] Custom operator dispatch code generation
- [ ] Destructuring assignment
