use {
    crate::states::menu::main::MainMenuScreen,
    bevy::prelude::{Reflect, StateSet, SubStates},
};

/// Tracks the current settings configuration screen.
///
/// Manages navigation between different settings categories.
/// Only active when `MainMenuScreen` is `Settings`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuScreen = MainMenuScreen::Settings)]
pub enum SettingsMenuScreen {
    /// Main settings overview with category selection.
    #[default]
    Overview,
    /// Audio settings (volume levels, device selection).
    Audio,
    /// Video settings (resolution, graphics quality, fullscreen).
    Video,
    /// Control bindings and input configuration.
    Controls,
}
