use {
    super::networking::{address::helpers::accept_session_request, ports},
    aeronet::io::{connection::Disconnect, server::Close},
    aeronet_replicon::server::AeronetRepliconServer,
    aeronet_webtransport::{
        cert,
        server::{WebTransportServer, WebTransportServerClient, WebTransportServerPlugin},
    },
    bevy::{ecs::query::QuerySingleError, prelude::*},
    chicken_notifications::Notify,
    chicken_settings_content::networking::NetworkingSettings,
    chicken_states::{
        events::session::{SetGoingPrivateStep, SetGoingPublicStep},
        states::session::{GoingPrivateStep, GoingPublicStep},
    },
    core::time::Duration,
};

pub(crate) struct QUICServerPlugin;

impl Plugin for QUICServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WebTransportServerPlugin)
            .add_observer(accept_session_request)
            .add_systems(
                OnEnter(GoingPublicStep::Validating),
                server_going_public_validating,
            )
            .add_systems(
                OnEnter(GoingPublicStep::StartingServer),
                server_going_public_starting_server,
            )
            .add_systems(
                Update,
                server_going_public_starting_discovery
                    .run_if(in_state(GoingPublicStep::StartingDiscovery)),
            )
            .add_systems(OnEnter(GoingPublicStep::Ready), server_going_public_ready)
            .add_systems(
                OnEnter(GoingPrivateStep::DisconnectingClients),
                server_going_private_disconnect_clients,
            )
            .add_systems(
                Update,
                server_going_private_wait_clients_disconnected
                    .run_if(in_state(GoingPrivateStep::DisconnectingClients)),
            )
            .add_systems(
                OnEnter(GoingPrivateStep::ClosingServer),
                server_going_private_close_server,
            )
            .add_systems(
                Update,
                server_going_private_wait_server_closed
                    .run_if(in_state(GoingPrivateStep::ClosingServer)),
            )
            .add_systems(
                OnEnter(GoingPrivateStep::Cleanup),
                server_going_private_cleanup,
            )
            .add_systems(OnEnter(GoingPrivateStep::Ready), server_going_private_ready);
    }
}

fn server_going_public_validating(
    mut commands: Commands,
    mut server_settings: ResMut<NetworkingSettings>,
) {
    if !server_settings.can_be_discovered {
        Notify::error(
            "Server is not able to set public. Please change the setting for the server!",
        );
        commands.trigger(SetGoingPublicStep::Failed);
        return;
    }

    #[cfg(feature = "server")]
    if !ports::is_port_available(server_settings.port) {
        commands.trigger(Notify::error(format!(
            "Game Port {} is already in use, please choose another port.",
            server_settings.port
        )));
        commands.trigger(SetGoingPublicStep::Failed);
        return;
    }

    #[cfg(feature = "client")]
    match ports::find_free_port(server_settings.port, 100) {
        Some(port) => {
            server_settings.port = port;
        }
        None => {
            commands.trigger(Notify::error(format!("Failed to find a free Game port. We start searching for a free Discovery port between {} and {}. Please try another Port not in this range. If this Error persists, please try to restart your System.", server_settings.port, server_settings.port + 100)));
            commands.trigger(SetGoingPublicStep::Failed);
            return;
        }
    }
    commands.trigger(SetGoingPublicStep::Next);
}

fn server_going_public_starting_server(
    mut commands: Commands,
    server_settings: Res<NetworkingSettings>,
) {
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
        .queue(WebTransportServer::open(config));

    commands.trigger(SetGoingPublicStep::Next);
}

fn server_going_public_starting_discovery(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
    mut server_settings: ResMut<NetworkingSettings>,
) {
    match server_query.single() {
        Ok(_) => {
            match ports::is_port_available(server_settings.discovery_port) {
                true => {
                    // TODO: add DiscoveryLogik
                }
                false => {
                    server_settings.can_be_discovered = false;
                    #[cfg(feature = "client")]
                    commands.trigger(Notify::error(
                        format!("DiscoveryPort ({}) is already in use. Discovering of this session will be disabled!", server_settings.discovery_port),
                    ));
                    #[cfg(feature = "server")]
                    commands.trigger(Notify::error(
                        format!("DiscoveryPort ({}) is already in use. Discovering of this server will be disabled! If this Error persists, please try to restart your System.", server_settings.discovery_port),
                    ));
                    commands.trigger(SetGoingPublicStep::Failed);
                    return;
                }
            }
            commands.trigger(SetGoingPublicStep::Next);
        }
        Err(QuerySingleError::NoEntities(_)) => {
            commands.trigger(Notify::error(
                "WebTransportServer entity missing in StartingDiscovery! — this is a bug",
            ));
            commands.trigger(SetGoingPublicStep::Failed);
            return;
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            commands.trigger(Notify::error(
                "Multiple WebTransportServer entities found! - this is a bug",
            ));
            commands.trigger(SetGoingPublicStep::Failed);
            return;
        }
    }
}

fn server_going_public_ready(mut commands: Commands) {
    commands.trigger(SetGoingPublicStep::Done);
}

fn server_going_private_disconnect_clients(
    mut commands: Commands,
    client_query: Query<Entity, With<WebTransportServerClient>>,
) {
    for client in client_query.iter() {
        commands.trigger(Disconnect::new(client, "Server closing"));
    }
}

fn server_going_private_wait_clients_disconnected(
    mut commands: Commands,
    client_query: Query<Entity, With<WebTransportServerClient>>,
) {
    if client_query.is_empty() {
        commands.trigger(SetGoingPrivateStep::Next);
    }
}

fn server_going_private_close_server(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
) {
    for server in server_query.iter() {
        commands.trigger(Close::new(server, "Server closing"));
    }
}

fn server_going_private_wait_server_closed(
    mut commands: Commands,
    server_query: Query<Entity, With<WebTransportServer>>,
) {
    if server_query.is_empty() {
        commands.trigger(SetGoingPrivateStep::Next);
    }
}

fn server_going_private_cleanup(mut commands: Commands) {
    // TODO: impl Cleanup routine
    commands.trigger(SetGoingPrivateStep::Next);
}

fn server_going_private_ready(mut commands: Commands) {
    commands.trigger(SetGoingPrivateStep::Done);
}
