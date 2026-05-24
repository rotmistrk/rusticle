//! LSP server implementation for rusticle/Tcl scripts.
//!
//! Provides `Server` for handling JSON-RPC messages, and transport
//! functions for stdio Content-Length framing.

mod handler;
mod transport;

pub use handler::Server;
pub use transport::{read_message, write_message};
