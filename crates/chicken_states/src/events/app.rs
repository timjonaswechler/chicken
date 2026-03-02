use {crate::states::app::AppScope, bevy::prelude::Event};

/// Event to transition the application to a different high-level scope.
///
/// Used for major application state changes like moving from the splash screen
/// to the main menu, or from the menu into an active game session.
#[derive(Event, Debug)]
pub enum SetAppScope {
    /// Transition to a specific app scope.
    To(AppScope),

    /// Exit the application.
    Exit,
}
