#![allow(deprecated)]
#![deny(
missing_copy_implementations,
trivial_casts, trivial_numeric_casts,
unstable_features,
unused_import_braces)]

extern crate httparse;
extern crate mio;
extern crate sha1;
extern crate rand;
extern crate url;
extern crate slab;
extern crate bytes;
extern crate byteorder;
#[macro_use]
extern crate log;

mod result;
mod connection;
mod handler;
mod factory;

mod message;

mod protocol;
mod communication;
mod io;
mod stream;


pub mod util;

pub use factory::Factory;
pub use handler::Handler;

pub use result::{Result, Error};
pub use result::Kind as ErrorKind;
pub use message::Message;
pub use communication::Sender;
pub use protocol::{CloseCode, OpCode};


use std::fmt;
use std::default::Default;
use std::net::{SocketAddr, ToSocketAddrs};
use std::borrow::Borrow;

use mio::Poll;

/// A utility function for setting up a WebSocket server.
///
/// # Safety
///
/// This function blocks until the event loop finishes running. Avoid calling this method within
/// another WebSocket handler.
///
/// # Examples
///
/// ```no_run
/// use ws::listen;
///
/// listen("127.0.0.1:3012", |out| {
///     move |msg| {
///        out.send(msg)
///    }
/// }).unwrap()
/// ```
///
pub fn listen<A, F, H>(addr: A, factory: F) -> Result<()>
                       where
                           A: ToSocketAddrs + fmt::Debug,
                           F: FnMut(Sender) -> H,
                           H: Handler,
{
    let ws = WebSocket::new(factory)?;
    ws.listen(addr)?;
    Ok(())
}

/// A utility function for setting up a WebSocket client.
///
/// # Safety
///
/// This function blocks until the event loop finishes running. Avoid calling this method within
/// another WebSocket handler. If you need to establish a connection from inside of a handler,
/// use the `connect` method on the Sender.
///
/// # Examples
///
/// ```no_run
/// use ws::{connect, CloseCode};
///
/// connect("ws://127.0.0.1:3012", |out| {
///     out.send("Hello WebSocket").unwrap();
///
///     move |msg| {
///         println!("Got message: {}", msg);
///         out.close(CloseCode::Normal)
///     }
/// }).unwrap()
/// ```
///
pub fn connect<F, H>(url: String, factory: F) -> Result<()>
                        where
                            F: FnMut(Sender) -> H,
                            H: Handler
{
    let mut ws = WebSocket::new(factory)?;
    //    let parsed =
    //        url::Url::parse(url.borrow())
    //            .map_err(|err| Error::new(ErrorKind::Internal, format!("Unable to parse {} as url due to {:?}", url.borrow(), err)))?;
    //    trace!("----{:?}---", url.borrow());
    ws.connect(url)?;
    ws.run()?;
    Ok(())
}

/// WebSocket settings
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// The maximum number of connections that this WebSocket will support.
    /// The default setting is low and should be increased when expecting more
    /// connections because this is a hard limit and no new connections beyond
    /// this limit can be made until an old connection is dropped.
    /// Default: 100
    pub max_connections: usize,
    /// The number of events anticipated per connection. The event loop queue size will
    /// be `queue_size` * `max_connections`. In order to avoid an overflow error,
    /// `queue_size` * `max_connections` must be less than or equal to `usize::max_value()`.
    /// The queue is shared between connections, which means that a connection may schedule
    /// more events than `queue_size` provided that another connection is using less than
    /// `queue_size`. However, if the queue is maxed out a Queue error will occur.
    /// Default: 5
    pub queue_size: usize,
    /// Whether to panic when unable to establish a new TCP connection.
    /// Default: false
    pub panic_on_new_connection: bool,
    /// Whether to panic when a shutdown of the WebSocket is requested.
    /// Default: false
    pub panic_on_shutdown: bool,
    /// The maximum number of fragments the connection can handle without reallocating.
    /// Default: 10
    pub fragments_capacity: usize,
    /// Whether to reallocate when `fragments_capacity` is reached. If this is false,
    /// a Capacity error will be triggered instead.
    /// Default: true
    pub fragments_grow: bool,
    /// The maximum length of outgoing frames. Messages longer than this will be fragmented.
    /// Default: 65,535
    pub fragment_size: usize,
    /// The size of the incoming buffer. A larger buffer uses more memory but will allow for fewer
    /// reallocations.
    /// Default: 2048
    pub in_buffer_capacity: usize,
    /// Whether to reallocate the incoming buffer when `in_buffer_capacity` is reached. If this is
    /// false, a Capacity error will be triggered instead.
    /// Default: true
    pub in_buffer_grow: bool,
    /// The size of the outgoing buffer. A larger buffer uses more memory but will allow for fewer
    /// reallocations.
    /// Default: 2048
    pub out_buffer_capacity: usize,
    /// Whether to reallocate the incoming buffer when `out_buffer_capacity` is reached. If this is
    /// false, a Capacity error will be triggered instead.
    /// Default: true
    pub out_buffer_grow: bool,
    /// Whether to panic when an Internal error is encountered. Internal errors should generally
    /// not occur, so this setting defaults to true as a debug measure, whereas production
    /// applications should consider setting it to false.
    /// Default: true
    pub panic_on_internal: bool,
    /// Whether to panic when a Capacity error is encountered.
    /// Default: false
    pub panic_on_capacity: bool,
    /// Whether to panic when a Protocol error is encountered.
    /// Default: false
    pub panic_on_protocol: bool,
    /// Whether to panic when an Encoding error is encountered.
    /// Default: false
    pub panic_on_encoding: bool,
    /// Whether to panic when a Queue error is encountered.
    /// Default: false
    pub panic_on_queue: bool,
    /// Whether to panic when an Io error is encountered.
    /// Default: false
    pub panic_on_io: bool,
    /// Whether to panic when a Timer error is encountered.
    /// Default: false
    pub panic_on_timeout: bool,
    /// Whether to shutdown the eventloop when an interrupt is received.
    /// Default: true
    pub shutdown_on_interrupt: bool,
    /// Disables Nagle's algorithm.
    /// Usually tcp socket tries to accumulate packets to send them all together (every 200ms).
    /// When enabled socket will try to send packet as fast as possible.
    ///
    /// Default: false
    pub tcp_nodelay: bool
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            max_connections: 100,
            queue_size: 5,
            panic_on_new_connection: false,
            panic_on_shutdown: false,
            fragments_capacity: 10,
            fragments_grow: true,
            fragment_size: u16::max_value() as usize,
            in_buffer_capacity: 2048,
            in_buffer_grow: true,
            out_buffer_capacity: 2048,
            out_buffer_grow: true,
            panic_on_internal: true,
            panic_on_capacity: false,
            panic_on_protocol: false,
            panic_on_encoding: false,
            panic_on_queue: false,
            panic_on_io: false,
            panic_on_timeout: false,
            shutdown_on_interrupt: true,
            tcp_nodelay: false
        }
    }
}


