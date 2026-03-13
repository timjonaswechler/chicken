use {
    crate::server::discovery::MAGIC,
    bevy::{
        prelude::*,
        tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
    },
    chicken_settings_content::networking::NetworkingSettings,
    chicken_states::states::menu::multiplayer::MultiplayerMenuScreen,
    std::{net::UdpSocket, time::Duration},
};

pub struct ClientDiscoveryPlugin;

impl Plugin for ClientDiscoveryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiscoveredServers>()
            .init_resource::<DiscoveryControl>()
            .add_systems(
                OnEnter(MultiplayerMenuScreen::JoinGame),
                client_discovery_reset,
            )
            .add_systems(
                OnExit(MultiplayerMenuScreen::JoinGame),
                client_discovery_cleanup,
            )
            .add_systems(
                Update,
                (client_discover_server, client_discover_server_collect)
                    .run_if(in_state(MultiplayerMenuScreen::JoinGame)),
            );
    }
}

/// Holds all servers that have been discovered by the client
#[derive(Resource, Default)]
pub struct DiscoveredServers(pub Vec<String>);

/// DiscoveryTask holds the task for the current discovery process
#[derive(Component)]
pub struct DiscoveryTask(Task<Vec<String>>);

/// DiscoveryControl holds all the state for the current discovery process
#[derive(Resource)]
pub struct DiscoveryControl {
    /// Timing of the discovery process
    pub timer: Timer,

    /// How many cycles to run the discovery process
    pub cycles_remaining: usize,

    /// Current ECS-generation of the discovery process
    pub current_generation: u64,

    /// Maps Address -> Last Seen Generation
    pub seen_servers: std::collections::HashMap<String, u64>,
}

impl Default for DiscoveryControl {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            cycles_remaining: 0,
            current_generation: 0,
            seen_servers: Default::default(),
        }
    }
}

fn client_discovery_reset(mut control: ResMut<DiscoveryControl>) {
    control.cycles_remaining = 5;
    control.timer.reset();
}

fn client_discovery_cleanup(mut commands: Commands, tasks: Query<Entity, With<DiscoveryTask>>) {
    for entity in &tasks {
        commands.entity(entity).despawn();
    }
}

fn client_discover_server(
    mut commands: Commands,
    time: Res<Time>,
    mut control: ResMut<DiscoveryControl>,
    query: Query<Entity, With<DiscoveryTask>>,
    server_settings: Res<NetworkingSettings>,
) {
    // Don't spawn if task is already running
    if !query.is_empty() {
        return;
    }

    if control.cycles_remaining == 0 {
        return;
    }

    if !control.timer.tick(time.delta()).is_finished() {
        return;
    }

    control.cycles_remaining -= 1;
    control.current_generation += 1;
    let generation = control.current_generation;

    info!(
        "Starting discovery cycle (Gen: {}). Remaining: {}",
        generation, control.cycles_remaining
    );

    let thread_pool = AsyncComputeTaskPool::get();
    let discovery_port = server_settings.discovery_port.clone();
    let task = thread_pool.spawn(async move {
        let socket = UdpSocket::bind(("0.0.0.0", 0)).expect("bind for discovery client");
        socket.set_broadcast(true).expect("enable broadcast");
        socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .ok();

        let _ = socket.send_to(MAGIC, ("255.255.255.255", discovery_port));

        let mut buf = [0u8; 256];
        let mut result = Vec::new();

        // Collect responses for a short window
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_millis(1500) {
            if let Ok((len, src)) = socket.recv_from(&mut buf) {
                let s = String::from_utf8_lossy(&buf[..len]);
                if let Some(port_str) = s.strip_prefix("FORGE_RESP_V1;") {
                    if let Ok(port) = port_str.parse::<u16>() {
                        // Ensure https:// prefix
                        let addr = format!("https://{}:{}", src.ip(), port);
                        result.push(addr);
                    }
                }
            }
        }

        result
    });

    commands.spawn((Name::new("DiscoveryTask"), DiscoveryTask(task)));
}

fn client_discover_server_collect(
    mut commands: Commands,
    mut discovered: ResMut<DiscoveredServers>,
    mut control: ResMut<DiscoveryControl>,
    mut query: Query<(Entity, &mut DiscoveryTask)>,
) {
    for (entity, mut task) in &mut query {
        if let Some(result) = check_ready(&mut task.0) {
            let current_gen = control.current_generation;

            // 1. Update seen servers with current generation
            for server in result {
                control.seen_servers.insert(server, current_gen);
            }

            // 2. Remove servers not seen in current generation
            // This fulfills "if entry does not appear anymore, remove it"
            control
                .seen_servers
                .retain(|_, seen_gen| *seen_gen == current_gen);

            // 3. Update the UI list
            discovered.0 = control.seen_servers.keys().cloned().collect();
            discovered.0.sort();

            commands.entity(entity).despawn();
        }
    }
}
