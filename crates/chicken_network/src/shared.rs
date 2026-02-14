use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

/// Network message sent from client to server to communicate the player's chosen name.
///
/// Sent during the connection handshake after the client establishes a connection.
/// The server uses this to identify the player and broadcast their name to other clients.
#[derive(Debug, Message, Clone, Serialize, Deserialize)]
pub struct PlayerNameMessage {
    /// The player's display name as entered by the user.
    pub player_name: String,
}

/// Resource storing the local player's chosen name.
///
/// This resource holds the name that will be sent to the server via `PlayerNameMessage`
/// when connecting. It is typically set from user input in the connection UI.
#[derive(Resource, Clone)]
pub struct PlayerName {
    /// The local player's display name.
    pub name: String,
}

pub(crate) struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_message::<PlayerNameMessage>(Channel::Ordered);
    }
}
