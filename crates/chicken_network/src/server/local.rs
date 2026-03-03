use crate::{LocalClient, LocalServer, LocalSession};

use {
    aeronet::io::{connection::Disconnect, server::Close},
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::{ecs::query::QuerySingleError, prelude::*},
    chicken_states::{
        ServerShutdownStep, ServerStartupStep, ServerStatus, SetServerShutdownStep,
        SetServerStartupStep,
    },
};

/// #Local Server

#[derive(Component)]
pub struct LocalBot;

pub(crate) struct LocalServerPlugin;

impl Plugin for LocalServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChannelIoPlugin)
            .add_systems(OnEnter(ServerStatus::Starting), server_starting)
            .add_systems(OnEnter(ServerStatus::Running), server_running)
            .add_systems(
                Update,
                server_stopping.run_if(in_state(ServerStatus::Stopping)),
            );
    }
}

fn server_starting(
    mut commands: Commands,
    step: Res<State<ServerStartupStep>>,
    server_query: Query<Entity, (With<LocalServer>, With<LocalSession>)>,
    client_query: Query<Entity, (With<LocalClient>, With<LocalSession>)>,
    bot_query: Query<Entity, (With<LocalBot>, With<LocalSession>)>,
) {
    match step.get() {
        ServerStartupStep::Init => {
            commands.spawn((Name::new("Local Server"), LocalSession, LocalServer));

            match server_query.single() {
                Ok(_) => commands.trigger(SetServerStartupStep::Next),
                Err(QuerySingleError::NoEntities(_)) => {
                    error!("Error: There is no LocalServer right now!");
                    todo!("Handle this Error properly");
                }
                Err(QuerySingleError::MultipleEntities(_)) => {
                    error!("Error: There is more than one LocalServer!");
                    todo!("Handle this Error properly");
                }
            }
        }

        ServerStartupStep::LoadWorld => commands.trigger(SetServerStartupStep::Next),

        ServerStartupStep::SpawnEntities => {
            #[cfg(feature = "client")]
            let client_entity = commands
                .spawn((Name::new("Local Client"), LocalSession, LocalClient))
                .id();

            #[cfg(feature = "client")]
            let server = match server_query.single() {
                Ok(server) => server,
                Err(QuerySingleError::NoEntities(_)) => {
                    error!("Error: No Server found!");
                    commands.trigger(SetServerStartupStep::Failed);
                    return;
                }
                Err(QuerySingleError::MultipleEntities(_)) => {
                    error!("Error: There is more than one LocalServer!");
                    commands.trigger(SetServerStartupStep::Failed);
                    return;
                }
            };
            #[cfg(feature = "client")]
            commands.queue(ChannelIo::open(server, client_entity));

            commands.trigger(SetServerStartupStep::Next);
        }

        ServerStartupStep::Ready => {
            #[cfg(feature = "client")]
            if client_query.count() == 1 {
                commands.trigger(SetServerStartupStep::Done);
            }

            #[cfg(not(feature = "client"))]
            if server_query.count() == 1 {
                commands.trigger(SetServerStartupStep::Done);
            }
        }
    }
}

fn server_running() {
    info!("Local server started");
}

fn server_stopping(
    mut commands: Commands,
    step: Res<State<ServerShutdownStep>>,
    server_query: Query<Entity, With<WebTransportServer>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    local_client_query: Query<Entity, With<LocalClient>>,
    local_bot_query: Query<Entity, With<LocalBot>>,
    local_server_query: Query<Entity, With<LocalServer>>,
) {
    match step.get() {
        ServerShutdownStep::SaveWorld => {
            commands.trigger(SetServerShutdownStep::Next);
        }

        ServerShutdownStep::DisconnectClients => {
            for client in &client_query {
                commands.trigger(Disconnect::new(client, "Singleplayer closing"));
            }
            if client_query.is_empty() {
                commands.trigger(SetServerShutdownStep::Next);
            }
        }

        ServerShutdownStep::DespawnLocalClient => {
            for bot in &local_bot_query {
                if let Ok(mut bot_entity) = commands.get_entity(bot) {
                    bot_entity.despawn();
                }
            }
            if let Ok(client_entity) = local_client_query.single() {
                if let Ok(mut client_entity) = commands.get_entity(client_entity) {
                    client_entity.despawn();
                }
            } else if local_client_query.is_empty() && local_bot_query.is_empty() {
                commands.trigger(SetServerShutdownStep::Next);
            }
        }

        ServerShutdownStep::Cleanup => {
            //TODO: Go through it and  create a clean structure
            if let Ok(server_entity) = server_query.single() {
                commands.trigger(Close::new(server_entity, "Singleplayer closing"));
            } else if let Ok(server_entity) = local_server_query.single() {
                if let Ok(mut server_entity) = commands.get_entity(server_entity) {
                    server_entity.despawn();
                }
            } else if server_query.is_empty() && local_server_query.is_empty() {
                commands.trigger(SetServerShutdownStep::Done);
            }
        }
    }
}
