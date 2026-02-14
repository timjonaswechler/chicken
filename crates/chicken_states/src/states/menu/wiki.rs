use {
    super::main::MainMenuContext,
    bevy::prelude::{Reflect, StateSet, SubStates},
};

/// Tracks the current screen within the wiki/help menu.
///
/// Manages navigation between different documentation sections.
/// Only active when `MainMenuContext` is `Wiki`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuContext = MainMenuContext::Wiki)]
pub enum WikiMenuScreen {
    /// Main wiki overview with topic categories.
    #[default]
    Overview,
}
