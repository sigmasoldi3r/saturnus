// This demonstrates the use of the native function `template()`,
// packed with the std, only in the Saturnus runtime!
use std;

let out = std::template("Hello {{variable}}!", '{ variable: "world" });

print("Out = " ++ out);
