use {
    crate::server::auth::PlayerRegistry,
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    chicken_protocols::{
        CHAT_MENTION_PREFIX, CHAT_MESSAGE_MAX_LENGTH, ChatCommandInfo, ChatErrorType,
        ChatPlayerInfo, ClientChat, ClientChatHistoryRequest, ServerChat, ServerChatAutocomplete,
        ServerChatError, ServerChatHistoryResponse, extract_command,
    },
    chicken_states::states::session::ServerStatus,
    std::collections::VecDeque,
};

pub(crate) struct ServerChatPlugin;

/// Server-seitige maximale Chat-History im RAM
pub const CHAT_HISTORY_SIZE: usize = 1024;
/// Maximale Anzahl Nachrichten die ein Client bei History-Request erhält
pub const CHAT_CLIENT_HISTORY_SIZE: usize = 128;

impl Plugin for ServerChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatHistory>()
            .init_resource::<ChatAutocompleteData>()
            .add_systems(
                Update,
                (handle_client_chat, handle_client_chat_history_request)
                    .chain()
                    .run_if(in_state(ServerStatus::Running)),
            )
            .add_systems(
                Update,
                broadcast_autocomplete_data.run_if(in_state(ServerStatus::Running)),
            );
    }
}

/// Chat-History im Server-RAM (begrenzt auf `CHAT_HISTORY_SIZE`)
#[derive(Resource, Default)]
pub struct ChatHistory(pub VecDeque<ServerChat>);

/// Serverseitig verwaltete Autocomplete-Daten (Commands + Spielerliste)
#[derive(Resource, Default)]
pub struct ChatAutocompleteData {
    pub commands: Vec<ChatCommandInfo>,
}

fn handle_client_chat(
    mut client_chat_events: MessageReader<FromClient<ClientChat>>,
    mut server_chat_events: MessageWriter<ToClients<ServerChat>>,
    mut error_events: MessageWriter<ToClients<ServerChatError>>,
    mut chat_history: ResMut<ChatHistory>,
    player_registry: Res<PlayerRegistry>,
) {
    for FromClient {
        client_id, message, ..
    } in client_chat_events.read()
    {
        let text = message.text.trim();

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

        let player = player_registry.0.get(client_id);
        let sender_name = player
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| format!("Client {client_id:?}"));
        let sender_steam_id = player.and_then(|p| p.steam_id);

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

fn handle_client_chat_history_request(
    mut history_requests: MessageReader<FromClient<ClientChatHistoryRequest>>,
    mut server_chat_events: MessageWriter<ToClients<ServerChatHistoryResponse>>,
    chat_history: Res<ChatHistory>,
    player_registry: Res<PlayerRegistry>,
) {
    for FromClient { client_id, .. } in history_requests.read() {
        let own_name = player_registry
            .0
            .get(client_id)
            .map(|p| p.display_name.clone())
            .unwrap_or_default();

        let history = filter_relevant_chat_history(&chat_history.0, &own_name);

        server_chat_events.write(ToClients {
            mode: SendMode::Direct(*client_id),
            message: ServerChatHistoryResponse { history },
        });
    }
}

fn broadcast_autocomplete_data(
    mut autocomplete_events: MessageWriter<ToClients<ServerChatAutocomplete>>,
    autocomplete_data: Res<ChatAutocompleteData>,
    player_registry: Res<PlayerRegistry>,
) {
    // Deduplizierung nach player_id — verhindert Doppeleinträge wenn
    // derselbe Spieler als ClientId::Server und ClientId::Client registriert ist.
    let mut seen_ids = std::collections::HashSet::new();
    let players: Vec<ChatPlayerInfo> = player_registry
        .0
        .values()
        .filter(|p| seen_ids.insert(p.player_id.clone()))
        .map(|p| ChatPlayerInfo {
            name: p.display_name.clone(),
            steam_id: p.steam_id,
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
