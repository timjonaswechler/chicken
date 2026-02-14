pub(crate) mod main;
pub(crate) mod multiplayer;
pub(crate) mod settings;
pub(crate) mod singleplayer;
pub(crate) mod wiki;

use bevy::prelude::Event;

/// Events for actions in the in-game pause menu.
///
/// Triggered when the player interacts with pause menu UI elements
/// while the game session is in a paused state.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuEvent {
    /// Resume gameplay and close the pause menu.
    Resume,
    /// Open the settings configuration screen.
    Settings,
    /// Open the save game dialog.
    Save,
    /// Open the load game dialog.
    Load,
    /// Exit to the main menu or desktop.
    Exit,
}
