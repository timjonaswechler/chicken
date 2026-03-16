// include all expect of plugin
pub mod network {
    pub use chicken_network::*;
}

pub mod identity {
    pub use chicken_identity::*;
}

pub mod states {
    pub use chicken_states::*;
}

pub mod notifications {
    pub use chicken_notifications::*;
}

pub mod protocols {
    pub use chicken_protocols::*;
}

pub mod exitcodes {
    pub use chicken_exitcodes::*;
}

pub mod settings {
    pub use chicken_settings::prelude::*;

    pub use chicken_settings_content::{SettingsContent, networking::NetworkingSettings};
}
// pub mod steam {
//     pub use chicken_steam::*;
// }

use bevy::prelude::*;
use chicken_settings::SettingsAppExt;

pub struct ChickenPlugin;

impl Plugin for ChickenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            chicken_notifications::ChickenNotificationPlugin,
            chicken_settings::ChickenSettingsPlugin,
            chicken_states::ChickenStatePlugin,
            chicken_network::ChickenNetPlugin,
            chicken_protocols::ProtocolPlugin,
            chicken_identity::ChickenIdentityPlugin,
        ))
        .add_settings::<settings::SettingsContent>()
        .add_settings::<settings::NetworkingSettings>();
        // .add_systems(Startup, test_identity);
    }
}

fn test_identity(identity: Res<identity::PlayerIdentity>) {
    error!("Identity: {:?}", identity);
}
