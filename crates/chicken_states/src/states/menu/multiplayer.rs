//! Multiplayer menu screen states.
//!
//! Defines states for multiplayer game setup:
//! - [`MultiplayerMenuScreen`]: Top-level multiplayer navigation (Overview, HostNewGame, HostSavedGame, JoinGame)
//! - [`HostNewGameMenuScreen`]: New multiplayer server configuration steps
//! - [`HostSavedGameMenuScreen`]: Existing save hosting workflow
//! - [`JoinGameMenuScreen`]: Server browser and connection interface
//!
//! These substates nest under `MainMenuScreen::Multiplayer` and handle both
//! hosting and joining multiplayer sessions.

use {
    crate::states::menu::main::MainMenuScreen,
    bevy::prelude::{Reflect, StateSet, SubStates},
};

/// Tracks the current screen within the multiplayer setup flow.
///
/// Manages the navigation between hosting and joining options.
/// Only active when `MainMenuScreen` is `Multiplayer`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuScreen = MainMenuScreen::Multiplayer)]
pub enum MultiplayerMenuScreen {
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
/// Only active when `MultiplayerMenuScreen` is `HostNewGame`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerMenuScreen = MultiplayerMenuScreen::HostNewGame)]
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
/// Only active when `MultiplayerMenuScreen` is `HostSavedGame`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerMenuScreen = MultiplayerMenuScreen::HostSavedGame)]
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
/// Only active when `MultiplayerMenuScreen` is `JoinGame`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MultiplayerMenuScreen = MultiplayerMenuScreen::JoinGame)]
pub enum JoinGameMenuScreen {
    /// Server browser and connection interface.
    #[default]
    Overview,
}
