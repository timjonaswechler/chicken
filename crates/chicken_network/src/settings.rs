// use bevy::prelude::*;
// use bevy_settings::*;
// use serde::{Deserialize, Serialize};

// #[cfg(feature = "server")]
// #[derive(SettingsGroup, Resource, Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
// #[reflect(Resource)]
// #[settings("server.toml")]
// pub struct ServerSettings {
//     pub network: NetworkSettings,
// }

// #[cfg(feature = "client")]
// #[derive(SettingsGroup, Resource, Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
// #[reflect(Resource)]
// #[settings("temp.toml")]
// pub struct ServerSettings {
//     pub network: NetworkSettings,
// }

// #[cfg(all(not(feature = "client"), not(feature = "server")))]
// #[derive(SettingsGroup, Resource, Serialize, Deserialize, Default, Clone, Reflect, PartialEq)]
// #[reflect(Resource)]
// #[settings("temp.toml")]
// pub struct ServerSettings {
//     pub network: NetworkSettings,
// }

// #[derive(Serialize, Deserialize, Clone, Reflect, PartialEq)]
// pub struct NetworkSettings {
//     /// Game port (default: 30105)
//     #[serde(default)]
//     pub port: u16,

//     /// Discovery port (default: 30150)
//     #[serde(default)]
//     pub discovery_port: u16,

//     /// Can be discovered (default: true)
//     #[serde(default)]
//     pub can_be_discovered: bool,
// }

// impl Default for NetworkSettings {
//     fn default() -> Self {
//         NetworkSettings {
//             port: 30105,
//             discovery_port: 30150,
//             can_be_discovered: true,
//         }
//     }
// }
