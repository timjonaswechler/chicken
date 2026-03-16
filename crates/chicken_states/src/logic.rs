//! State transition logic and observers.
//!
//! This module contains the implementation of state transition handlers:
//! - `app`: Application scope transition logic and validation
//! - `menu`: Menu navigation observers and transition validators (hosted builds only)
//! - `session`: Server lifecycle and client connection flow management
//!
//! Each submodule provides plugins that register observers for events defined
//! in [`crate::events`], validating transitions before applying state changes.

pub mod app;
#[cfg(feature = "hosted")]
pub mod menu;
pub mod session;
