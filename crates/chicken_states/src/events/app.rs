//! Application scope transition events.
//!
//! Defines events for high-level application state changes:
//! - Transition between Menu and Session scopes
//! - Exit the application gracefully
//!
//! These events are processed by [`crate::logic::app`] observers which validate
//! the requested transition against allowed state transitions before applying.

use bevy::prelude::Event;

/// Event to transition the application to a different high-level scope.
///
/// Used for major application state changes like moving from the splash screen
/// to the main menu, or from the menu into an active game session.
#[derive(Event, Debug)]
pub enum SetAppScope {
    /// Transition to the main menu.
    Menu,

    /// Transition to an active game session.
    Session,

    /// Exit the application.
    Exit,
}
