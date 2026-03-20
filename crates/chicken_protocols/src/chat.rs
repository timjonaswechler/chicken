#[cfg(any(feature = "hosted", feature = "headless"))]
use {
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    serde::{Deserialize, Serialize},
};

use super::auth::*;

// ─── Konstanten ──────────────────────────────────────────────────────────────

/// Maximale Zeichenlänge einer einzelnen Chat-Nachricht
#[cfg(any(feature = "hosted", feature = "headless"))]
pub const CHAT_MESSAGE_MAX_LENGTH: usize = 512;
/// Prefix-Zeichen für Chat-Commands (z.B. `/help`)
#[cfg(any(feature = "hosted", feature = "headless"))]
pub const CHAT_COMMAND_PREFIX: char = '/';
/// Prefix-Zeichen für Mentions (z.B. `@Spieler`)
#[cfg(any(feature = "hosted", feature = "headless"))]
pub const CHAT_MENTION_PREFIX: char = '@';

// ─── Plugin ──────────────────────────────────────────────────────────────────

#[cfg(any(feature = "hosted", feature = "headless"))]
pub struct ProtocolPlugin;

#[cfg(any(feature = "hosted", feature = "headless"))]
impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_client_message::<ClientChat>(Channel::Ordered)
            .add_client_message::<ClientChatHistoryRequest>(Channel::Ordered)
            .add_server_message::<ServerChat>(Channel::Ordered)
            .add_server_message::<ServerChatHistoryResponse>(Channel::Ordered)
            .add_server_message::<ServerChatError>(Channel::Ordered)
            .add_server_message::<ServerChatAutocomplete>(Channel::Ordered);

        // Auth-Nachrichten registrieren
        app.add_client_message::<ClientIdentityHello>(Channel::Ordered)
            .add_client_message::<ClientAuthResponse>(Channel::Ordered)
            .add_server_message::<ServerAuthChallenge>(Channel::Ordered)
            .add_server_message::<ServerAuthResult>(Channel::Ordered);
    }
}

// ─── Client → Server ─────────────────────────────────────────────────────────

/// Nachricht, die ein Client an den Server sendet
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChat {
    pub text: String,
}

/// Client fordert die Chat-History an (z.B. nach dem Verbinden)
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChatHistoryRequest;

// ─── Server → Client ─────────────────────────────────────────────────────────

/// Broadcast-Nachricht mit einer einzelnen Chat-Zeile
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChat {
    pub sender_name: String,
    pub sender_steam_id: Option<u64>,
    pub text: String,
    /// Unix-Timestamp in Sekunden (gesetzt beim Empfang auf dem Server)
    pub timestamp: Option<u64>,
}

/// Antwort auf `ClientChatHistoryRequest`
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChatHistoryResponse {
    pub history: Vec<ServerChat>,
}

/// Fehler-Rückmeldung an den Client der die ungültige Nachricht gesendet hat
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChatError {
    pub error_type: ChatErrorType,
    pub message: String,
}

/// Autocomplete-Daten die der Server periodisch an alle Clients sendet
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChatAutocomplete {
    pub commands: Vec<ChatCommandInfo>,
    pub players: Vec<ChatPlayerInfo>,
}

// ─── Hilfstypen ──────────────────────────────────────────────────────────────

/// Klassifizierung eines Chat-Fehlers
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ChatErrorType {
    MessageTooLong,
    EmptyMessage,
    UnknownCommand,
}

/// Beschreibung eines verfügbaren Server-Commands für Autocomplete
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCommandInfo {
    pub command: String,
    pub description: String,
    pub usage: String,
}

/// Spieler-Eintrag für @mention-Autocomplete
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatPlayerInfo {
    pub name: String,
    pub steam_id: Option<u64>,
}

// ─── Hilfsfunktionen ─────────────────────────────────────────────────────────

/// Extrahiert den Command-Namen aus einer Nachricht (z.B. `/help` → `Some("help")`)
#[cfg(any(feature = "hosted", feature = "headless"))]
pub fn extract_command(text: &str) -> Option<&str> {
    text.strip_prefix(CHAT_COMMAND_PREFIX)
        .map(|s| s.split_whitespace().next().unwrap_or(""))
        .filter(|s| !s.is_empty())
}

/// Extrahiert alle @mentions aus einer Nachricht
#[cfg(any(feature = "hosted", feature = "headless"))]
pub fn extract_mentions(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .filter_map(|word| word.strip_prefix(CHAT_MENTION_PREFIX))
        .filter(|s| !s.is_empty())
        .collect()
}
