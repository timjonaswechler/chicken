//! Singleplayer menu navigation events.
//!
//! Events for controlling the singleplayer setup workflows:
//! - [`SetSingleplayerMenu`]: Navigate between overview, new game, and load game screens
//! - [`SetSingleplayerNewGame`]: Step through new game creation (player config, world config, save config)
//! - [`SetSingleplayerSavedGame`]: Select and load existing save files
//!
//! Triggered by UI interactions and processed by the `logic::menu::singleplayer` observers.

use bevy::prelude::Event;

/// Events for navigating within the singleplayer setup menu.
///
/// Used to switch between different singleplayer setup contexts or return
/// to the main menu.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSingleplayerMenu {
    /// Navigate to the singleplayer overview screen.
    Overview,

    /// Navigate to the new game setup screen.
    NewGame,

    /// Navigate to the load game screen.
    LoadGame,

    /// Return to the main menu.
    Back,
}

/// Events for controlling the new singleplayer game creation workflow.
///
/// Used during the step-by-step configuration of a new singleplayer game,
/// including player settings, world generation parameters, and save options.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSingleplayerNewGame {
    /// Advance to the next configuration step.
    Next,
    /// Return to the previous configuration step.
    Previous,
    /// Confirm settings and start the new game.
    Confirm,
    /// Cancel the game creation process and discard all configuration.
    Cancel,
}

/// Events for controlling the saved game loading workflow.
///
/// Used when loading an existing singleplayer save file, managing
/// save selection and load configuration.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSingleplayerSavedGame {
    /// Advance to the next step in the load game flow.
    Next,
    /// Return to the previous step.
    Previous,
    /// Confirm the selected save and load the game.
    Confirm,
    /// Cancel the load process.
    Cancel,
}
