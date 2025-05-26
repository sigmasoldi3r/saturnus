// Simple hello world with OOP
class Greeter {
    // Self is available within the class context.
    // References this class.
    static fn new(name <- "Stranger") = Self.'{ name };
    fn greet() = print("Hello \"" ++ self.name ++ "\"!");
    // You could invoke new as:
    static fn default() = Self::new "unnamed";
}

let world = Greeter::new "World";
world.greet();

// Default initializers:
Greeter::new().greet();
