// This example illustrates the use of the net library to host microservices
use net;
use JSON;

let port = 8080;

let server = net::server('{ port }).start.{
    let co = coroutine::create.{ _ =>
        print("Incoming request");
        let inc = it.read();
        print("inc = ", inc);
    };
    loop {
        let [ok, msg] = [coroutine::resume(co)];
        print("Status: ", ok, " msg: ", msg);
        if not ok {
            break;
        }
    }
    print("Stream ended");
};

print("Listening on port " ++ port);
