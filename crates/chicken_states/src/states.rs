//! State definitions for the game's state machines.
//!
//! This module contains all Bevy state types organized by scope:
//! - `app`: High-level application scope (Splash, Menu, Session)
//! - `menu`: Menu navigation states for the main menu UI (hosted builds only)
//! - `session`: Game session states including server/client lifecycle and pause menus
//!
//! States are implemented using Bevy's state system with `States`, `SubStates`,
//! and `ComputedStates` for hierarchical state management.

pub mod app;
#[cfg(feature = "hosted")]
pub mod menu;
pub mod session;
