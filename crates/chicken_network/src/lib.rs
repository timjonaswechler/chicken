#![warn(missing_docs, clippy::unwrap_used)]
/*!
# Chicken Network

Networking crate for the Chicken game, built on Bevy's ecosystem with Aeronet and WebTransport.

## Architecture

This crate provides a unified networking layer supporting two modes:

- **Client**: Connects to remote multiplayer servers with discovery and connection management
- **Server**: Hosts multiplayer sessions with visibility control and UDP discovery broadcast

Built on `bevy_replicon` for entity replication and `aeronet` for WebTransport networking.

## Key Components

- `ChickenNetPlugin`: Main plugin that wires all networking systems
- `client`: Client connection lifecycle, server discovery, and teardown
- `server`: Server hosting, visibility state machine, and discovery responses
- `shared`: Common message types and replicated resources
- `settings`: Network configuration (`ServerSettings`, `NetworkSettings`)

## Usage

Add `ChickenNetPlugin` to your Bevy app:

```rust,no_run
use bevy::prelude::*;
use chicken_network::ChickenNetPlugin;

fn main() {
    App::new()
        .add_plugins(ChickenNetPlugin)
        .run();
}
```

## Features

- `client` (default) - Client-side networking and singleplayer support
- `server` (default) - Server hosting capabilities
- `auth` (optional) - Authentication support for verified connections

Use `--no-default-features` with specific features for dedicated server or client-only builds.
*/

/// Client networking logic, connection management, and server discovery.
#[cfg(feature = "client")]
pub mod client;

/// Server hosting, visibility management, and discovery broadcast.
pub mod server;

/// Network configuration settings for servers and clients.
mod settings;

/// Shared message types, resources, and replicated components.
pub mod shared;

#[cfg(feature = "client")]
use aeronet_replicon::client::AeronetRepliconClientPlugin;

use {
    aeronet_replicon::server::AeronetRepliconServerPlugin, bevy::prelude::*,
    bevy_replicon::RepliconPlugins,
};

pub use server::local::{LocalBot, LocalClient, LocalServer, LocalSession};

/// Main plugin that wires all networking systems for the Chicken game.
///
/// This plugin sets up the complete networking stack including:
/// - Entity replication via `bevy_replicon`
/// - WebTransport networking via `aeronet`
/// - Server hosting and visibility management
/// - Client connection lifecycle and discovery
///
/// The plugin automatically configures the appropriate systems based on enabled features:
/// - With `server` feature: Server hosting and discovery broadcast
/// - With `client` feature: Client connections and discovery
///
/// # Usage
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use chicken_network::ChickenNetPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(ChickenNetPlugin)
///         .run();
/// }
/// ```
pub struct ChickenNetPlugin;

impl Plugin for ChickenNetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RepliconPlugins,
            AeronetRepliconServerPlugin,
            server::ServerLogicPlugin,
            shared::SharedPlugin,
        ));

        #[cfg(feature = "client")]
        app.add_plugins((AeronetRepliconClientPlugin, client::ClientLogicPlugin));
    }
}
