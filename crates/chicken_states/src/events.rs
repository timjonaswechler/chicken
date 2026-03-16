//! Events for triggering state transitions across the application.
//!
//! This module defines the event types used to request state changes:
//! - `app`: Application scope transitions (Menu, Session, Exit)
//! - `menu`: Menu navigation events for UI interaction (hosted builds only)
//! - `session`: Game session control events (pause menu, server lifecycle, connection flow)
//!
//! Events are processed by observers in the corresponding [`crate::logic`] modules
//! which validate and execute the requested state transitions.

pub mod app;
#[cfg(feature = "hosted")]
pub mod menu;
pub mod session;
