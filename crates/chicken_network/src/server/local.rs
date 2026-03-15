use {
    aeronet::io::{connection::Disconnect, server::Close},
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::{ecs::query::QuerySingleError, prelude::*},
    chicken_notifications::Notify,
    chicken_states::{
        events::session::{SetGoingPublicStep, SetServerShutdownStep, SetServerStartupStep},
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
            .add_systems(OnEnter(ServerStartupStep::Init), server_starting_init)
            .add_systems(
                Update,
                server_starting_load_world.run_if(in_state(ServerStartupStep::LoadWorld)),
            );

        #[cfg(feature = "client")]
        app.add_systems(
            OnEnter(ServerStartupStep::SpawnEntities),
            server_starting_spawn_entities,
        );

        app.add_systems(OnEnter(ServerStartupStep::Ready), server_starting_ready)
            .add_systems(OnEnter(ServerStatus::Running), server_running)
            .add_systems(
                OnEnter(ServerShutdownStep::SaveWorld),
                server_stopping_save_world,
            )
            .add_systems(
                Update,
                server_stopping_disconnect_clients
                    .run_if(in_state(ServerShutdownStep::DisconnectClients)),
            );

        #[cfg(feature = "client")]
        app.add_systems(
            Update,
            server_stopping_despawn_local_client
                .run_if(in_state(ServerShutdownStep::DespawnLocalClient)),
        )
        .add_systems(
            Update,
            server_stopping_cleanup.run_if(in_state(ServerShutdownStep::Cleanup)),
        )
        .add_systems(OnEnter(ServerShutdownStep::Ready), server_stopping_ready);
    }
}

fn server_starting_init(mut commands: Commands) {
    commands.spawn((Name::new("Local Server"), LocalSession, LocalServer));
    commands.trigger(SetServerStartupStep::Next);
}

fn server_starting_load_world(
    mut commands: Commands,
    server_query: Query<Entity, (With<LocalServer>, With<LocalSession>)>,
) {
    match server_query.single() {
        Ok(_) => commands.trigger(SetServerStartupStep::Next),
        Err(QuerySingleError::NoEntities(_)) => {
            commands.trigger(Notify::error(
                "There is no LocalServer right now! - this is a bug",
            ));
            commands.trigger(SetServerStartupStep::Failed);
            return;
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            commands.trigger(Notify::error(
                "There is more than one LocalServer! - this is a bug",
            ));
            commands.trigger(SetServerStartupStep::Failed);
            return;
        }
    }
}

fn server_starting_spawn_entities(
    mut commands: Commands,
    server_query: Query<Entity, (With<LocalServer>, With<LocalSession>)>,
) {
    let client_entity = commands
        .spawn((Name::new("Local Client"), LocalSession, LocalClient))
        .id();

    let server = match server_query.single() {
        Ok(server) => server,
        _ => {
            commands.trigger(Notify::error(
                "To spawn the local entity, no local server was found! - this is a bug",
            ));
            commands.trigger(SetServerStartupStep::Failed);
            return;
        }
    };
    commands.queue(ChannelIo::open(server, client_entity));
    commands.trigger(SetServerStartupStep::Next);
}

fn server_starting_ready(
    mut commands: Commands,
    #[cfg(feature = "server")] server_query: Query<Entity, (With<LocalServer>, With<LocalSession>)>,
    #[cfg(feature = "client")] client_query: Query<Entity, (With<LocalClient>, With<LocalSession>)>,
    #[cfg(feature = "client")] bot_query: Query<Entity, (With<LocalBot>, With<LocalSession>)>,
) {
    #[cfg(feature = "client")]
    if client_query.count() == 1 {
        commands.trigger(SetServerStartupStep::Done);
    }

    #[cfg(feature = "server")]
    if server_query.count() == 1 {
        commands.trigger(SetServerStartupStep::Done);
    }
}

fn server_running(marker: Option<Res<PendingGoingPublic>>, mut commands: Commands) {
    if marker.is_some() {
        commands.trigger(SetGoingPublicStep::Start);
        commands.remove_resource::<PendingGoingPublic>();
        info!("Server automatically going public");
        return;
    }
    info!("Local server started");
}

fn server_stopping_save_world(mut commands: Commands) {
    commands.trigger(SetServerShutdownStep::Next);
}

fn server_stopping_disconnect_clients(
    mut commands: Commands,
    client_query: Query<Entity, With<WebTransportServerClient>>,
) {
    for client in &client_query {
        commands.trigger(Disconnect::new(client, "Singleplayer closing"));
    }
    if client_query.is_empty() {
        commands.trigger(SetServerShutdownStep::Next);
    }
}

fn server_stopping_despawn_local_client(
    mut commands: Commands,
    local_client_query: Query<Entity, With<LocalClient>>,
    local_bot_query: Query<Entity, With<LocalBot>>,
) {
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

fn server_stopping_cleanup(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    local_server_query: Query<Entity, With<LocalServer>>,
) {
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

fn server_stopping_ready(mut commands: Commands) {
    commands.trigger(SetServerShutdownStep::Done);
}
