use {
    super::main::MainMenuContext,
    bevy::prelude::{Reflect, StateSet, SubStates},
};

/// Tracks the current settings configuration screen.
///
/// Manages navigation between different settings categories.
/// Only active when `MainMenuContext` is `Settings`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuContext = MainMenuContext::Settings)]
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
