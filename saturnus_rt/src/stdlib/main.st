/// ## Functional application operator
///
/// This operator applies at runtime the expression on the left, to
/// the target function on the right.
///
/// Example:
/// ```st
/// "Hello" |> print;
/// // Equivalent to print("Hello");
/// ```
pub fn `|>`(arg: unknown, target: Fn) = target(arg);

/// ## Range operator
///
/// Creates an iterator that goes "from" until "to",
/// see Range class.
pub fn `..`(from: int, to: int) = Range::new(from, to);

/// ## Number range.
///
/// Yields an integer with each step. You can adjust the
/// step amount by using the `.step(amount)` method.
pub class Range {
    let step = 1;
    let from: int;
    let to: int;

    static fn new(from: int, to: int) = Range.'{ from, to };

    fn step(amount: int) {
        self.step = amount;
        return self;
    }

    fn `()`() {
        let i = self.from;
        let step = self.step;
        let max = self.to;
        return {
            i += step;
            if i > max {
                return ();
            } else {
                return i;
            }
        };
    }
}

/// ## Simple functional mapper
///
/// Creates a mapper function upon invocation. The first argument is the
/// mapper function `a -> b`, example: `[1, 2, 3] |> map.{ it * 2 }`.
pub fn map(mapper: Fn) = {
    let out = [];
    for (k, v) in ipairs(it) {
        out[k] = mapper(v, k);
    }
    return out;
};
