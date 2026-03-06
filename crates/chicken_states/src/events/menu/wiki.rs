use bevy::prelude::Event;

/// Events for navigating within the wiki/help menu.
///
/// Used to browse different sections of the in-game documentation
/// and return to the previous menu.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetWikiMenu {
    /// Navigate to the creatures documentation.
    Creatures,
    /// Navigate to the weapons documentation.
    Weapons,
    /// Navigate to the armor documentation.
    Armor,
    /// Return to the main menu (from Overview).
    Back,
}
