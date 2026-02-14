use {
    super::main::MainMenuContext,
    bevy::prelude::{Reflect, StateSet, SubStates},
};

/// Tracks the current screen within the multiplayer setup flow.
///
/// Manages the navigation between hosting and joining options.
/// Only active when `MainMenuContext` is `Multiplayer`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuContext = MainMenuContext::Multiplayer)]
pub enum MultiplayerSetup {
    /// Main multiplayer menu with options to host or join games.
    #[default]
    Overview,
    /// Setup flow for hosting a new multiplayer game.
    HostNewGame,
    /// Setup flow for hosting from an existing save file.
    HostSavedGame,
    /// Setup flow for joining an existing multiplayer session.
    JoinGame,
}

/// Tracks the current configuration step when hosting a new multiplayer game.
///
/// Guides the user through server, world, and save configuration.
/// Only active when `MultiplayerSetup` is `HostNewGame`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerSetup = MultiplayerSetup::HostNewGame)]
pub enum HostNewGameMenuScreen {
    /// Configure server settings (name, password, max players, visibility).
    #[default]
    ConfigServer,
    /// Configure world generation parameters.
    ConfigWorld,
    /// Configure save file location and options.
    ConfigSave,
}

/// Tracks the current screen when hosting from a saved game.
///
/// Manages save selection and server configuration.
/// Only active when `MultiplayerSetup` is `HostSavedGame`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerSetup = MultiplayerSetup::HostSavedGame)]
pub enum HostSavedGameMenuScreen {
    /// Select a saved game file to host.
    #[default]
    Overview,
    /// Configure server settings for the saved game session.
    ConfigServer,
}

/// Tracks the current screen when joining a multiplayer game.
///
/// Manages server discovery and connection setup.
/// Only active when `MultiplayerSetup` is `JoinGame`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerSetup = MultiplayerSetup::JoinGame)]
pub enum JoinGameMenuScreen {
    /// Server browser and connection interface.
    #[default]
    Overview,
}
