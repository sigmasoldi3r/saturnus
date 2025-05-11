# How to test this module

Compile the project:

```sh
cargo build
```

And then run the script like:

```sh
cargo run --bin saturnc -- run -l=target/debug/example_lib.dll -i example_lib/test.st
cargo run --bin saturnc -- run -l=target/debug/example_lib.os -i example_lib/test.st
```

Note: The library extension depends on the platform you're compiling this into.
