use url;
use log::LogLevel::Error as ErrorLevel;
#[cfg(feature="ssl")]
use openssl::ssl::{SslMethod, SslStream, SslConnectorBuilder};

use message::Message;
use frame::Frame;
use protocol::CloseCode;
use result::{Result, Error, Kind};
use util::{Token, Timeout};

#[cfg(feature="ssl")]
use util::TcpStream;


/// The core trait of this library.
/// Implementing this trait provides the business logic of the WebSocket application.
pub trait Handler {
    
    // general
    
    /// Called when a request to shutdown all connections has been received.
    #[inline]
    fn on_shutdown(&mut self) {
        debug!("Handler received WebSocket shutdown request.");
    }
    
    // WebSocket events
    
    /// Called when the WebSocket handshake is successful and the connection is open for sending
    /// and receiving messages.
    fn on_open(&mut self) -> Result<()> {
        if let Some(addr) = try!(shake.remote_addr()) {
            debug!("Connection with {} now open", addr);
        }
        Ok(())
    }
    
    /// Called on incoming messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        debug!("Received message {:?}", msg);
        Ok(())
    }
    
    /// Called any time this endpoint receives a close control frame.
    /// This may be because the other endpoint is initiating a closing handshake,
    /// or it may be the other endpoint confirming the handshake initiated by this endpoint.
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        debug!("Connection closing due to ({:?}) {}", code, reason);
    }
    
    /// Called when an error occurs on the WebSocket.
    fn on_error(&mut self, err: Error) {
        // Ignore connection reset errors by default, but allow library clients to see them by
        // overriding this method if they want
        if let Kind::Io(ref err) = err.kind {
            if let Some(104) = err.raw_os_error() {
                return
            }
        }
        
        error!("{:?}", err);
        if !log_enabled!(ErrorLevel) {
            println!("Encountered an error: {}\nEnable a logger to see more information.", err);
        }
    }
    
    
    // timeout events
    
    /// Called when a timeout is triggered.
    ///
    /// This method will be called when the eventloop encounters a timeout on the specified
    /// token. To schedule a timeout with your specific token use the `Sender::timeout` method.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// const GRATI: Token = Token(1);
    ///
    /// ... Handler
    ///
    /// fn on_open(&mut self, _: Handshake) -> Result<()> {
    ///     // schedule a timeout to send a gratuitous pong every 5 seconds
    ///     self.ws.timeout(5_000, GRATI)
    /// }
    ///
    /// fn on_timeout(&mut self, event: Token) -> Result<()> {
    ///     if event == GRATI {
    ///         // send gratuitous pong
    ///         try!(self.ws.pong(vec![]))
    ///         // reschedule the timeout
    ///         self.ws.timeout(5_000, GRATI)
    ///     } else {
    ///         Err(Error::new(ErrorKind::Internal, "Invalid timeout token encountered!"))
    ///     }
    /// }
    /// ```
    #[inline]
    fn on_timeout(&mut self, event: Token) -> Result<()> {
        debug!("Handler received timeout token: {:?}", event);
        Ok(())
    }
    

    #[inline]
    fn on_new_timeout(&mut self, _: Token, _: Timeout) -> Result<()> {
        // default implementation discards the timeout handle
        Ok(())
    }
    
}

impl<F> Handler for F
    where F: Fn(Message) -> Result<()>
{
    fn on_message(&mut self, msg: Message) -> Result<()> {
        self(msg)
    }
}

mod test {
    #![allow(unused_imports, unused_variables, dead_code)]
    use super::*;
    use url;
    use mio;
    use protocol::CloseCode;
    use frame;
    use message;
    use result::Result;
    
    #[derive(Debug, Eq, PartialEq)]
    struct M;
    impl Handler for M {
        fn on_message(&mut self, _: message::Message) -> Result<()> {
            Ok(println!("test"))
        }
        
    }
    
    #[test]
    fn handler() {
        struct H;
        
        impl Handler for H {
            
            fn on_open(&mut self) -> Result<()> {
                assert!(shake.request.key().is_ok());
                assert!(shake.response.key().is_ok());
                Ok(())
            }
            
            fn on_message(&mut self, msg: message::Message) -> Result<()> {
                Ok(assert_eq!(msg, message::Message::Text(String::from("testme"))))
            }
            
            fn on_close(&mut self, code: CloseCode, _: &str) {
                assert_eq!(code, CloseCode::Normal)
            }
            
        }
        
        let mut h = H;
        let url = url::Url::parse("wss://127.0.0.1:3012").unwrap();
//        let res = Response::from_request(&req).unwrap();
        h.on_open().unwrap();
        h.on_message(message::Message::Text("testme".to_owned())).unwrap();
        h.on_close(CloseCode::Normal, "");
    }
    
    #[test]
    fn closure_handler() {
        let mut close = |msg| {
            assert_eq!(msg, message::Message::Binary(vec![1, 2, 3]));
            Ok(())
        };
        
        close.on_message(message::Message::Binary(vec![1, 2, 3])).unwrap();
    }
}
