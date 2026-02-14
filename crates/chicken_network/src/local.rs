use bevy::prelude::*;

/// Marker component for entities participating in a local singleplayer session.
///
/// Used to identify and query entities that are part of the local client-server loop
/// in singleplayer mode, distinguishing them from networked multiplayer entities.
#[derive(Component)]
pub struct LocalSession;

/// Marker component for the local client entity in singleplayer mode.
///
/// Attached to the client entity that connects to the local server via in-memory
/// channels. Used to identify local client-specific systems and resources.
#[derive(Component)]
pub struct LocalClient;

/// Marker component for the local server entity in singleplayer mode.
///
/// Attached to the server entity that hosts the game locally via in-memory
/// channels. Used to identify local server-specific systems and resources.
#[derive(Component)]
pub struct LocalServer;

/// Marker component for AI-controlled bot entities.
///
/// Used to distinguish bot players from human players in both singleplayer
/// and multiplayer sessions. Bots may be spawned locally or replicated from server.
#[derive(Component)]
pub struct LocalBot;
