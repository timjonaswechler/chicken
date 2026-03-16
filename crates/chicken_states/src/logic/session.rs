//! Game session lifecycle management.
//!
//! Handles server startup/shutdown sequences and client connection flows:
//! - [`client`]: Client-side connection lifecycle (connecting, syncing, disconnecting)
//!   and pause menu handling. Only available in hosted builds.
//! - [`server`]: Server-side lifecycle management including startup/shutdown steps,
//!   visibility transitions (public/private), and session state management.
//!   Available in both hosted and headless builds.
//!
//! These modules work together to coordinate game sessions across singleplayer,
//! multiplayer host, multiplayer client, and dedicated server configurations.

#[cfg(feature = "hosted")]
pub mod client;
pub mod server;
