use {
    super::app::AppScope,
    bevy::prelude::{ComputedStates, Reflect, StateSet, States, SubStates},
};

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
    #[cfg(feature = "client")]
    Paused,
}

/// Defines the type of session.
#[cfg(feature = "client")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum SessionType {
    /// This is active if there is no active game running, for example when the game is in the main menu.
    #[default]
    None,

    /// This Type is active if the game is in a singleplayer session or multiplayer session where the user is the host.
    Singleplayer,

    /// Client is active if the game is connected to a multiplayer session where the user is a client.
    Client,
}

// --- Client Specific Session States ---

/// Defines the State of the client.
#[cfg(feature = "client")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SessionType = SessionType::Client)]
pub enum ClientStatus {
    /// If the Clients starts it is always first in a connecting state.
    #[default]
    Connecting,

    /// If the connecting is successful the client switches to the connected state.
    Connected,

    /// When the client is connected the next state is always syncing data from the server to the client.
    Syncing,

    /// Is all in sync, the client switches to the running state. In this state the client is ready to send and receive data (Gameplay Loop)
    Running,

    /// When the user wants to disconnect from the server or the server sends a disconnect message, the client switches to the disconnecting state.
    Disconnecting,
}

/// In disconnecting phase the clients steps through the shutdown steps. This steps are defined here.
#[cfg(feature = "client")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(ClientStatus = ClientStatus::Disconnecting)]
pub enum ClientShutdownStep {
    /// First the client disconnects from the server.
    #[default]
    DisconnectFromServer,

    /// Then the client despawns the local client entity.
    DespawnLocalClient,
}

// --- Singleplayer Specific Session States ---

/// Tracks the lifecycle state of a singleplayer game session.
///
/// This state machine manages the startup, active gameplay, and shutdown phases
/// of a local singleplayer session. Only active when `SessionType` is `Singleplayer`.
#[cfg(feature = "client")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SessionType = SessionType::Singleplayer)]
pub enum SingleplayerStatus {
    /// Initial startup phase: loading world, spawning entities, initializing systems.
    #[default]
    Starting,
    /// Active gameplay phase: physics running, player input processed, game logic executing.
    Running,
    /// Shutdown phase: cleaning up entities, saving game state, releasing resources.
    Stopping,
}

/// Defines the ordered shutdown sequence for a singleplayer session.
///
/// When a singleplayer session transitions to `Stopping`, it progresses through
/// these steps to ensure clean teardown of all session components.
#[cfg(feature = "client")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates, Reflect)]
#[source(SingleplayerStatus = SingleplayerStatus::Stopping)]
pub enum SingleplayerShutdownStep {
    /// First, disconnect any remote clients if the session was open to LAN.
    #[default]
    DisconnectRemoteClients,
    /// Close the local server instance.
    CloseRemoteServer,
    /// Despawn all AI-controlled bot entities.
    DespawnBots,
    /// Despawn the local player client entity.
    DespawnLocalClient,
    /// Finally, despawn the local server entity and complete cleanup.
    DespawnLocalServer,
}

// --- Server Specific Session States ---

/// Controls the visibility status of a multiplayer server.
///
/// Manages the server's presence in public server listings, including
/// transitions between private and public states. The server can be
/// private (invisible), public (listed), or in transition between these states.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States, Reflect)]
pub enum ServerVisibility {
    /// Server is not listed in public listings; only direct IP connections allowed.
    #[default]
    Private,
    /// Request to make the server public has been submitted but not yet confirmed.
    #[cfg(feature = "client")]
    PendingPublic,
    /// Server is in the process of transitioning to public visibility.
    GoingPublic,
    /// Server is publicly listed and discoverable by other players.
    Public,
    /// Server is in the process of transitioning to private visibility.
    GoingPrivate,
    /// The visibility transition failed; server remains in previous state.
    Failed,
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

// Physics runs when SessionState is Active.
impl ComputedStates for PhysicsSimulation {
    type SourceStates = SessionState;

    fn compute(session: SessionState) -> Option<Self> {
        match session {
            SessionState::Active => Some(PhysicsSimulation::Running),
            _ => Some(PhysicsSimulation::Paused),
        }
    }
}
