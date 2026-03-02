#![warn(missing_docs, clippy::unwrap_used)]

//! The crate is designed in a modular manner.
//! The modules are organized into the following categories:
//! - [logic]: Contains the state management logic for the game and is hidden from the user.
//! - [states]: Contains all states and events to change the states that are used in the game. The user can set these states directly , but is preferred to use the provided  Events to get a clean state change. In Some cases, changing one state has an impact on other states. For example, changing the app scope may also affect the session state.
//!  <div class="warning">
//!  The module [states] should be split into states and events.
//!  Events should be have a uniform interface.
//! <\div>

#[cfg(all(not(feature = "hosted"), not(feature = "headless")))]
compile_error!("You must enable either the 'hosted' or 'headless' feature to build this crate.");

#[cfg(all(feature = "hosted", feature = "headless"))]
compile_error!("You cannot enable both the 'hosted' and 'headless' features.");

/// Here are all events that can be used to change the states that are used for the generell app logic.
pub mod events;

// TODO: Observer Entities gebunden hinzu fügenn oder entfernen
/// In this module, the app logic is implemented.
/// Most of the logic ist implemented as observers.
/// to provide a clean state change it is nesseary to add the observers that are available to you to combine witha controll entity.
pub mod logic;

/// Contains all states and events to change the states that are used in the game.
/// The user is intended to use the provided Events to get a clean state change.
/// In Some cases, changing one state has an impact on other states.
/// For example, changing the app scope may also affect the session state.
pub mod states;

use bevy::prelude::{App, Plugin};

/// This plugin bundles all status management logic for the game.
pub struct ChickenStatePlugin;

impl Plugin for ChickenStatePlugin {
    fn build(&self, app: &mut App) {
        // App Logic (Initialization)
        app.add_plugins(logic::app::AppLogicPlugin);

        // Session Logic
        #[cfg(feature = "hosted")]
        app.add_plugins((
            logic::session::client::ClientSessionPlugin,
            logic::menu::MenuPlugin,
        ));

        // Server Logic (Dedicated Server OR Singleplayer Host)
        #[cfg(any(feature = "hosted", feature = "headless"))]
        app.add_plugins(logic::session::server::ServerSessionPlugin);
    }
}
