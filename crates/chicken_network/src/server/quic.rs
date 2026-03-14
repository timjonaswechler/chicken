use {
    super::networking::ports,
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
        events::session::{SetGoingPrivateStep, SetGoingPublicStep, SetServerStartupStep},
        states::session::{GoingPrivateStep, GoingPublicStep, ServerVisibility},
    },
    core::time::Duration,
};

pub(crate) struct QUICServerPlugin;

impl Plugin for QUICServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WebTransportServerPlugin)
            .add_systems(OnEnter(ServerVisibility::GoingPublic), server_going_public)
            .add_systems(
                OnEnter(ServerVisibility::GoingPrivate),
                server_going_private,
            );
    }
}

fn server_going_public(
    mut commands: Commands,
    mut server_settings: ResMut<NetworkingSettings>,
    step: Res<State<GoingPublicStep>>,
    server_query: Query<Entity, With<WebTransportServer>>,
) {
    if !server_settings.can_be_discovered {
        Notify::error(
            "Server is not able to set public. Please change the setting for the server!",
        );
        commands.trigger(SetGoingPublicStep::Failed);
        return;
    }

    match step.get() {
        GoingPublicStep::Validating => {
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
        GoingPublicStep::StartingServer => {
            let identity = aeronet_webtransport::wtransport::Identity::self_signed([
                "localhost",
                "127.0.0.1",
                "::1",
            ])
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
            // .observe(on_server_is_public)
            // .observe(on_server_session_request)
            // .observe(on_server_client_disconnected);

            match server_query.single() {
                Ok(_) => commands.trigger(SetServerStartupStep::Next),
                Err(QuerySingleError::NoEntities(_)) => {
                    error!("Error: There is no LocalServer right now!");
                    todo!("Handle this Error properly");
                }
                Err(QuerySingleError::MultipleEntities(_)) => {
                    error!("Error: There is more than one LocalServer right now!");
                    todo!("Handle this Error properly");
                }
            }
        }
        GoingPublicStep::StartingDiscovery => {
            match ports::is_port_available(server_settings.discovery_port) {
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
            commands.trigger(SetGoingPublicStep::Next);
        }
        GoingPublicStep::Ready => commands.trigger(SetGoingPublicStep::Done),
    }
    // TODO: Implement User interface infos for server
}

fn server_going_private(
    mut commands: Commands,
    step: Res<State<GoingPrivateStep>>,
    server_query: Query<Entity, With<WebTransportServer>>,
    client_query: Query<Entity, With<WebTransportServerClient>>,
) {
    match step.get() {
        GoingPrivateStep::DisconnectingClients => {
            for client in client_query.iter() {
                commands.trigger(Disconnect::new(client, "Server closing"));
            }

            if client_query.is_empty() {
                commands.trigger(SetGoingPrivateStep::Next);
            }
        }
        GoingPrivateStep::ClosingServer => {
            for server in server_query.iter() {
                commands.trigger(Close::new(server, "Server closing"));
            }

            if server_query.is_empty() {
                commands.trigger(SetGoingPrivateStep::Next);
            }
        }
        GoingPrivateStep::Cleanup => {
            // TODO: impl Cleanup routine
            commands.trigger(SetGoingPrivateStep::Next);
        }
        GoingPrivateStep::Ready => commands.trigger(SetGoingPrivateStep::Done),
    }
}
