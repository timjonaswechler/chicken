# State-Architektur

## 1. AppScope (Root State)

```
AppScope
├── Splash (default) ────────────── Client only: Initial splash/intro screen
├── Menu ────────────────────────── Client only: Main menu
│   └── MainMenuScreen
│       ├── Main (default)
│       ├── Singleplayer
│       │   └── SingleplayerMenuScreen
│       │       ├── Overview (default)
│       │       ├── NewGame
│       │       │   └── NewGameMenuScreen
│       │       │       ├── ConfigPlayer (default)
│       │       │       ├── ConfigWorld
│       │       │       └── ConfigSave
│       │       └── LoadGame
│       │           └── SavedGameMenuScreen
│       │               └── SelectSaveGame (default)
│       ├── Multiplayer
│       │   └── MultiplayerMenuScreen
│       │       ├── Overview (default)
│       │       ├── HostNewGame
│       │       │   └── HostNewGameMenuScreen
│       │       │       ├── ConfigServer (default)
│       │       │       ├── ConfigWorld
│       │       │       └── ConfigSave
│       │       ├── HostSavedGame
│       │       │   └── HostSavedGameMenuScreen
│       │       │       ├── Overview (default)
│       │       │       └── ConfigServer
│       │       └── JoinGame
│       │           └── JoinGameMenuScreen
│       │               └── Overview (default)
│       ├── Wiki
│       │   └── WikiMenuScreen
│       │       └── Overview (default)
│       └── Settings
│           └── SettingsMenuScreen
│               ├── Overview (default)
│               ├── Audio
│               ├── Video
│               └── Controls
└── Session ────────────────────── Client & Server: Active game session
    └── SessionState
        ├── Setup (default)
        ├── Active
        └── Paused ──────────────── Client only
            └── PauseMenu
                ├── Overview (default)
                ├── Settings
                ├── Save
                ├── Load
                └── Exit

```

## 2. SessionType (SubState von AppScope::Session)

```
SessionType
├── None (default) ──────────────── No active game, in main menu
├── Singleplayer [hosted] ─────────→ ServerStatus
├── Client [hosted] ────────────────→ ClientConnectionStatus
└── DedicatedServer [headless] ────→ ServerStatus

```

## 3. Server-Seite (Singleplayer & DedicatedServer)

```
ServerStatus (SubState von SessionType::Singleplayer oder DedicatedServer)
├── Offline (default)
├── Starting ──► ServerStartupStep
│   ├── Init (default)
│   ├── LoadWorld
│   ├── SpawnEntities
│   └── Ready ──► ServerStatus::Running
├── Running
│   └── ServerVisibility (SubState von ServerStatus::Running)
│       ├── Private (default)
│       ├── GoingPublic ──► GoingPublicStep
│       │   ├── Validating (default)
│       │   ├── StartingServer
│       │   ├── StartingDiscovery
│       │   └── Ready ──► ServerVisibility::Public
│       ├── Public
│       └── GoingPrivate ──► GoingPrivateStep
│           ├── DisconnectingClients (default)
│           ├── ClosingServer
│           ├── Cleanup 
│           └── Ready ──► ServerVisibility::Private
└── Stopping ──► ServerShutdownStep
    ├── SaveWorld (default)
    ├── DisconnectClients
    ├── DespawnLocalClient [hosted only]
    ├── Cleanup 
    └── Ready ──► ServerStatus::Offline

```

## 4. Client-Seite (SessionType::Client)

```
ClientConnectionStatus (SubState von SessionType::Client)
├── Disconnected (default)
├── Connecting ──► ConnectingStep
│   ├── ResolveAddress (default)
│   ├── OpenSocket
│   ├── SendHandshake
│   ├── WaitForAccept
│   └── Ready ──► ClientConnectionStatus::Connected
├── Connected
├── Syncing ──► SyncingStep
│   ├── RequestWorld (default)
│   ├── ReceiveChunks
│   ├── SpawnEntities
│   └── Ready ──► ClientConnectionStatus::Playing
├── Playing
└── Disconnecting ──► DisconnectingStep
    ├── SendDisconnect (default)
    ├── WaitForAck
    ├── Cleanup
    └── Ready ──► ClientConnectionStatus::Disconnected

```

## 5. Computed States

```
PhysicsSimulation (Computed aus SessionState)
├── Running ────────────────────────── Wenn SessionState::Active
└── Paused ──────────────────────────── Wenn SessionState::Setup oder Paused

```

## Feature-Flags

- `hosted`: Client-Funktionalität (Menus, Client-Verbindung)
- `headless`: Dedicated Server Funktionalität
