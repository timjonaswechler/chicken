use crate::states::session::SessionType;
use bevy::prelude::Event;

/// Events for actions in the in-game pause menu.
///
/// Triggered when the player interacts with pause menu UI elements
/// while the game session is in a paused state.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPauseMenu {
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

#[derive(Event, Debug, Clone, Copy)]
/// Events for controlling the session type.
pub enum SetSessionType {
    /// Set the session type to the given value.
    To(SessionType),
}

/// Events for controlling the server startup sequence.
#[derive(Event, Debug, Clone, Copy)]
pub enum SetServerStartupStep {
    /// Initiate the startup process.
    Start,
    /// Proceed to the next startup step.
    Next,
    /// Startup process complete.
    Done,
    /// Startup process failed.
    Failed,
}

/// Events for controlling the server shutdown sequence.
#[derive(Event, Debug, Clone, Copy)]
pub enum SetServerShutdownStep {
    /// Initiate the shutdown process.
    Start,
    /// Proceed to the next shutdown step.
    Next,
    /// Shutdown process complete.
    Done,
    /// Shutdown process failed.
    Failed,
}

/// Events for controlling the going-public sequence.
#[derive(Event, Debug, Clone, Copy)]
pub enum SetGoingPublicStep {
    /// Initiate the going-public process.
    Start,
    /// Proceed to the next step.
    Next,
    /// Process complete.
    Done,
    /// Going-public process failed.
    Failed,
}

/// Events for controlling the going-private sequence.
#[derive(Event, Debug, Clone, Copy)]
pub enum SetGoingPrivateStep {
    /// Initiate the going-private process.
    Start,
    /// Proceed to the next step.
    Next,
    /// Process complete.
    Done,
    /// Process failed.
    Failed,
}

/// Events for controlling the connecting sequence.
#[cfg(feature = "hosted")]
#[derive(Event, Debug, Clone, Copy)]
pub enum SetConnectingStep {
    /// Initiate the connection process.
    Start,
    /// Proceed to the next connecting step.
    Next,
    /// Connection process complete.
    Done,
    /// Connection process failed.
    Failed,
}

/// Events for controlling the syncing sequence.
#[cfg(feature = "hosted")]
#[derive(Event, Debug, Clone, Copy)]
pub enum SetSyncingStep {
    /// Initiate the syncing process.
    Start,
    /// Proceed to the next syncing step.
    Next,
    /// Syncing process complete.
    Done,
    /// Syncing process failed.
    Failed,
}

/// Events for controlling the disconnecting sequence.
#[cfg(feature = "hosted")]
#[derive(Event, Debug, Clone, Copy)]
pub enum SetDisconnectingStep {
    /// Initiate the disconnect process.
    Start,
    /// Proceed to the next disconnect step.
    Next,
    /// Disconnect process complete.
    Done,
    /// Disconnect process failed.
    Failed,
}