/// The WebSocket struct. A WebSocket can support multiple incoming and outgoing connections.
pub struct WebSocket<F>
    where F: Factory
{
    poll: Poll,
    handler: io::Handler<F>,
}

impl<F> WebSocket<F>
    where F: Factory
{
    /// Create a new WebSocket using the given Factory to create handlers.
    pub fn new(factory: F) -> Result<WebSocket<F>> {
        Builder::new().build(factory)
    }
    
    /// Consume the WebSocket and bind to the specified address.
    /// If the `addr_spec` yields multiple addresses this will return after the
    /// first successful bind. `local_addr` can be called to determine which
    /// address it ended up binding to.
    /// After the server is succesfully bound you should start it using `run`.
    pub fn bind<A>(mut self, addr_spec: A) -> Result<WebSocket<F>>
                   where A: ToSocketAddrs
    {
        let mut last_error = Error::new(ErrorKind::Internal, "No address given");
        
        for addr in addr_spec.to_socket_addrs()? {
            if let Err(e) = self.handler.listen(&mut self.poll, &addr) {
                error!("Unable to listen on {}", addr);
                last_error = e;
            } else {
                let actual_addr = self.handler.local_addr().unwrap_or(addr);
                info!("Listening for new connections on {}.", actual_addr);
                return Ok(self);
            }
        }
        
        Err(last_error)
    }
    
    /// Consume the WebSocket and listen for new connections on the specified address.
    ///
    /// # Safety
    ///
    /// This method will block until the event loop finishes running.
    pub fn listen<A>(self, addr_spec: A) -> Result<WebSocket<F>>
                     where A: ToSocketAddrs
    {
        self.bind(addr_spec).and_then(|server| server.run())
    }
    
    /// Queue an outgoing connection on this WebSocket. This method may be called multiple times,
    /// but the actual connections will not be established until `run` is called.
//        pub fn connect(&mut self, url: url::Url) -> Result<&mut WebSocket<F>> {
//            let sender = self.handler.sender();
//            info!("Queuing connection to {}", url);
//            sender.connect(url)?;
//            Ok(self)
//        }
    
    pub fn connect(&mut self, addr_spec: String) -> Result<&mut WebSocket<F>>
    {
        let sender = self.handler.sender();
        info!("Queuing connection to {}", addr_spec);
        sender.connect(addr_spec)?;
        Ok(self)
    }
    
    /// Run the WebSocket. This will run the encapsulated event loop blocking the calling thread until
    /// the WebSocket is shutdown.
    pub fn run(mut self) -> Result<WebSocket<F>> {
        self.handler.run(&mut self.poll)?;
        Ok(self)
    }
    
    /// Get a Sender that can be used to send messages on all connections.
    /// Calling `send` on this Sender is equivalent to calling `broadcast`.
    /// Calling `shutdown` on this Sender will shutdown the WebSocket even if no connections have
    /// been established.
    #[inline]
    pub fn broadcaster(&self) -> Sender {
        self.handler.sender()
    }
    
    /// Get the local socket address this socket is bound to. Will return an error
    /// if the backend returns an error. Will return a `NotFound` error if
    /// this WebSocket is not a listening socket.
    pub fn local_addr(&self) -> ::std::io::Result<SocketAddr> {
        self.handler.local_addr()
    }
}

/// Utility for constructing a WebSocket from various settings.
#[derive(Debug, Clone, Copy)]
pub struct Builder {
    settings: Settings,
}

// TODO: add convenience methods for each setting
impl Builder {
    /// Create a new Builder with default settings.
    pub fn new() -> Builder {
        Builder {
            settings: Settings::default(),
        }
    }
    
    /// Build a WebSocket using this builder and a factory.
    /// It is possible to use the same builder to create multiple WebSockets.
    pub fn build<F>(&self, factory: F) -> Result<WebSocket<F>>
                    where F: Factory
    {
        Ok(WebSocket {
            poll: Poll::new()?,
            handler: io::Handler::new(factory, self.settings),
        })
    }
    
    /// Set the WebSocket settings to use.
    pub fn with_settings(&mut self, settings: Settings) -> &mut Builder {
        self.settings = settings;
        self
    }
}
