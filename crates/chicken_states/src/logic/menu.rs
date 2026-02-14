pub mod main;
pub mod multiplayer;
pub mod settings;
pub mod singleplayer;
pub mod wiki;

use bevy::prelude::{App, Plugin};
use main::MainMenuPlugin;
use multiplayer::MultiplayerMenuPlugin;
use settings::SettingsMenuPlugin;
use singleplayer::SingleplayerMenuPlugin;
use wiki::WikiMenuPlugin;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SingleplayerMenuPlugin,
            MultiplayerMenuPlugin,
            SettingsMenuPlugin,
            WikiMenuPlugin,
            MainMenuPlugin,
        ));
    }
}
