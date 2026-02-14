use {crate::states::menu::wiki::WikiMenuScreen, bevy::prelude::Event};

/// Events for navigating within the wiki/help menu.
///
/// Used to browse different sections of the in-game documentation
/// and return to the previous menu.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiMenuEvent {
    /// Navigate to a specific wiki screen or section.
    Navigate(WikiMenuScreen),
    /// Return to the main menu.
    Back,
}
