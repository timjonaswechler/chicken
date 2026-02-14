#![warn(missing_docs, clippy::unwrap_used)]

//! The crate is designed in a modular manner.
//! The modules are organized into the following categories:
//! - [logic]: Contains the state management logic for the game and is hidden from the user.
//! - [states]: Contains all states and events to change the states that are used in the game. The user can set these states directly , but is preferred to use the provided  Events to get a clean state change. In Some cases, changing one state has an impact on other states. For example, changing the app scope may also affect the session state.
//!  <div class="warning">
//!  The module [states] should be split into states and events.
//!  Events should be have a uniform interface.
//! <\div>

#[cfg(all(not(feature = "client"), not(feature = "server")))]
compile_error!(
    "You must enable either the 'client' or 'server' feature (or both) to build this crate."
);
pub(crate) mod events;
pub(crate) mod logic;
pub(crate) mod states;

pub use {
    events::session::SetServerVisibility,
    states::{
        app::AppScope,
        session::{PhysicsSimulation, ServerVisibility, SessionState},
    },
};

#[cfg(feature = "client")]
pub use {
    events::{
        app::ChangeAppScope,
        menu::{
            PauseMenuEvent,
            main::MainMenuInteraction,
            multiplayer::{SetJoinGame, SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame},
            settings::SettingsMenuEvent,
            singleplayer::{SetSingleplayerMenu, SetSingleplayerNewGame, SetSingleplayerSavedGame},
            wiki::WikiMenuEvent,
        },
        session::{
            SetClientShutdownStep, SetClientStatus, SetSingleplayerShutdownStep,
            SetSingleplayerStatus,
        },
    },
    states::{
        menu::{
            PauseMenu,
            main::MainMenuContext,
            multiplayer::{
                HostNewGameMenuScreen, HostSavedGameMenuScreen, JoinGameMenuScreen,
                MultiplayerSetup,
            },
            settings::SettingsMenuScreen,
            singleplayer::{NewGameMenuScreen, SavedGameMenuScreen, SingleplayerSetup},
            wiki::WikiMenuScreen,
        },
        session::{
            ClientShutdownStep, ClientStatus, SessionType, SingleplayerShutdownStep,
            SingleplayerStatus,
        },
    },
};

use bevy::prelude::{App, Plugin};

/// This plugin bundles all status management logic for the game.
pub struct ChickenStatePlugin;

impl Plugin for ChickenStatePlugin {
    fn build(&self, app: &mut App) {
        // App Logic (Initialization)
        app.add_plugins(logic::app::AppLogicPlugin);

        // Session Logic
        #[cfg(feature = "client")]
        app.add_plugins(logic::session::client::ClientSessionPlugin);

        // Server Logic (Dedicated Server OR Singleplayer Host)
        #[cfg(any(feature = "server", feature = "client"))]
        app.add_plugins(logic::session::server::ServerSessionPlugin);

        // Menu Logic
        #[cfg(feature = "client")]
        app.add_plugins(logic::menu::MenuPlugin);
    }
}
