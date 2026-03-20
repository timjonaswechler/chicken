use chicken_settings::Settings;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Server-Zugangskontrolle: Passwort, Whitelist, Blacklist, Kapazität.
/// Wird beim Start aus `server/access.toml` geladen.
/// RAM ist die Source of Truth — Änderungen werden direkt in die Resource geschrieben
/// und danach zur Persistenz in die Datei gespeichert.
#[derive(Settings, Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[settings("server/access.toml")]
pub struct ServerAccessSettings {
    /// Ob der Server passwortgeschützt ist.
    pub password_protected: bool,
    /// SHA-256 Hex-Hash des Server-Passworts. None falls kein Passwort gesetzt.
    pub password_hash: Option<String>,
    /// Maximale Spieleranzahl. -1 = unbegrenzt.
    pub max_players: i32,
    /// Whitelist der erlaubten Display-Namen. Leer = deaktiviert (alle erlaubt).
    pub whitelist: Vec<String>,
    /// Liste gesperrter Spieler.
    pub blacklist: Vec<BlacklistEntry>,
}

impl Default for ServerAccessSettings {
    fn default() -> Self {
        Self {
            password_protected: false,
            password_hash: None,
            max_players: -1,
            whitelist: Vec::new(),
            blacklist: Vec::new(),
        }
    }
}

/// Eintrag in der Blacklist.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlacklistEntry {
    /// SHA-256 Hash des öffentlichen Ed25519-Schlüssels (stabile Spieler-ID).
    pub player_id: String,
    /// Optionale Steam-ID für zusätzliche Prüfung.
    pub steam_id: Option<u64>,
    /// Lesbarer Name — nur zur Orientierung, wird nicht für den Match verwendet.
    pub display_name: Option<String>,
}
