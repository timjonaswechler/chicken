//! Settings menu screen states.
//!
//! Defines the settings configuration state [`SettingsMenuScreen`] which tracks
//! the current settings category: Overview, Audio, Video, or Controls.
//!
//! Unlike other menu categories, settings requires returning to Overview before
//! switching categories (except via Back/Apply/Cancel actions). This enforces
//! a clear navigation pattern and proper handling of unsaved changes.

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
