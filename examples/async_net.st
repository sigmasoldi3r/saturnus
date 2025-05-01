// This example illustrates a basic usage of the async (default) net library.
use std::net;

let url = "https://jsonplaceholder.typicode.com/todos/1";

net::request('{ url, method: "get" }, { res =>
    let { body } = res;

    print("Body = " ++ body);  
});

print "Done";
