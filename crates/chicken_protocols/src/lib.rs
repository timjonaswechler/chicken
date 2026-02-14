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

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatHistory>()
            .init_resource::<ChatIdentities>();
        // Wir versuchen es ohne expliziten Pfad, falls es im Prelude ist,
        // oder nutzen u8 falls es eine ID ist (eher unwahrscheinlich)
        app.add_client_message::<ClientChat>(Channel::Ordered)
            .add_client_message::<ClientChatHistoryRequest>(Channel::Ordered)
            .add_client_message::<ClientChatIdentity>(Channel::Ordered)
            .add_server_message::<ServerChat>(Channel::Ordered)
            .add_server_message::<ServerChatHistoryResponse>(Channel::Ordered);
    }
}

const CHAT_HISTORY_SIZE: usize = 1;

/// If a new Client connects, we send them the current chat history the client is supposed to receive.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChatHistoryRequest;

/// If the client requests the chat history, we send them the messages the client is supposed to receive.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChatHistoryResponse {
    pub history: Vec<ServerChat>,
}

/// Client identity for chat display (name + optional Steam ID).
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChatIdentity {
    pub name: String,
    pub steam_id: Option<u64>,
}

/// Chat history with the length of `CHAT_HISTORY_SIZE` saved in the server RAM.
#[derive(Resource, Default)]
pub struct ChatHistory(VecDeque<ServerChat>);

#[derive(Debug, Clone)]
pub struct ChatIdentity {
    pub name: String,
    pub steam_id: Option<u64>,
}

#[derive(Resource, Default)]
pub struct ChatIdentities(HashMap<ClientId, ChatIdentity>);

/// Nachricht, die ein Client an den Server sendet
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientChat {
    pub text: String,
}

/// Nachricht, die der Server an alle (oder bestimmte) Clients sendet
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerChat {
    pub sender_name: String,
    pub sender_steam_id: Option<u64>,
    pub text: String,
}

pub fn handle_client_chat(
    mut client_chat_events: MessageReader<FromClient<ClientChat>>,
    mut server_chat_events: MessageWriter<ToClients<ServerChat>>,
    mut chat_history: ResMut<ChatHistory>,
    chat_identities: Res<ChatIdentities>,
) {
    for FromClient {
        client_id, message, ..
    } in client_chat_events.read()
    {
        let identity = chat_identities.0.get(&client_id);
        let sender_name = identity
            .map(|entry| entry.name.clone())
            .unwrap_or_else(|| format!("Client {client_id:?}"));
        let sender_steam_id = identity.and_then(|entry| entry.steam_id);

        let server_chat = ServerChat {
            sender_name,
            sender_steam_id,
            text: message.text.clone(),
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
) {
    for FromClient { client_id, .. } in history_requests.read() {
        server_chat_events.write(ToClients {
            mode: SendMode::Direct(*client_id),
            message: ServerChatHistoryResponse {
                history: chat_history.0.iter().cloned().collect(),
            },
        });
    }
}
