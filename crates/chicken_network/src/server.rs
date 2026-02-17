pub(crate) mod discovery;

#[cfg(feature = "client")]
use chicken_states::SingleplayerStatus;

#[cfg(feature = "server")]
use exitcodes::ExitCode;

use crate::shared::PlayerNameMessage;

use {
    aeronet::io::{
        connection::{Disconnect, Disconnected},
        server::{Close, Closed, Server, ServerEndpoint},
    },
    aeronet_io::connection::DisconnectReason,
    aeronet_replicon::server::AeronetRepliconServer,
    aeronet_webtransport::{
        cert,
        server::{
            SessionRequest, WebTransportServer, WebTransportServerClient, WebTransportServerPlugin,
        },
    },
    bevy::app::AppExit,
    bevy::prelude::*,
    bevy_replicon::{server::ServerSystems, shared::message::client_message::FromClient},
    chicken_notifications::Notify,
    chicken_settings_content::networking::NetworkingSettings,
    chicken_states::{ServerVisibility, SetServerVisibility},
    core::time::Duration,
    discovery::DiscoveryServerPlugin,
};

pub(crate) struct ServerLogicPlugin;

impl Plugin for ServerLogicPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "client")]
        app.add_systems(
            Update,
            server_pending_going_public.run_if(in_state(ServerVisibility::PendingPublic)),
        );

        app.add_plugins((WebTransportServerPlugin, DiscoveryServerPlugin))
            .add_systems(
                OnEnter(ServerVisibility::GoingPublic),
                on_server_going_public,
            )
            .add_systems(OnEnter(ServerVisibility::Public), on_server_is_running)
            .add_systems(
                Update,
                server_is_running.run_if(in_state(ServerVisibility::Public)),
            )
            .add_systems(
                OnEnter(ServerVisibility::GoingPrivate),
                on_server_going_private,
            )
            .add_systems(
                PreUpdate,
                receive
                    .after(ServerSystems::Receive)
                    .run_if(in_state(ServerVisibility::Public)),
            )
            .add_systems(Last, on_app_exit_notify_clients);
    }
}

fn receive(mut pings: MessageReader<FromClient<PlayerNameMessage>>, mut commands: Commands) {
    for ping in pings.read() {
        info!(
            "received ping from client `{}` with name `{}`",
            ping.client_id, ping.message.player_name
        );
        let Some(entity) = ping.client_id.entity() else {
            error!("Received ping from invalid client ID");
            return;
        };
        commands
            .entity(entity)
            .insert(Name::new(ping.message.player_name.clone()));
    }
}

#[cfg(feature = "client")]
fn server_pending_going_public(
    mut commands: Commands,
    singleplayer_state: Res<State<SingleplayerStatus>>,
) {
    if *singleplayer_state.get() == SingleplayerStatus::Running {
        {
            info!("Singleplayer Running detected, requesting Public transition.");
            commands.trigger(SetServerVisibility {
                transition: ServerVisibility::GoingPublic,
            });
        }
    }
}

fn on_server_going_public(
    mut commands: Commands,
    mut server_settings: ResMut<NetworkingSettings>,
    #[cfg(feature = "server")] mut app_exit: MessageWriter<AppExit>,
) {
    // TODO: implement error if server cant get started

    // TODO: Implement User interface infos for server

    // Implement Port usage detection
    #[cfg(feature = "server")]
    if !helpers::ports::is_port_available(server_settings.port) {
        commands.trigger(Notify::error(format!(
            "Game Port {} is already in use, please choose another port.",
            server_settings.port
        )));
        app_exit.write(AppExit::Error(ExitCode::BindPortFailed.nonzero()));
        return;
    }

    #[cfg(feature = "client")]
    match helpers::ports::find_free_port(server_settings.port, 100) {
        Some(port) => {
            server_settings.port = port;
        }
        None => {
            commands.trigger(Notify::error(format!("Failed to find a free Game port. We start searching for a free Discovery port between {} and {}. Please try another Port not in this range. If this Error persists, please try to restart your System.", server_settings.port, server_settings.port + 100)));
        }
    }

    if server_settings.can_be_discovered {
        match helpers::ports::is_port_available(server_settings.discovery_port) {
            true => {}
            false => {
                server_settings.can_be_discovered = false;
                #[cfg(feature = "client")]
                commands.trigger(Notify::error(
                    "DiscoveryPort is already in use. Discovering of this session is disabled!",
                ));
                #[cfg(feature = "server")]
                commands.trigger(Notify::error(
                    "DiscoveryPort is already in use. Discovering of this server is disabled! If this Error persists, please try to restart your System.",
                ));
            }
        }
    }

    let identity =
        aeronet_webtransport::wtransport::Identity::self_signed(["localhost", "127.0.0.1", "::1"])
            .expect("all given SANs should be valid DNS names");
    let cert = &identity.certificate_chain().as_slice()[0];
    let _spki_fingerprint =
        cert::spki_fingerprint_b64(cert).expect("should be a valid certificate");
    let _cert_hash = cert::hash_to_b64(cert.hash());

    let config = aeronet_webtransport::wtransport::ServerConfig::builder()
        .with_bind_default(server_settings.port)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(1)))
        .max_idle_timeout(Some(Duration::from_secs(5)))
        .expect("should be a valid idle timeout")
        .build();

    commands
        .spawn((Name::new("WebTransportServer"), AeronetRepliconServer))
        .queue(WebTransportServer::open(config))
        .observe(on_server_is_public)
        .observe(on_check_is_server_private)
        .observe(on_server_session_request)
        .observe(on_server_client_disconnected);
}

