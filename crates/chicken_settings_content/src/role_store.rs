use bevy_ecs::prelude::*;
use chicken_settings::Settings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Spieler-Rolle auf einem Server.
/// Reihenfolge ist aufsteigend: Player < Moderator < Admin < Owner.
/// Wird als Component an die Client-Entity gehängt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Component)]
#[serde(rename_all = "lowercase")]
pub enum PlayerRole {
    /// Standard-Rolle ohne Moderations-Rechte.
    Player,
    /// Darf Kick und Temp-Ban ausführen.
    Moderator,
    /// Darf Kick, Ban, Temp-Ban und bis Moderator promoten.
    Admin,
    /// Volle Kontrolle — automatisch vergeben an den ersten Spieler (Server-Ersteller).
    Owner,
}

impl Default for PlayerRole {
    fn default() -> Self {
        Self::Player
    }
}

/// Persistierte Rollentabelle: `player_id → PlayerRole`.
/// Wird aus `server/roles.toml` geladen. RAM ist Source of Truth.
#[derive(Settings, Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[settings("server/roles.toml")]
pub struct PlayerRoles {
    /// Zuordnung SHA-256(public_key) → Rolle.
    pub roles: HashMap<String, PlayerRole>,
}

impl Default for PlayerRoles {
    fn default() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }
}

impl PlayerRoles {
    /// Gibt die Rolle eines Spielers zurück. Fällt auf `Player` zurück falls nicht bekannt.
    pub fn get(&self, player_id: &str) -> PlayerRole {
        self.roles
            .get(player_id)
            .copied()
            .unwrap_or(PlayerRole::Player)
    }

    /// Gibt true zurück wenn kein Owner in der Tabelle eingetragen ist.
    pub fn has_no_owner(&self) -> bool {
        !self.roles.values().any(|r| *r == PlayerRole::Owner)
    }
}
