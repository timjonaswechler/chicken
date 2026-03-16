//! Multiplayer menu navigation events.
//!
//! Events for controlling multiplayer setup workflows:
//! - [`SetMultiplayerMenu`]: Navigate between overview, host new/saved game, and join game screens
//! - [`SetNewHostGame`]: Step through new multiplayer server configuration
//! - [`SetSavedHostGame`]: Select existing save and configure server settings
//! - [`SetJoinGame`]: Connect to remote multiplayer servers
//!
//! Confirmation events trigger session initialization and transition to the game.
//! Processed by the `logic::menu::multiplayer` observers.

use bevy::prelude::Event;

/// Events for navigating within the multiplayer setup menu.
///
/// Used to switch between different multiplayer setup contexts or return
/// to the parent menu.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetMultiplayerMenu {
    /// Navigate to the overview settings screen.
    Overview,
    /// Navigate to the host new game screen.
    HostNewGame,
    /// Navigate to the host saved game screen.
    HostSavedGame,
    /// Navigate to the join game screen.
    JoinGame,
    /// Return to the previous menu level.
    Back,
}

/// Events for controlling the new multiplayer game hosting workflow.
///
/// Used during the step-by-step configuration of a new multiplayer server,
/// including server settings, world generation, and save configuration.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetNewHostGame {
    /// Advance to the next configuration step.
    Next,
    /// Return to the previous configuration step.
    Previous,
    /// Confirm settings and proceed to game start.
    Confirm,
    /// Cancel the hosting process and discard all configuration.
    Cancel,
}

/// Events for controlling the saved game hosting workflow.
///
/// Used when hosting a multiplayer session from an existing saved game,
/// allowing selection and configuration of the save file.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSavedHostGame {
    /// Advance to the next step in the saved game hosting flow.
    Next,
    /// Return to the previous step.
    Previous,
    /// Confirm the selected save and start hosting.
    Confirm,
    /// Cancel the hosting process.
    Cancel,
}

/// Events for controlling the game joining workflow.
///
/// Used when connecting to an existing multiplayer server, managing
/// the server discovery, selection, and connection process.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetJoinGame {
    /// Advance to the next step in the join process.
    Next,
    /// Return to the previous step.
    Previous,
    /// Confirm server selection and attempt connection.
    Confirm,
    /// Cancel the join process.
    Cancel,
}
