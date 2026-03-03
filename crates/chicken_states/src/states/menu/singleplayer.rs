use {
    crate::states::menu::main::MainMenuScreen,
    bevy::prelude::{Reflect, StateSet, SubStates},
};

/// Tracks the current screen within the singleplayer setup flow.
///
/// Manages navigation between new game creation and save loading options.
/// Only active when `MainMenuScreen` is `Singleplayer`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(MainMenuScreen = MainMenuScreen::Singleplayer)]
pub enum SingleplayerMenuScreen {
    /// Main singleplayer menu with options for new game or load game.
    #[default]
    Overview,
    /// Setup flow for creating a new singleplayer game.
    NewGame,
    /// Save file selection for loading an existing game.
    LoadGame,
}

/// Tracks the current configuration step when creating a new singleplayer game.
///
/// Guides the user through player, world, and save configuration.
/// Only active when `SingleplayerMenuScreen` is `NewGame`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SingleplayerMenuScreen = SingleplayerMenuScreen::NewGame)]
pub enum NewGameMenuScreen {
    /// Configure player settings (name, appearance).
    #[default]
    ConfigPlayer,
    /// Configure world generation parameters.
    ConfigWorld,
    /// Configure save file name and location.
    ConfigSave,
}

/// Tracks the current screen when loading a saved game.
///
/// Manages save file selection.
/// Only active when `SingleplayerMenuScreen` is `LoadGame`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SingleplayerMenuScreen = SingleplayerMenuScreen::LoadGame)]
pub enum SavedGameMenuScreen {
    /// Browse and select a save file to load.
    #[default]
    SelectSaveGame,
}
