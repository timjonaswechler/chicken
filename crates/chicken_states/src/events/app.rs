#![cfg(feature = "hosted")]

use {crate::states::app::AppScope, bevy::prelude::Event};

/// Event to transition the application to a different high-level scope.
///
/// Used for major application state changes like moving from the splash screen
/// to the main menu, or from the menu into an active game session.
#[derive(Event, Debug)]
pub struct ChangeAppScope {
    /// The target application scope to transition to.
    pub transition: AppScope,
}
