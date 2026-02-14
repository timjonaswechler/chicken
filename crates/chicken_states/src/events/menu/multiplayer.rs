use {crate::states::menu::multiplayer::MultiplayerSetup, bevy::prelude::Event};

/// Events for navigating within the multiplayer setup menu.
///
/// Used to switch between different multiplayer setup contexts or return
/// to the parent menu.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetMultiplayerMenu {
    /// Navigate to a specific multiplayer setup screen (Overview, HostNewGame, HostSavedGame, JoinGame).
    Navigate(MultiplayerSetup),
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
    /// Return to the multiplayer overview without saving changes.
    Back,
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
    /// Return to the multiplayer overview.
    Back,
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
    /// Return to the multiplayer overview.
    Back,
    /// Cancel the join process.
    Cancel,
}
