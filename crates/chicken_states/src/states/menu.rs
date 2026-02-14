pub(crate) mod main;
pub(crate) mod multiplayer;
pub(crate) mod settings;
pub(crate) mod singleplayer;
pub(crate) mod wiki;

use super::session::SessionState;
use bevy::prelude::{Reflect, StateSet, SubStates};

/// Tracks which pause menu screen is currently active.
///
/// This is a substate that is only valid when the session is in the `Paused` state.
/// It manages the navigation flow within the pause menu system.
#[derive(SubStates, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[source(SessionState = SessionState::Paused)]
pub enum PauseMenu {
    /// Main pause menu with primary options (Resume, Settings, Save, Load, Exit).
    #[default]
    Overview,
    /// Settings configuration screen for audio, video, and controls.
    Settings,
    /// Save game slot selection screen.
    Save,
    /// Load game slot selection screen.
    Load,
    /// Exit confirmation dialog.
    Exit,
}
