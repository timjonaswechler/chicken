use {
    super::app::AppScope,
    bevy::prelude::{ComputedStates, Reflect, StateSet, States, SubStates},
};

/// Defines the type of session.
#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub enum SessionType {
    /// This is active if there is no active game running, for example when the game is in the main menu.
    #[cfg(feature = "hosted")]
    #[default]
    None,

    /// This Type is active if the game is in a singleplayer session or multiplayer session where the user is the host.
    #[cfg(feature = "hosted")]
    Singleplayer,

    /// Client is active if the game is connected to a multiplayer session where the user is a client.
    #[cfg(feature = "hosted")]
    Client,

    /// DedicatedServer is active when running as a dedicated server without a local client.
    #[cfg(feature = "headless")]
    #[default]
    DedicatedServer,
}

/// The [SessionState] holds the current state of the In-Game Loop.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(AppScope = AppScope::Session)]
pub enum SessionState {
    /// Initial setup phase.
    /// - Server: World Generation, Map Loading.
    /// - Client: Connecting, Syncing, Loading Screen.
    #[default]
    Setup,

    /// The active game loop.
    /// Physics and Gameplay are running.
    Active,

    /// In-Game Pause Menu (Client only).
    /// Physics might be paused here.
    #[cfg(feature = "hosted")]
    Paused,
}

/// Tracks which pause menu screen is currently active.
///
/// This is a substate that is only valid when the session is in the `Paused` state.
/// It manages the navigation flow within the pause menu system.
#[cfg(feature = "hosted")]
#[derive(SubStates, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[source(SessionState = SessionState::Paused)]
pub enum PauseMenu {
    /// Main pause menu with primary options (Resume, Settings, Save, Load, Exit).
    #[default]
    Overview,
    /// Settings configuration screen for audio, video, and controls.
    Settings,
    /// Save game slot selection screen.
    Save,
    /// Load game slot selection screen.
    Load,
    /// Exit confirmation dialog.
    Exit,
}

// --- Server Status (Singleplayer & DedicatedServer) ---

/// Tracks the lifecycle state of a server session.
///
/// This state machine manages the startup, active gameplay, and shutdown phases
/// of both singleplayer (local) and dedicated server sessions.
/// Active when `SessionType` is `Singleplayer` or `DedicatedServer`.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[cfg(feature = "hosted")]
#[source(SessionType = SessionType::Singleplayer)]
pub enum ServerStatus {
    /// Server is not running; no active game session.
    #[default]
    Offline,
    /// Server is starting up: loading world, spawning entities, initializing systems.
    Starting,
    /// Server is running: physics active, accepting connections, processing gameplay.
    Running,
    /// Server is shutting down: saving state, disconnecting clients, releasing resources.
    Stopping,
}

/// Tracks the lifecycle state of a server session.
///
/// This state machine manages the startup, active gameplay, and shutdown phases
/// of dedicated server sessions.
/// Active when `SessionType` is `DedicatedServer`.
///
/// Note: There is no `Offline` state — the binary starting means the server starts.
/// On shutdown or failure, the process exits via `AppExit` instead of returning to idle.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[cfg(feature = "headless")]
#[source(SessionType = SessionType::DedicatedServer)]
pub enum ServerStatus {
    /// Server is starting up: loading world, spawning entities, initializing systems.
    #[default]
    Starting,
    /// Server is running: physics active, accepting connections, processing gameplay.
    Running,
    /// Server is shutting down: saving state, disconnecting clients, releasing resources.
    Stopping,
}

/// Defines the ordered startup sequence for a server session.
///
/// When a server transitions to `Starting`, it progresses through
/// these steps to ensure proper initialization of all server components.
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(ServerStatus = ServerStatus::Starting)]
pub enum ServerStartupStep {
    /// Initial initialization phase.
    #[default]
    Init,
    /// Loading world data and map generation.
    LoadWorld,
    /// Spawning all game entities.
    SpawnEntities,
    /// Server is fully ready to accept connections.
    Ready,
}

/// Defines the ordered shutdown sequence for a server session.
///
/// When a server transitions to `Stopping`, it progresses through
/// these steps to ensure clean teardown of all session components.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(ServerStatus = ServerStatus::Stopping)]
pub enum ServerShutdownStep {
    /// First, save the current world state to disk.
    #[default]
    SaveWorld,
    /// Disconnect all connected remote clients gracefully.
    DisconnectClients,
    /// Despawn the local client entity (client-hosted sessions only).
    #[cfg(feature = "hosted")]
    DespawnLocalClient,
    /// Final cleanup: despawn server entity and release resources.
    Cleanup,
    /// Signal that the server has stopped gracefully.
    Ready,
}

// --- Server Visibility ---

