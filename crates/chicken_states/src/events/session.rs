use {
    crate::states::session::{ServerStatus, ServerVisibility},
    bevy::prelude::Event,
};

#[cfg(feature = "hosted")]
use crate::states::session::ClientConnectionStatus;

/// Event to transition the server status.
#[derive(Event, Debug, Clone, Copy)]
pub struct SetServerStatus {
    /// The target server status.
    pub transition: ServerStatus,
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
}

/// Event to transition the server visibility state.
#[derive(Event, Debug, Clone, Copy)]
pub struct SetServerVisibility {
    /// The target server visibility state.
    pub transition: ServerVisibility,
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
}

/// Event to transition the client connection status.
#[cfg(feature = "hosted")]
#[derive(Event, Debug, Clone, Copy)]
pub struct SetClientConnectionStatus {
    /// The target client connection status.
    pub transition: ClientConnectionStatus,
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
}
