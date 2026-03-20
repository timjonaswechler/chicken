pub mod auth;
pub mod chat;
pub mod discovery;
pub mod local;
pub mod networking;
pub mod quic;
pub mod roles;
pub mod steam;

use bevy::prelude::*;

pub(crate) struct ServerLogicPlugin;

impl Plugin for ServerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            roles::ServerRolesPlugin,
            auth::ServerAuthPlugin,
            chat::ServerChatPlugin,
            discovery::DiscoveryServerPlugin,
            local::LocalServerPlugin,
            quic::QUICServerPlugin,
            // steam::SteamServerPlugin,
        ));
    }
}
