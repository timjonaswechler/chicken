pub(crate) mod auth;
pub(crate) mod discovery;

use bevy_replicon::prelude::Replicated;
pub use auth::LocalIdentity;
pub use discovery::{DiscoveredServers, DiscoveryControl, DiscoveryTask};

use {
    crate::server::local::LocalClient,
    aeronet::io::{
        Session,
        connection::{Disconnect, Disconnected},
    },
    aeronet_io::connection::DisconnectReason,
    aeronet_replicon::client::AeronetRepliconClient,
    aeronet_webtransport::client::{WebTransportClient, WebTransportClientPlugin},
    auth::ClientAuthPlugin,
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    chicken_identity::PlayerIdentity,
    chicken_notifications::Notify,
    chicken_protocols::ClientIdentityHello,
    chicken_states::{
        events::session::{SetConnectingStep, SetDisconnectingStep, SetSyncingStep},
        states::session::{ClientConnectionStatus, ConnectingStep, DisconnectingStep, SyncingStep},
    },
    discovery::ClientDiscoveryPlugin,
    helpers::client_config,
};

pub(crate) struct ClientLogicPlugin;

impl Plugin for ClientLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WebTransportClientPlugin, ClientDiscoveryPlugin, ClientAuthPlugin))
            .init_resource::<ClientTarget>()
            .add_systems(
                OnEnter(ClientConnectionStatus::Connecting),
                on_client_connecting,
            )
            .add_systems(
                Update,
                advance_connecting_steps.run_if(in_state(ClientConnectionStatus::Connecting)),
            )
            .add_systems(
                Update,
                client_syncing.run_if(in_state(ClientConnectionStatus::Syncing)),
            )
            .add_systems(
                OnEnter(ClientConnectionStatus::Connected),
                on_client_running,
            )
            .add_systems(
                OnEnter(ClientConnectionStatus::Playing),
                on_client_playing,
            )
            .add_systems(
                Update,
                client_disconnecting.run_if(in_state(ClientConnectionStatus::Disconnecting)),
            );
    }
}

fn on_client_disconnected(
    trigger: On<Disconnected>,
    state: Res<State<ClientConnectionStatus>>,
    mut commands: Commands,
) {
    match state.get() {
        ClientConnectionStatus::Connected | ClientConnectionStatus::Syncing => {
            notify_disconnect(&trigger.reason, &mut commands);
            commands.trigger(SetConnectingStep::Failed);
        }
        ClientConnectionStatus::Playing => {
            notify_disconnect(&trigger.reason, &mut commands);
            commands.trigger(SetDisconnectingStep::Start);
        }
        _ => {}
    }
}

fn notify_disconnect(reason: &DisconnectReason, commands: &mut Commands) {
    match reason {
        DisconnectReason::ByPeer(msg) => {
            info!("Server closed connection: {msg}");
            commands.trigger(Notify::info(format!("Server closed connection: {msg}")));
        }
        DisconnectReason::ByError(err) => {
            error!("Connection lost: {err}");
            commands.trigger(Notify::error(format!("Connection lost: {err}")));
        }
        DisconnectReason::ByUser(_) => {}
    }
}

/// ClientTarget is the data structure which the user selected or putted in a text field
/// <div class="warning">
/// should be improved to handle real data instead of just a string
/// </div>
#[derive(Resource, Default)]
pub struct ClientTarget {
    /// The input string provided by the user
    pub input: String, // "127.0.0.1:8080"

    /// Combination of ip and port
    pub real_address: String,

    /// String representation of the ip address
    pub ip: String,

    /// u16 representation of the port
    pub port: u16,

    /// Marker to indicate if the target is valid
    pub is_valid: bool,
}

impl ClientTarget {
    /// Updates the client target with a new input string, parsing and validating it.
    ///
    /// Parses the input to extract IP address and port, updating the `real_address`,
    /// `ip`, `port`, and `is_valid` fields accordingly. If parsing fails, marks the
    /// target as invalid and clears the derived fields.
    ///
    /// # Arguments
    ///
    /// * `input` - The raw connection string (e.g., "127.0.0.1:8080")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chicken_network::client::ClientTarget;
    ///
    /// let mut target = ClientTarget::default();
    /// target.update_input("127.0.0.1:8080".to_string());
    /// assert!(target.is_valid);
    /// assert_eq!(target.ip, "127.0.0.1");
    /// assert_eq!(target.port, 8080);
    /// ```
    pub fn update_input(&mut self, input: String) {
        self.input = input;

        if let Some((ip, port, real_address)) = helpers::parse_target_live(&self.input) {
            self.ip = ip;
            self.port = port;
            self.real_address = real_address;
            self.is_valid = true;
        } else {
            self.ip.clear();
            self.port = 0;
            self.real_address.clear();
            self.is_valid = false;
        }
    }
}

