use {
    bevy::prelude::*, chicken_settings_content::networking::NetworkingSettings,
    chicken_states::states::session::ServerVisibility, std::net::UdpSocket,
};

pub const MAGIC: &[u8] = b"FORGE_DISCOVER_V1";

#[derive(Resource)]
struct DiscoverySocket(UdpSocket);

pub struct DiscoveryServerPlugin;

impl Plugin for DiscoveryServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ServerVisibility::Public), insert_discovery_socket);
        app.add_systems(OnExit(ServerVisibility::Public), remove_discovery_socket);

        app.add_systems(
            Update,
            discovery_server_system
                .run_if(in_state(ServerVisibility::Public))
                .run_if(resource_exists::<DiscoverySocket>),
        );
    }
}

fn insert_discovery_socket(mut commands: Commands, settings: Res<NetworkingSettings>) {
    if !settings.can_be_discovered {
        info!("Discovery server disabled by settings");
        return;
    }
    commands.insert_resource(setup_discovery_socket(settings));
}

fn remove_discovery_socket(mut commands: Commands) {
    commands.remove_resource::<DiscoverySocket>();
}

fn setup_discovery_socket(settings: Res<NetworkingSettings>) -> DiscoverySocket {
    let socket = UdpSocket::bind(("0.0.0.0", settings.discovery_port))
        .expect("failed to bind discovery socket");
    socket
        .set_broadcast(true)
        .expect("failed to enable broadcast");
    socket
        .set_nonblocking(true)
        .expect("failed to set nonblocking");
    DiscoverySocket(socket)
}

fn discovery_server_system(socket: Res<DiscoverySocket>, settings: Res<NetworkingSettings>) {
    let mut buf = [0u8; 256];
    // alle eingehenden Pakete abarbeiten
    while let Ok((len, src)) = socket.0.recv_from(&mut buf) {
        if &buf[..len] == MAGIC {
            // minimale Antwort: Magic + Port
            let resp = format!("FORGE_RESP_V1;{}", settings.port);
            let _ = socket.0.send_to(resp.as_bytes(), src);
        }
    }
}
