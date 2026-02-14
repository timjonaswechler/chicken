use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use chicken_settings::Settings;

pub const DISCOVERY_PORT: u16 = 30150;

#[derive(Settings, Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[settings("networking.toml")]
pub struct NetworkingSettings {
    /// Game port (default: 30105)
    #[serde(default)]
    pub port: u16,

    /// Discovery port (30150)
    #[serde(default)]
    pub discovery_port: u16,

    /// Can be discovered (default: false)
    #[serde(default)]
    pub can_be_discovered: bool,
}

impl Default for NetworkingSettings {
    fn default() -> Self {
        Self {
            port: 30105,
            discovery_port: DISCOVERY_PORT,
            // if we are in debug mode the server can be discovered as default
            // if the Build is in release mode the server can not be discovered as default
            can_be_discovered: cfg!(debug_assertions),
        }
    }
}

impl NetworkingSettings {
    pub fn new(port: u16, can_be_discovered: bool) -> Self {
        Self {
            port,
            discovery_port: DISCOVERY_PORT,
            can_be_discovered,
        }
    }
}