fn on_server_is_public(_: On<Add, Server>, mut next_state: ResMut<NextState<ServerVisibility>>) {
    info!("WebTransport server is fully opened");
    next_state.set(ServerVisibility::Public);
}

fn on_server_is_running(_: Commands) {
    info!("WebTransport Server is running");
}

fn server_is_running(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
) {
    if server_query.is_empty() {
        {
            commands.trigger(SetServerVisibility {
                transition: ServerVisibility::GoingPrivate,
            });
        }
    }
}

fn on_server_going_private(
    mut commands: Commands,
    client_query: Query<Entity, With<WebTransportServerClient>>,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    info!(
        "Server going down\n Still {} clients connected\n Servers: {} active",
        client_query.iter().count(),
        server_query.iter().count()
    );
    if !client_query.is_empty() {
        {
            info!("Disconnect all clients");
            for client in client_query.iter() {
                {
                    commands.trigger(Disconnect::new(client, "Server closing"));
                }
            }
            return;
        }
    }
    if let Ok(server) = server_query.single() {
        {
            info!("Close server");
            commands.trigger(Close::new(server, "Server closing"));
            return;
        }
    }
    if client_query.is_empty() && server_query.is_empty() {
        {
            info!("Server is down");
            next_state.set(ServerVisibility::Private);
        }
    }
}

fn on_check_is_server_private(_: On<Closed>, mut commands: Commands) {
    info!("Closed is triggered");
    commands.trigger(SetServerVisibility {
        transition: ServerVisibility::Private,
    });
}

fn on_server_session_request(trigger: On<SessionRequest>, clients: Query<&ChildOf>) {
    let client = trigger.event_target();
    let Ok(&ChildOf(server)) = clients.get(client) else {
        return;
    };

    helpers::handle_server_accept_connection(client, server, trigger);
}

fn on_server_client_disconnected(
    trigger: On<Disconnected>,
    // Optional: Query for player data if needed
) {
    let client_entity = trigger.event_target();

    match &trigger.reason {
        DisconnectReason::ByPeer(reason) => {
            on_server_client_graceful_disconnect(client_entity, reason);
        }
        DisconnectReason::ByError(err) => {
            let err_msg = err.to_string();
            // Simple heuristic to distinguish timeout from other errors
            if err_msg.to_lowercase().contains("timed out") {
                on_server_client_timeout(client_entity, err_msg);
            } else {
                on_server_client_lost(client_entity, err_msg);
            }
        }
        DisconnectReason::ByUser(reason) => {
            info!("Client {client_entity} was kicked by server: {reason}");
        }
    }
}

fn on_server_client_timeout(client: Entity, msg: String) {
    warn!("Client {client} timed out: {msg}");
    // TODO: Cleanup player entity, notify others, etc.
}

fn on_server_client_lost(client: Entity, msg: String) {
    error!("Client {client} connection lost: {msg}");
    // TODO: Handle sudden connection loss (e.g. keep player data for reconnect)
}

fn on_server_client_graceful_disconnect(client: Entity, msg: &str) {
    info!("Client {client} left the game gracefully: {msg}");
    // TODO: Save player state, clean up entity immediately
}

fn on_app_exit_notify_clients(
    mut app_exit_events: MessageReader<AppExit>,
    mut commands: Commands,
    client_query: Query<Entity, With<WebTransportServerClient>>,
) {
    if app_exit_events.read().next().is_some() {
        let client_count = client_query.iter().count();
        if client_count > 0 {
            info!("Notifying {} clients of server shutdown", client_count);
            for client in client_query.iter() {
                commands.trigger(Disconnect::new(client, "Server shutting down"));
            }
        }
    }
}

pub mod helpers {
    use {
        aeronet_webtransport::server::{SessionRequest, SessionResponse},
        bevy::prelude::*,
    };

    pub(super) fn handle_server_accept_connection(
        client: Entity,
        server: Entity,
        mut trigger: On<SessionRequest>,
    ) {
        info!("{client} connecting to {server} with headers:");
        for (header_key, header_value) in &trigger.headers {
            info!("  {header_key}: {header_value}");
        }

        trigger.respond(SessionResponse::Accepted);
    }

    /// Get the local IP address of the server.
    pub fn get_local_ip() -> Option<std::net::IpAddr> {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        socket.local_addr().ok().map(|addr| addr.ip())
    }

    pub(super) fn _handle_server_reject_connection() {
        // TODO: client UUID or Name is on the server's blacklist
        // TODO: Server password is incorrect
        // TODO: Server is full
        todo!("Implement on_server_shutdown_notify_clients")
    }

    pub(crate) mod ports {
        // https://en.wikipedia.org/wiki/List_of_TCP_and_UDP_port_numbers#Registered_ports
        use std::net::UdpSocket;

        #[allow(dead_code)]
        pub(crate) fn is_port_available(port: u16) -> bool {
            if UdpSocket::bind(("0.0.0.0", port)).is_err() {
                return false;
            }
            true
        }

        pub(in crate::server) fn find_free_port(
            start_port: u16,
            max_attempts: usize,
        ) -> Option<u16> {
            (0..max_attempts).find_map(|i| {
                let port = start_port + i as u16;
                if port > u16::MAX {
                    return None;
                }
                if is_port_available(port) {
                    Some(port)
                } else {
                    None
                }
            })
        }
    }
}
