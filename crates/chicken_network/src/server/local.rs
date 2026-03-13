use {
    aeronet::io::{connection::Disconnect, server::Close},
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::{ecs::query::QuerySingleError, prelude::*},
    chicken_states::{
        events::session::{SetServerShutdownStep, SetServerStartupStep},
        logic::session::server::PendingGoingPublic,
        states::session::{ServerShutdownStep, ServerStartupStep, ServerStatus, ServerVisibility},
    },
};

/// Marker component for entities participating in a local singleplayer session.
///
/// Used to identify and query entities that are part of the local client-server loop
/// in singleplayer mode, distinguishing them from networked multiplayer entities.
#[derive(Component)]
pub struct LocalSession;

/// Marker component for the local client entity in singleplayer mode.
///
/// Attached to the client entity that connects to the local server via in-memory
/// channels. Used to identify local client-specific systems and resources.
#[derive(Component)]
pub struct LocalClient;

/// Marker component for the local server entity in singleplayer mode.
///
/// Attached to the server entity that hosts the game locally via in-memory
/// channels. Used to identify local server-specific systems and resources.
#[derive(Component)]
pub struct LocalServer;

/// Marker component for AI-controlled bot entities.
///
/// Used to distinguish bot players from human players in both singleplayer
/// and multiplayer sessions. Bots may be spawned locally or replicated from server.
#[derive(Component)]
pub struct LocalBot;

/// #Local Server

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

fn server_running(
    marker: Option<Res<PendingGoingPublic>>,
    mut next_visibility: ResMut<NextState<ServerVisibility>>,
    mut commands: Commands,
) {
    if marker.is_some() {
        next_visibility.set(ServerVisibility::GoingPublic);
        commands.remove_resource::<PendingGoingPublic>();
        info!("Server automatically going public");
    }
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
                commands.trigger(SetServerShutdownStep::Next);
            }
        }
        ServerShutdownStep::Ready => {
            commands.trigger(SetServerShutdownStep::Done);
        }
    }
}
