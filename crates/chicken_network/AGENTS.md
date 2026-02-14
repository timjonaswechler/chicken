# AGENTS

## Project overview
- `network` is a Rust library crate that bundles Bevy (0.18) plugins for networking, server hosting, clients, and singleplayer loops (see `src/lib.rs:25-40`).
- Networking sits on Aeronet + WebTransport, with replication handled by `bevy_replicon` and project-specific crates (`states`, `notification`, etc.).
- The crate exposes `ChickenNetPlugin`, which downstream Bevy apps add to gain server/client functionality. Features (`client`, `server`, `auth`) gate large portions of the code.

## Repository layout
- `Cargo.toml`: defines local path dependencies (`../states`, `../notification`, `../exitcodes`, `../bevy_settings`, `../bevy_paths`). These must exist alongside this repo for builds/tests to succeed.
- `src/lib.rs`: wires `ChickenNetPlugin` and re-exports `local` helpers.
- `src/server.rs`: server plugin (`ServerLogicPlugin`) plus systems handling visibility state, WebTransport lifecycle, Aeronet observers, and helper utilities like port scanning.
- `src/server/discovery.rs`: UDP broadcast server that advertises discovery info while the server is public.
- `src/client.rs`: client plugin (`ClientLogicPlugin`) orchestrating connection lifecycle, discovery control, and teardown.
- `src/client/discovery.rs`: async UDP discovery jobs kicked off when joining multiplayer sessions.
- `src/shared.rs`: replicated messages (`PlayerNameMessage`) and shared resources added to the Bevy app.
- `src/singleplayer.rs`: manages a local client/server loop for singleplayer sessions via `ChannelIo` bridges.
- `src/settings.rs`: defines `ServerSettings` (`server.toml`/`temp.toml`) and `NetworkSettings` defaults used by both server and client builds.
- `src/local.rs`: marker components for local entities.
- Empty placeholders (`src/server/black_list.rs`, `white_list.rs`) exist for future access control logic.

## Build & run commands
> All commands require the sibling path crates listed above. Run them from the repository root.

| Purpose | Command |
| --- | --- |
| Fast type-check | `cargo check` |
| Compile with default (server + client) stack | `cargo build` |
| Server-only build | `cargo build --no-default-features --features server` |
| Client-only build | `cargo build --no-default-features --features client` |
| Auth-enabled build (stacked with desired role) | `cargo build --features "client auth"` or `cargo build --features "server auth"` |
| Unit tests (none defined yet, but catches regressions) | `cargo test` |

Notes:
- The crate is a library; you embed it into a Bevy app binary that sets up states (`ServerVisibility`, `ClientStatus`, etc.) before adding `ChickenNetPlugin`.
- Feature gating is strict: client modules (`src/client.rs`, `singleplayer.rs`) are compiled only when `client` feature is active; server-only targets must opt out of defaults.

## Testing expectations
- No dedicated tests are present, but `cargo test` ensures the code compiles across feature combos. Run it after meaningful changes.
- Because the crate depends on network hardware and async tasks, prefer writing integration tests around Bevy `App` states when adding new logic.

## Key systems & patterns
- **Server lifecycle** (`src/server.rs:37-239`): `ServerLogicPlugin` installs systems reacting to `ServerVisibility` state machine. Systems are scheduled on `OnEnter`, `Update`, or `PreUpdate` stages, often guarded by `run_if` conditions. When adding new server systems, follow this pattern and keep them state-scoped.
- **Networking helpers** (`src/server.rs:314-375`): Utility methods for accepting sessions and locating free ports. Reuse these instead of rolling new socket helpers.
- **Client lifecycle** (`src/client.rs:22-253`): Systems respond to `ClientStatus` transitions and use `Commands::trigger` to drive `states` events. Keep user feedback consistent via `notification::Notify` triggers.
- **Discovery**:
  - Server broadcast (`src/server/discovery.rs:5-60`) answers `FORGE_DISCOVER_V1` probes with the TCP/QUIC port.
  - Client discovery jobs (`src/client/discovery.rs:11-156`) run on the async task pool, deduplicate entries per "generation", and update a shared `DiscoveredServers` list.
- **Shared messaging** (`src/shared.rs:5-20`): register replicon messages with specific channels inside `SharedPlugin` instead of at call sites.
- **Singleplayer orchestration** (`src/singleplayer.rs:16-134`): uses `ChannelIo` to connect local server/client entities and steps through a shutdown state machine. When expanding singleplayer logic, hook into `SingleplayerStatus` / `SingleplayerShutdownStep` states rather than ad-hoc flags.

## Settings & configuration
- `ServerSettings` (`src/settings.rs:5-52`) is registered through `bevy_settings::SettingsPlugin` on both server and client builds (see `src/server.rs:41-44`). Update these structs when new configurable fields are needed, and ensure defaults are meaningful because headless builds may run without config files.
- Discovery ports and visibility flags originate from `ServerSettings`. Systems assume the struct is registered; remember to call `app.register_type::<ServerSettings>()` if you manipulate it outside `ServerLogicPlugin`.

## Gotchas / tips
- **Path dependencies**: Without the sibling `../chicken_*` and `../bevy_*` crates, `cargo` commands fail. Clone or link them before running builds/tests.
- **Feature guards**: Many functions sit behind `#[cfg(feature = ...)]`. When editing shared modules, confirm both client and server feature matrices compile (e.g., `cargo check --no-default-features --features server`).
- **State-driven design**: Both server and client rely on `states` enums (`ServerVisibility`, `ClientStatus`, etc.). New logic should react to these states via `OnEnter/OnExit` or `run_if(in_state(...))` filters; avoid `if` chains on raw resources inside `Update` systems.
- **Networking resources**: Systems often spawn `WebTransportServer`/`Client` entities and attach observers. Respect the observer pattern (`.observe(...)`) for reacting to Aeronet events instead of adding ad-hoc queries.
- **Ports and discovery**: The server automatically checks/adjusts ports when going public (`src/server.rs:111-179`). When changing networking defaults, review these helpers to keep detection logic accurate.
- **Empty scaffolding**: `src/server/black_list.rs` and `white_list.rs` are empty—verify intended behavior or remove unused modules to avoid accidental imports.

Document last updated: 2026-01-31.
