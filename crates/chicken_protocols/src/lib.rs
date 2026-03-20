#[cfg(all(not(feature = "hosted"), not(feature = "headless")))]
compile_error!("You must enable either the 'hosted' or 'headless' feature to build this crate.");

#[cfg(all(feature = "hosted", feature = "headless"))]
compile_error!("You cannot enable both the 'hosted' and 'headless' features.");

pub mod auth;
pub mod chat;

pub use auth::*;
pub use chat::*;
