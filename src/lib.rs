//! Lightweight, event-driven WebSockets for Rust.
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
#[cfg(feature="ssl")]
extern crate openssl;
#[macro_use]
extern crate log;

pub mod io;
pub mod ws;
use ws::*;
