// Simple hello world with OOP
class Greeter {
    static fn new(name) = Greeter.'{ name };
    fn greet() = print("Hello " ++ self.name ++ "!");
}

let world = Greeter::new "World";

world.greet();
