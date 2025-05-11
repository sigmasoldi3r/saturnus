// This example illustrates the use of the net library to host microservices
use net;
use JSON;

let port = 8080;

let server = net::server('{ port }).start.{
    let co = coroutine::create.{ _ =>
        print("Incoming request");
        let inc = it.read();
        print("inc = ", inc);
        it.write("HTTP/1.1 200 OK
Content-Length: 12

Hello world!");
    };
    loop {
        let [ok, msg] = [coroutine::resume(co)];
        if not ok {
            print("Status: ", ok, " msg: ", msg);
            break;
        }
    }
    print("Stream ended");
};

print("Listening on port " ++ port);
