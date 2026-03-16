//! Menu navigation logic and state transition handlers.
//!
//! Aggregates menu sub-plugins and provides the main [`MenuPlugin`]:
//! - `singleplayer`: Singleplayer menu flow observers and validation
//! - `multiplayer`: Multiplayer hosting and joining logic
//! - `settings`: Settings navigation and apply/cancel handling
//! - `wiki`: Wiki documentation browser navigation
//!
//! Each sub-plugin registers its own substates and observers for menu events.

pub(crate) mod multiplayer;
pub(crate) mod settings;
pub(crate) mod singleplayer;
pub(crate) mod wiki;

use bevy::prelude::{App, Plugin};
use multiplayer::MultiplayerMenuPlugin;
use settings::SettingsMenuPlugin;
use singleplayer::SingleplayerMenuPlugin;
use wiki::WikiMenuPlugin;

/// Plugin that manages all menu navigation state machines.
///
/// Registers sub-plugins for singleplayer, multiplayer, settings, and wiki menus.
/// Each sub-plugin handles its own state transitions via observers for menu-specific events.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SingleplayerMenuPlugin,
            MultiplayerMenuPlugin,
            SettingsMenuPlugin,
            WikiMenuPlugin,
        ));
    }
}
