use {
    crate::states::session::{ClientStatus, ServerVisibility, SingleplayerStatus},
    bevy::prelude::Event,
};

/// Events for transitioning the client connection status.
///
/// Used to manage the client lifecycle during connection attempts,
/// including successful transitions and failure handling.
#[cfg(feature = "client")]
#[derive(Event, Debug, Clone, Copy)]
pub enum SetClientStatus {
    /// Transition to a new client connection status.
    Transition(ClientStatus),
    /// Mark the connection attempt as failed.
    Failed,
}

/// Events for controlling the client shutdown sequence.
///
/// Used to gracefully shut down the client connection in stages,
/// ensuring proper cleanup of network resources.
#[cfg(feature = "client")]
#[derive(Event, Debug, Clone, Copy)]
pub enum SetClientShutdownStep {
    /// Initiate the shutdown process.
    Start,
    /// Proceed to the next shutdown step.
    Next,
    /// Complete the shutdown process.
    Done,
}

/// Event to transition the singleplayer session status.
///
/// Used to manage the lifecycle of a local singleplayer game session,
/// tracking transitions between starting, running, and stopping states.
#[cfg(feature = "client")]
#[derive(Event, Debug, Clone, Copy)]
pub struct SetSingleplayerStatus {
    /// The target singleplayer status to transition to.
    pub transition: SingleplayerStatus,
}

/// Events for controlling the singleplayer session shutdown sequence.
///
/// Used to gracefully shut down a singleplayer session in stages,
/// including cleanup of local server, clients, and bots.
#[cfg(feature = "client")]
#[derive(Event, Debug, Clone, Copy)]
pub enum SetSingleplayerShutdownStep {
    /// Initiate the singleplayer shutdown process.
    Start,
    /// Proceed to the next shutdown step.
    Next,
    /// Complete the singleplayer shutdown process.
    Done,
}

/// Event to transition the server visibility state.
///
/// Used to manage the visibility of a multiplayer server,
/// controlling whether it appears in public server listings.
#[derive(Event, Debug, Clone, Copy)]
pub struct SetServerVisibility {
    /// The target server visibility state.
    pub transition: ServerVisibility,
}
