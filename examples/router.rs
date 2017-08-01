/// WebSocket server using trait objects to route
/// to an infinitely extensible number of handlers
extern crate ws;
extern crate env_logger;

// A WebSocket handler that routes connections to different boxed handlers by resource
struct Router {
    sender: ws::Sender,
    inner: Box<ws::Handler>,
}

impl ws::Handler for Router {
   
    // Pass through any other methods that should be delegated to the child.
    //
    // You could probably use a macro for this if you have many different
    // routers or were building some sort of routing framework.

    fn on_shutdown(&mut self) {
        self.inner.on_shutdown()
    }

    fn on_open(&mut self) -> ws::Result<()> {
        self.inner.on_open(shake)
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.inner.on_message(msg)
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        self.inner.on_close(code, reason)
    }

    fn on_error(&mut self, err: ws::Error) {
        self.inner.on_error(err);
    }
}

// This handler returns a 404 response to all handshake requests
struct NotFound;

impl ws::Handler for NotFound {


}


// This handler simply echoes all messages back to the client
struct Echo {
    ws: ws::Sender,
}

impl ws::Handler for Echo {

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        println!("Echo handler received a message: {}", msg);
        self.ws.send(msg)
    }

}

// This handler sends some data to the client and then terminates the connection on the first
// message received, presumably confirming receipt of the data
struct Data {
    ws: ws::Sender,
    data: Vec<&'static str>,
}

impl ws::Handler for Data {
    fn on_open(&mut self) -> ws::Result<()> {
        for msg in self.data.iter() {
            try!(self.ws.send(*msg))
        }
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        println!("Data handler received a message: {}", msg);
        println!("Data handler going down.");
        self.ws.close(ws::CloseCode::Normal)
    }
}


fn main () {

    env_logger::init().unwrap();

    // Listen on an address and call the closure for each connection
    if let Err(error) = ws::listen("127.0.0.1:3012", |out| {

        // Use our router as the handler to route the new connection
        Router {
            sender: out,
            // Default to returning a 404 when the route doesn't match.
            // You could default to any handler here.
            inner: Box::new(NotFound),
        }

    }) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to {:?}", error);
    }
}