/// Command to set the client's connection target.
///
/// When applied, this command creates or updates the `ClientTarget` resource
/// with the parsed connection information (IP, port, validation status).
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use chicken_network::client::SetClientTarget;
///
/// fn set_target(mut commands: Commands) {
///     commands.add(SetClientTarget {
///         input: "127.0.0.1:8080".to_string(),
///     });
/// }
/// ```
pub struct SetClientTarget {
    /// The raw connection string input (e.g., "127.0.0.1:8080" or hostname).
    pub input: String,
}

impl Command for SetClientTarget {
    fn apply(self, world: &mut World) {
        let mut target = ClientTarget::default();
        target.update_input(self.input);
        world.insert_resource(target);
    }
}

fn on_client_connecting(
    mut commands: Commands,
    client_target: Res<ClientTarget>,
    mut cert_hash: Local<String>,
    mut session_id: Local<usize>,
    mut control: ResMut<DiscoveryControl>,
    tasks: Query<Entity, With<DiscoveryTask>>,
) {
    // Stop discovery when connecting
    control.cycles_remaining = 0;
    for entity in &tasks {
        commands.entity(entity).despawn();
    }

    let _cert_hash_resp = &mut *cert_hash;
    let cert_hash = cert_hash.clone();
    let config = match client_config(cert_hash) {
        Ok(config) => config,
        Err(err) => {
            commands.trigger(Notify::error(format!(
                "Failed to create client config: {err:?}"
            )));
            return;
        }
    };

    *session_id += 1;
    let name = format!("{:#?}. {:?}", *session_id, client_target.input);
    info!("Connecting to server at {:?}", client_target.input);
    commands
        .spawn((
            Name::new(name),
            LocalClient,
            AeronetRepliconClient,
            Replicated,
        ))
        .queue(WebTransportClient::connect(
            config,
            client_target.real_address.clone(),
        ))
        .observe(on_client_connected)
        .observe(on_client_connection_failed)
        .observe(on_client_disconnected);
}

fn on_client_connection_failed(
    trigger: On<Disconnected>,
    current_state: Option<Res<State<ClientConnectionStatus>>>,
    mut commands: Commands,
    mut client_target: ResMut<ClientTarget>,
) {
    if let Some(current_state) = current_state {
        if *current_state.get() == ClientConnectionStatus::Connecting {
            match &trigger.reason {
                DisconnectReason::ByError(err) => {
                    error!("Connection Error: {}", err);
                    commands.trigger(Notify::error(format!("Connection Error: {}", err)));
                    client_target.is_valid = false;
                    commands.trigger(SetConnectingStep::Failed);
                }
                DisconnectReason::ByUser(err) => {
                    error!("Connection Error: {}", err);
                    commands.trigger(Notify::error(format!("Connection Error: {}", err)));
                    client_target.is_valid = false;
                    commands.trigger(SetConnectingStep::Failed);
                }
                DisconnectReason::ByPeer(err) => {
                    error!("Connection Error: {}", err);
                    commands.trigger(Notify::error(format!("Connection Error: {}", err)));
                    client_target.is_valid = false;
                    commands.trigger(SetConnectingStep::Failed);
                }
            }
        }
    }
}

fn on_client_connected(
    trigger: On<Add, Session>,
    names: Query<&Name>,
    mut commands: Commands,
    mut hello_writer: MessageWriter<ClientIdentityHello>,
    identity: Option<Res<LocalIdentity>>,
    player_identity: Option<Res<PlayerIdentity>>,
) {
    let target = trigger.event_target();

    let name = names.get(target).ok();
    if let Some(name) = name {
        info!("Connected as {}", name.as_str());
    } else {
        warn!("Session {} missing Name component", target);
    }

    let Some(identity) = identity else {
        warn!("LocalIdentity nicht verfügbar beim Verbinden — Auth wird fehlschlagen");
        commands.trigger(SetConnectingStep::Next);
        return;
    };

    let display_name = player_identity
        .as_ref()
        .map(|pi| pi.display_name.clone())
        .unwrap_or_else(|| format!("Player-{}", &identity.player_id[..8]));

    let steam_id = player_identity.as_ref().and_then(|pi| pi.steam_id);

    hello_writer.write(ClientIdentityHello {
        public_key: identity.verifying_key_bytes(),
        display_name,
        steam_id,
        password: None, // TODO: aus UI-Eingabe befüllen sobald Passwort-Dialog implementiert ist
    });

    // Session established → OpeningConnection → Authenticating
    commands.trigger(SetConnectingStep::Next);
}

