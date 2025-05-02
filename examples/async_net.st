// This example illustrates a basic usage of the async (default) net library.
use net;

let url = "https://jsonplaceholder.typicode.com/todos/1";

print "Starting request";

net::request('{ url, method: "get" }, { res =>
    let { body } = res;

    print("Body = " ++ tostring(body));  
});

print "Done";
