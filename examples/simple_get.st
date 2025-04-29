// This example illustrates a basic usage of the net library.
use std::net;

let { body, status } = net::get "https://jsonplaceholder.typicode.com/todos/1";

print("Status = " ++ status);
print("Body = " ++ body);
