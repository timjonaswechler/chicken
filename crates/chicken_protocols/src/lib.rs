// SPDX-License-Identifier: MIT
// Copyright (c) 2026 lib.rs
//
// Permission is hereby granted, free of charge, to any person obtaining a copy...

use {
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    serde::{Deserialize, Serialize},
    std::collections::{HashMap, VecDeque},
};

#[cfg(feature = "server")]
use chicken_states::states::session::ServerStatus;

// ─── Konstanten ──────────────────────────────────────────────────────────────

/// Server-seitige maximale Chat-History im RAM
pub const CHAT_HISTORY_SIZE: usize = 1024;
/// Maximale Anzahl Nachrichten die ein Client bei History-Request erhält
pub const CHAT_CLIENT_HISTORY_SIZE: usize = 128;
/// Maximale Zeichenlänge einer einzelnen Chat-Nachricht
pub const CHAT_MESSAGE_MAX_LENGTH: usize = 512;
/// Prefix-Zeichen für Chat-Commands (z.B. `/help`)
pub const CHAT_COMMAND_PREFIX: char = '/';
/// Prefix-Zeichen für Mentions (z.B. `@Spieler`)
pub const CHAT_MENTION_PREFIX: char = '@';

// ─── Plugin ──────────────────────────────────────────────────────────────────

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatHistory>()
            .init_resource::<ChatIdentities>()
            .init_resource::<ChatAutocompleteData>();
        app.add_client_message::<ClientChat>(Channel::Ordered)
            .add_client_message::<ClientChatHistoryRequest>(Channel::Ordered)
            .add_client_message::<ClientChatIdentity>(Channel::Ordered)
            .add_server_message::<ServerChat>(Channel::Ordered)
            .add_server_message::<ServerChatHistoryResponse>(Channel::Ordered)
            .add_server_message::<ServerChatError>(Channel::Ordered)
            .add_server_message::<ServerChatAutocomplete>(Channel::Ordered);

        #[cfg(feature = "server")]
        app.add_systems(
            Update,
            (
                handle_client_chat_identity,
                handle_client_chat,
                handle_client_chat_history_request,
            )
                .chain()
                .run_if(in_state(ServerStatus::Running)),
        )
        .add_systems(
            Update,
            broadcast_autocomplete_data.run_if(in_state(ServerStatus::Running)),
        );
    }
}

// ─── Client → Server ─────────────────────────────────────────────────────────

/// Nachricht, die ein Client an den Server sendet
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChat {
    pub text: String,
}

/// Client fordert die Chat-History an (z.B. nach dem Verbinden)
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChatHistoryRequest;

/// Client meldet seinen Anzeigenamen und optionale Steam-ID
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChatIdentity {
    pub name: String,
    pub steam_id: Option<u64>,
}

// ─── Server → Client ─────────────────────────────────────────────────────────

/// Broadcast-Nachricht mit einer einzelnen Chat-Zeile
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChat {
    pub sender_name: String,
    pub sender_steam_id: Option<u64>,
    pub text: String,
    /// Unix-Timestamp in Sekunden (gesetzt beim Empfang auf dem Server)
    pub timestamp: Option<u64>,
}

/// Antwort auf `ClientChatHistoryRequest`
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChatHistoryResponse {
    pub history: Vec<ServerChat>,
}

/// Fehler-Rückmeldung an den Client der die ungültige Nachricht gesendet hat
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChatError {
    pub error_type: ChatErrorType,
    pub message: String,
}

/// Autocomplete-Daten die der Server periodisch an alle Clients sendet
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChatAutocomplete {
    pub commands: Vec<ChatCommandInfo>,
    pub players: Vec<ChatPlayerInfo>,
}

// ─── Hilfstypen ──────────────────────────────────────────────────────────────

/// Klassifizierung eines Chat-Fehlers
#[derive(Message, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ChatErrorType {
    MessageTooLong,
    EmptyMessage,
    UnknownCommand,
}

/// Beschreibung eines verfügbaren Server-Commands für Autocomplete
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatCommandInfo {
    pub command: String,
    pub description: String,
    pub usage: String,
}

/// Spieler-Eintrag für @mention-Autocomplete
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatPlayerInfo {
    pub name: String,
    pub steam_id: Option<u64>,
}

// ─── Server-Ressourcen ───────────────────────────────────────────────────────

/// Chat-History im Server-RAM (begrenzt auf `CHAT_HISTORY_SIZE`)
#[derive(Resource, Default)]
pub struct ChatHistory(pub VecDeque<ServerChat>);

/// Zuordnung von `ClientId` → Anzeigename + Steam-ID
#[derive(Resource, Default)]
pub struct ChatIdentities(pub HashMap<ClientId, ChatIdentity>);

#[derive(Debug, Clone)]
pub struct ChatIdentity {
    pub name: String,
    pub steam_id: Option<u64>,
}

/// Serverseitig verwaltete Autocomplete-Daten (Commands + Spielerliste)
#[derive(Resource, Default)]
pub struct ChatAutocompleteData {
    pub commands: Vec<ChatCommandInfo>,
}

// ─── Server-Systeme ──────────────────────────────────────────────────────────

