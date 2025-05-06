// This example illustrates a basic usage of the async net library.
use net;
use JSON;

let url = "https://jsonplaceholder.typicode.com/todos/1";

print "Starting request";

net::request('{ url, method: "get" }, { res =>
    let { body } = res;
    let ( ok, err ) = JSON::parse(body);
    if err != () {
        print("Invalid body: '" ++ body ++ "' caused by " ++ err);
        return ();
    }
    let { id, title } = ok;
    print("Entry [" ++ id ++ "] = '" ++ title ++ "'");
});

print "Done, response should arrive soon";
