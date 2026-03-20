pub mod networking;
pub mod server_access;
pub use server_access::{BlacklistEntry, ServerAccessSettings};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use chicken_settings::Settings;

#[derive(Settings, Resource, Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[settings("settings.toml")]
pub struct SettingsContent {
    pub audio: AudioSettings,
    pub graphics: GraphicsSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSettings {
    pub volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self { volume: 0.8 }
    }
}

impl AudioSettings {
    pub fn new(volume: f32) -> Self {
        Self { volume }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphicsSettings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            fullscreen: false,
        }
    }
}

impl GraphicsSettings {
    pub fn new(resolution: (u32, u32), fullscreen: bool) -> Self {
        Self {
            resolution,
            fullscreen,
        }
    }
}