pub fn handle_client_chat(
    mut client_chat_events: MessageReader<FromClient<ClientChat>>,
    mut server_chat_events: MessageWriter<ToClients<ServerChat>>,
    mut error_events: MessageWriter<ToClients<ServerChatError>>,
    mut chat_history: ResMut<ChatHistory>,
    chat_identities: Res<ChatIdentities>,
) {
    for FromClient {
        client_id, message, ..
    } in client_chat_events.read()
    {
        let text = message.text.trim();

        // Validierung
        if text.is_empty() {
            error_events.write(ToClients {
                mode: SendMode::Direct(*client_id),
                message: ServerChatError {
                    error_type: ChatErrorType::EmptyMessage,
                    message: "Nachricht darf nicht leer sein.".to_string(),
                },
            });
            continue;
        }
        if text.len() > CHAT_MESSAGE_MAX_LENGTH {
            error_events.write(ToClients {
                mode: SendMode::Direct(*client_id),
                message: ServerChatError {
                    error_type: ChatErrorType::MessageTooLong,
                    message: format!(
                        "Nachricht zu lang ({} / {} Zeichen).",
                        text.len(),
                        CHAT_MESSAGE_MAX_LENGTH
                    ),
                },
            });
            continue;
        }

        // TODO: Command-Parsing via extract_command() implementieren
        if extract_command(text).is_some() {
            error_events.write(ToClients {
                mode: SendMode::Direct(*client_id),
                message: ServerChatError {
                    error_type: ChatErrorType::UnknownCommand,
                    message: "Chat-Commands sind noch nicht implementiert.".to_string(),
                },
            });
            continue;
        }

        let identity = chat_identities.0.get(&client_id);
        let sender_name = identity
            .map(|e| e.name.clone())
            .unwrap_or_else(|| format!("Client {client_id:?}"));
        let sender_steam_id = identity.and_then(|e| e.steam_id);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());

        let server_chat = ServerChat {
            sender_name,
            sender_steam_id,
            text: text.to_string(),
            timestamp: now,
        };

        server_chat_events.write(ToClients {
            mode: SendMode::Broadcast,
            message: server_chat.clone(),
        });

        chat_history.0.push_back(server_chat);
        if chat_history.0.len() > CHAT_HISTORY_SIZE {
            chat_history.0.pop_front();
        }
    }
}

pub fn handle_client_chat_identity(
    mut identity_events: MessageReader<FromClient<ClientChatIdentity>>,
    mut chat_identities: ResMut<ChatIdentities>,
) {
    for FromClient {
        client_id, message, ..
    } in identity_events.read()
    {
        chat_identities.0.insert(
            *client_id,
            ChatIdentity {
                name: message.name.clone(),
                steam_id: message.steam_id,
            },
        );
    }
}

pub fn handle_client_chat_history_request(
    mut history_requests: MessageReader<FromClient<ClientChatHistoryRequest>>,
    mut server_chat_events: MessageWriter<ToClients<ServerChatHistoryResponse>>,
    chat_history: Res<ChatHistory>,
    chat_identities: Res<ChatIdentities>,
) {
    for FromClient { client_id, .. } in history_requests.read() {
        let own_name = chat_identities
            .0
            .get(&client_id)
            .map(|e| e.name.clone())
            .unwrap_or_default();

        let history = filter_relevant_chat_history(&chat_history.0, &own_name);

        server_chat_events.write(ToClients {
            mode: SendMode::Direct(*client_id),
            message: ServerChatHistoryResponse { history },
        });
    }
}

/// Sendet aktuelle Autocomplete-Daten (Commands + Spielerliste) an alle Clients.
/// Sollte periodisch oder bei Änderungen aufgerufen werden.
pub fn broadcast_autocomplete_data(
    mut autocomplete_events: MessageWriter<ToClients<ServerChatAutocomplete>>,
    autocomplete_data: Res<ChatAutocompleteData>,
    chat_identities: Res<ChatIdentities>,
) {
    let players: Vec<ChatPlayerInfo> = chat_identities
        .0
        .values()
        .map(|e| ChatPlayerInfo {
            name: e.name.clone(),
            steam_id: e.steam_id,
        })
        .collect();

    autocomplete_events.write(ToClients {
        mode: SendMode::Broadcast,
        message: ServerChatAutocomplete {
            commands: autocomplete_data.commands.clone(),
            players,
        },
    });
}

// ─── Hilfsfunktionen ─────────────────────────────────────────────────────────

/// Extrahiert den Command-Namen aus einer Nachricht (z.B. `/help` → `Some("help")`)
pub fn extract_command(text: &str) -> Option<&str> {
    text.strip_prefix(CHAT_COMMAND_PREFIX)
        .map(|s| s.split_whitespace().next().unwrap_or(""))
        .filter(|s| !s.is_empty())
}

/// Extrahiert alle @mentions aus einer Nachricht
pub fn extract_mentions(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .filter_map(|word| word.strip_prefix(CHAT_MENTION_PREFIX))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Filtert die Chat-History für einen bestimmten Spieler:
/// Priorisiert @mentions, füllt mit neuesten Nachrichten auf bis `CHAT_CLIENT_HISTORY_SIZE`.
fn filter_relevant_chat_history(history: &VecDeque<ServerChat>, own_name: &str) -> Vec<ServerChat> {
    let mention = format!("{}{}", CHAT_MENTION_PREFIX, own_name);

    let mut mentioned: Vec<&ServerChat> = history
        .iter()
        .filter(|m| m.text.contains(&mention))
        .collect();

    let mut result: Vec<ServerChat> = history
        .iter()
        .rev()
        .filter(|m| !m.text.contains(&mention))
        .take(CHAT_CLIENT_HISTORY_SIZE.saturating_sub(mentioned.len()))
        .cloned()
        .collect();

    result.reverse();
    mentioned.sort_by_key(|m| m.timestamp);
    result.extend(mentioned.into_iter().cloned());
    result.sort_by_key(|m| m.timestamp);
    result.dedup_by(|a, b| a.timestamp == b.timestamp && a.sender_name == b.sender_name);
    result
}
