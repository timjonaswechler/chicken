//! Wiki/help menu screen states.
//!
//! Defines the documentation browser state [`WikiMenuScreen`] which tracks
//! the current documentation section: Overview, Creatures, Weapons, or Armor.
//!
//! The wiki allows free navigation between categories without returning to Overview,
//! making it easy to browse different documentation sections sequentially.

use {
    crate::states::menu::main::MainMenuScreen,
    bevy::prelude::{Reflect, StateSet, SubStates},
};

/// Tracks the current screen within the wiki/help menu.
///
/// Manages navigation between different documentation sections.
/// Only active when `MainMenuScreen` is `Wiki`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuScreen = MainMenuScreen::Wiki)]
pub enum WikiMenuScreen {
    /// Main wiki overview with topic categories.
    #[default]
    Overview,
    /// Creatures documentation section.
    Creatures,
    /// Weapons documentation section.
    Weapons,
    /// Armor documentation section.
    Armor,
}
