//! Main menu screen states.
//!
//! Defines the top-level menu navigation state [`MainMenuScreen`] which tracks
//! the current section within the main menu: Overview, Singleplayer, Multiplayer,
//! Wiki, or Settings.
//!
//! This is the root of the menu state hierarchy, with each variant having its own
//! nested substates for detailed navigation within that section.

use crate::states::app::AppScope;
use bevy::prelude::{Reflect, StateSet, SubStates};
/// Tracks the current section within the main menu.
///
/// This state manages navigation between the top-level menu categories.
/// Only active when the application is in the `Menu` scope.
#[cfg(feature = "hosted")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(AppScope = AppScope::Menu)]
pub enum MainMenuScreen {
    /// The main menu root with primary options (Singleplayer, Multiplayer, Wiki, Settings, Exit).
    #[default]
    Overview,
    /// Singleplayer game setup and load screens.
    Singleplayer,
    /// Multiplayer game setup (host/join) screens.
    Multiplayer,
    /// In-game wiki and help documentation.
    Wiki,
    /// Game settings configuration screens.
    Settings,
}
