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
