use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Resource, die die Spieler-Identität speichert
/// Wird von allen Crates genutzt (chat, network, ui)
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayerIdentity {
    /// Anzeigename (aus Steam, oder manuell gesetzt)
    pub display_name: String,
    /// Steam ID (optional)
    pub steam_id: Option<u64>,
    /// Eindeutige Spieler-ID (UUID oder Steam ID)
    pub player_id: String,
}

impl PlayerIdentity {
    /// Erstellt eine lokale Identität ohne Steam
    pub fn local(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            steam_id: None,
            player_id: format!("local:{}", name),
        }
    }

    /// Erstellt eine Steam-Identität
    pub fn steam(steam_id: u64, name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            display_name: name,
            steam_id: Some(steam_id),
            player_id: format!("steam:{}", steam_id),
        }
    }

    /// Prüft ob diese Identität von Steam kommt
    pub fn is_steam(&self) -> bool {
        self.steam_id.is_some()
    }
}

/// Event, das ausgelöst wird wenn sich die Identität ändert
#[derive(Event, Clone, Debug)]
pub struct IdentityChanged {
    pub old: PlayerIdentity,
    pub new: PlayerIdentity,
}

/// Plugin that registers the `IdentityChanged` event and triggers it whenever
/// `PlayerIdentity` is mutated.
pub struct ChickenIdentityPlugin;

impl Plugin for ChickenIdentityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, detect_identity_changed);
    }
}

fn detect_identity_changed(
    identity: Option<Res<PlayerIdentity>>,
    mut last: Local<Option<PlayerIdentity>>,
    mut commands: Commands,
) {
    let Some(identity) = identity else { return };

    if identity.is_changed() {
        let new = identity.clone();
        if let Some(old) = last.take() {
            commands.trigger(IdentityChanged {
                old,
                new: new.clone(),
            });
        }
        *last = Some(new);
    } else if last.is_none() {
        *last = Some(identity.clone());
    }
}
