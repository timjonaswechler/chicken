#[cfg(feature = "client")]
pub mod client;

// Available for both dedicated server AND client (as local host)
#[cfg(any(feature = "server", feature = "client"))]
pub mod server; 