/// Controls the visibility status of a multiplayer server.
///
/// Manages the server's presence in public server listings, including
/// transitions between private and public states. The server can be
/// private (invisible), public (listed), or in transition between these states.
#[derive(SubStates, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
#[source(ServerStatus = ServerStatus::Running)]
pub enum ServerVisibility {
    /// Server is not listed in public listings; only direct IP connections allowed.
    #[default]
    Private,
    /// Server is in the process of transitioning to public visibility.
    GoingPublic,
    /// Server is publicly listed and discoverable by other players.
    Public,
    /// Server is in the process of transitioning to private visibility.
    GoingPrivate,
}

/// Defines the ordered sequence for making a server publicly visible.
///
/// When a server transitions to `GoingPublic`, it progresses through
/// these steps to register with discovery services and become publicly listed.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(ServerVisibility = ServerVisibility::GoingPublic)]
pub enum GoingPublicStep {
    /// Validate server configuration and prerequisites.
    #[default]
    Validating,
    /// Start the public-facing server socket.
    StartingServer,
    /// Register with the discovery/matchmaking service.
    StartingDiscovery,
    /// Server is fully public and discoverable.
    Ready,
}

/// Defines the ordered sequence for making a server private.
///
/// When a server transitions to `GoingPrivate`, it progresses through
/// these steps to unregister from public listings and close public access.
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(ServerVisibility = ServerVisibility::GoingPrivate)]
pub enum GoingPrivateStep {
    /// Disconnect all clients connected via public discovery.
    #[default]
    DisconnectingClients,
    /// Close the public-facing server socket.
    ClosingServer,
    /// Complete cleanup; server is now private.
    Cleanup,
    /// Server is fully private.
    Ready,
}

// --- Client Connection Status ---

/// Defines the connection state of a client.
///
/// This state machine manages the entire client connection lifecycle,
/// from initial connection attempt through gameplay to disconnection.
/// Only active when `SessionType` is `Client`.
#[cfg(feature = "hosted")]
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(SessionType = SessionType::Client)]
pub enum ClientConnectionStatus {
    /// Client is not connected to any server.
    #[default]
    Disconnected,
    /// Client is attempting to establish a connection to the server.
    Connecting,
    /// Connection established, but not yet synced.
    Connected,
    /// Client is receiving world state from the server.
    Syncing,
    /// Client is fully synced and participating in gameplay.
    Playing,
    /// Client is in the process of disconnecting from the server.
    Disconnecting,
}

/// Defines the ordered connection sequence for a client.
///
/// When a client transitions to `Connecting`, it progresses through
/// these steps to establish a connection to the server.
#[cfg(feature = "hosted")]
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(ClientConnectionStatus = ClientConnectionStatus::Connecting)]
pub enum ConnectingStep {
    /// Resolve the server address (DNS or IP).
    #[default]
    ResolveAddress,
    /// Open a socket connection to the server.
    OpenSocket,
    /// Send the initial handshake packet.
    SendHandshake,
    /// Wait for server acceptance response.
    WaitForAccept,
    /// Connection fully established.
    Ready,
}

/// Defines the ordered world synchronization sequence for a client.
///
/// When a client transitions to `Syncing`, it progresses through
/// these steps to receive the current world state from the server.
#[cfg(feature = "hosted")]
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(ClientConnectionStatus = ClientConnectionStatus::Syncing)]
pub enum SyncingStep {
    /// Request the current world state from the server.
    #[default]
    RequestWorld,
    /// Receive and load chunk data from the server.
    ReceiveChunks,
    /// Spawn all entities received from the server.
    SpawnEntities,
    /// Synchronization complete; ready for gameplay.
    Ready,
}

/// Defines the ordered disconnection sequence for a client.
///
/// When a client transitions to `Disconnecting`, it progresses through
/// these steps to cleanly terminate the connection to the server.
#[cfg(feature = "hosted")]
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
#[source(ClientConnectionStatus = ClientConnectionStatus::Disconnecting)]
pub enum DisconnectingStep {
    /// Send disconnect notification to the server.
    #[default]
    SendDisconnect,
    /// Wait for server acknowledgment of disconnect.
    WaitForAck,
    /// Clean up local connection resources.
    Cleanup,
    /// Disconnection process complete.
    Ready,
}

// --- Computed States ---

/// Computed state indicating whether the physics simulation is currently active.
///
/// This state is automatically derived from `SessionState`:
/// - `Running` when the session is `Active`
/// - `Paused` when the session is in `Setup` or `Paused` states
///
/// Used to conditionally run physics systems based on game state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum PhysicsSimulation {
    /// Physics simulation is actively processing entity movements and collisions.
    Running,
    /// Physics simulation is frozen; entities maintain their current state.
    Paused,
}

impl ComputedStates for PhysicsSimulation {
    type SourceStates = SessionState;

    fn compute(session: SessionState) -> Option<Self> {
        match session {
            SessionState::Active => Some(PhysicsSimulation::Running),
            _ => Some(PhysicsSimulation::Paused),
        }
    }
}
