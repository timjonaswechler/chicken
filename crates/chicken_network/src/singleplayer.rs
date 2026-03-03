use {
    crate::local::*,
    aeronet::io::{connection::Disconnect, server::Close},
    aeronet_channel::{ChannelIo, ChannelIoPlugin},
    // aeronet_replicon::client::AeronetRepliconClientPlugin,
    // aeronet_replicon::server::AeronetRepliconServerPlugin,
    aeronet_webtransport::server::{WebTransportServer, WebTransportServerClient},
    bevy::prelude::*,
    // bevy_replicon::RepliconPlugins,
    chicken_states::states::session::{
        SessionType, SetSingleplayerShutdownStep, SetSingleplayerStatus, SingleplayerShutdownStep,
        SingleplayerStatus,
    },
};

pub(crate) struct SingleplayerLogicPlugin;

impl Plugin for SingleplayerLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChannelIoPlugin,
            // RepliconPlugins,
            // AeronetRepliconClientPlugin,
            // AeronetRepliconServerPlugin,
        ))
        .add_systems(
            OnEnter(SingleplayerStatus::Starting),
            on_singleplayer_starting,
        )
        .add_observer(on_singleplayer_ready)
        .add_systems(
            OnEnter(SingleplayerStatus::Running),
            on_singleplayer_running,
        )
        .add_systems(
            Update,
            singleplayer_stopping.run_if(in_state(SingleplayerStatus::Stopping)),
        );
    }
}

fn on_singleplayer_starting(mut commands: Commands) {
    info!("Starting Singleplayer");

    let server_entity = commands
        .spawn((Name::new("Local Server"), LocalSession, LocalServer))
        .id();
    let client_entity = commands
        .spawn((Name::new("Local Client"), LocalSession, LocalClient))
        .id();

    commands.queue(ChannelIo::open(server_entity, client_entity));
}

fn on_singleplayer_ready(
    _: On<Add, LocalClient>,
    mut commands: Commands,
    current_state: Res<State<SessionType>>,
) {
    if *current_state.get() == SessionType::Singleplayer {
        commands.trigger(SetSingleplayerStatus {
            transition: SingleplayerStatus::Running,
        });
        info!("Singleplayer is ready");
    }
}

fn on_singleplayer_running(mut _commands: Commands) {
    debug!("Singleplayer is running");
}

fn singleplayer_stopping(
    mut commands: Commands,
    step: Res<State<SingleplayerShutdownStep>>,
    server_query: Query<Entity, With<WebTransportServer>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    local_client_query: Query<Entity, With<LocalClient>>,
    local_bot_query: Query<Entity, With<LocalBot>>,
    local_server_query: Query<Entity, With<LocalServer>>,
) {
    match step.get() {
        SingleplayerShutdownStep::DisconnectRemoteClients => {
            // 1. Tick: Remote-Clients trennen (public / LAN)
            for client in &client_query {
                commands.trigger(Disconnect::new(client, "Singleplayer closing"));
            }
            if client_query.is_empty() {
                commands.trigger(SetSingleplayerShutdownStep::Next);
            }
        }

        SingleplayerShutdownStep::CloseRemoteServer => {
            // 2. Tick: Remote-Server schließen (WebTransportServer)
            if let Ok(server_entity) = server_query.single() {
                commands.trigger(Close::new(server_entity, "Singleplayer closing"));
            } else if server_query.is_empty() {
                commands.trigger(SetSingleplayerShutdownStep::Next);
            }
        }

        SingleplayerShutdownStep::DespawnBots => {
            // 3. Tick: Lokale Bots despawnen
            for bot in &local_bot_query {
                if let Ok(mut bot_entity) = commands.get_entity(bot) {
                    bot_entity.despawn();
                }
            }
            if local_bot_query.is_empty() {
                commands.trigger(SetSingleplayerShutdownStep::Next);
            }
        }

        SingleplayerShutdownStep::DespawnLocalClient => {
            // 4. Tick: Lokalen Client despawnen
            if let Ok(client_entity) = local_client_query.single() {
                if let Ok(mut client_entity) = commands.get_entity(client_entity) {
                    client_entity.despawn();
                }
            } else if local_client_query.is_empty() {
                commands.trigger(SetSingleplayerShutdownStep::Next);
            }
        }

        SingleplayerShutdownStep::DespawnLocalServer => {
            // 5. Tick: Lokalen Server despawnen
            if let Ok(server_entity) = local_server_query.single() {
                if let Ok(mut server_entity) = commands.get_entity(server_entity) {
                    server_entity.despawn();
                }
            } else if local_server_query.is_empty() {
                commands.trigger(SetSingleplayerShutdownStep::Done);
            }
        }
    }
}
