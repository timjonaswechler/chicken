//! Wiki/help menu navigation events.
//!
//! Events for browsing the in-game documentation:
//! - Navigate between documentation categories: Overview, Creatures, Weapons, Armor
//! - Return to the main menu
//!
//! Unlike settings, wiki navigation allows free movement between categories
//! without returning to overview. Processed by the `logic::menu::wiki` observers.

use bevy::prelude::Event;

/// Events for navigating within the wiki/help menu.
///
/// Used to browse different sections of the in-game documentation
/// and return to the previous menu.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetWikiMenu {
    /// Navigate to the wiki overview.
    Overview,
    /// Navigate to the creatures documentation.
    Creatures,
    /// Navigate to the weapons documentation.
    Weapons,
    /// Navigate to the armor documentation.
    Armor,
    /// Return to the main menu (from Overview).
    Back,
}