/// Verwaltet ConnectingStep-Übergänge.
/// `OpeningConnection` → `Authenticating`: driven by on_client_connected (On<Add, Session>).
/// `Authenticating` → `WaitingForAccept`: driven by client/auth.rs (ServerAuthChallenge empfangen).
/// `WaitingForAccept` → `Ready`: driven by client/auth.rs (ServerAuthResult empfangen).
fn advance_connecting_steps(step: Option<Res<State<ConnectingStep>>>, mut commands: Commands) {
    let Some(step) = step else { return };

    match step.get() {
        ConnectingStep::OpeningConnection => {
            // Wartet auf On<Add, Session> — driven by on_client_connected
        }
        ConnectingStep::Authenticating => {
            // Wartet auf ServerAuthChallenge → ClientAuthResponse in client/auth.rs
        }
        ConnectingStep::WaitingForAccept => {
            // Wartet auf ServerAuthResult in client/auth.rs
        }
        ConnectingStep::Ready => {
            commands.trigger(SetConnectingStep::Done);
        }
    }
}

/// Advances SyncingStep one step per frame.
/// TODO: replace with real world-sync logic (request world, receive chunks, spawn entities).
fn client_syncing(step: Option<Res<State<SyncingStep>>>, mut commands: Commands) {
    let Some(step) = step else { return };
    match step.get() {
        SyncingStep::RequestWorld | SyncingStep::ReceiveChunks | SyncingStep::SpawnEntities => {
            commands.trigger(SetSyncingStep::Next);
        }
        SyncingStep::Ready => {
            commands.trigger(SetSyncingStep::Done);
        }
    }
}

fn on_client_running(mut commands: Commands) {
    info!("Client connected, starting sync");
    commands.trigger(SetSyncingStep::Start);
}

fn on_client_playing(identity: Option<Res<PlayerIdentity>>) {
    let name = identity
        .map(|id| id.display_name.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    info!("Player '{}' is now in-game (Playing)", name);
}

fn client_disconnecting(
    mut commands: Commands,
    step: Res<State<DisconnectingStep>>,
    client_query: Query<Entity, With<LocalClient>>,
) {
    match step.get() {
        DisconnectingStep::SendDisconnect => {
            // Send disconnect signal to server
            if let Ok(entity) = client_query.single() {
                commands.trigger(Disconnect::new(entity, "client disconnecting"));
            }
            commands.trigger(SetDisconnectingStep::Next);
        }
        DisconnectingStep::WaitForAck => {
            // TODO: Wait for server acknowledgment before proceeding
            commands.trigger(SetDisconnectingStep::Next);
        }
        DisconnectingStep::Cleanup => {
            // Despawn local client entity
            if let Ok(entity) = client_query.single() {
                if let Ok(mut entity) = commands.get_entity(entity) {
                    entity.despawn();
                }
            } else if client_query.is_empty() {
                commands.trigger(SetDisconnectingStep::Next);
            }
        }
        DisconnectingStep::Ready => {
            commands.trigger(SetDisconnectingStep::Done);
        }
    }
}

pub(crate) mod helpers {
    use {
        aeronet_webtransport::{cert, client::ClientConfig, wtransport::tls::Sha256Digest},
        bevy::prelude::*,
        core::time::Duration,
        std::net::SocketAddr,
    };

    // TODO: Remove anyhow here
    pub(super) fn client_config(cert_hash: String) -> Result<ClientConfig, anyhow::Error> {
        let config = ClientConfig::builder().with_bind_default();

        let config = if cert_hash.is_empty() {
            #[cfg(debug_assertions)]
            {
                warn!("Connecting with no certificate validation");
                config.with_no_cert_validation()
            }
            #[cfg(not(debug_assertions))]
            {
                config.with_server_certificate_hashes([])
            }
        } else {
            let hash = cert::hash_from_b64(&cert_hash)?;
            config.with_server_certificate_hashes([Sha256Digest::new(hash)])
        };

        Ok(config
            .keep_alive_interval(Some(Duration::from_secs(1)))
            .max_idle_timeout(Some(Duration::from_secs(5)))
            .expect("should be a valid idle timeout")
            .build())
    }

    pub(super) fn parse_target_live(input: &str) -> Option<(String, u16, String)> {
        let trimmed = input.trim();

        // Remove known prefixes to parse the raw address
        let raw_addr_str = if let Some(stripped) = trimmed.strip_prefix("https://") {
            stripped
        } else if let Some(stripped) = trimmed.strip_prefix("http://") {
            stripped
        } else {
            trimmed
        };

        // Try to parse IP:Port
        if let Ok(addr) = raw_addr_str.parse::<SocketAddr>() {
            if addr.port() == 0 {
                return None;
            }
            // Reconstruct clean URL, forcing https for WebTransport
            let clean_url = format!("https://{}", addr);
            return Some((addr.ip().to_string(), addr.port(), clean_url));
        }

        None
    }
}
