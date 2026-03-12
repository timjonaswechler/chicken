#![cfg(feature = "hosted")]
#[path = "../common/mod.rs"]
mod common;

use chicken_states::{
    events::session::{SetGoingPublicStep, SetServerShutdownStep},
    states::{
        app::AppScope,
        session::{ServerStatus, ServerVisibility, SessionType},
    },
};

// =============================================================================
// Singleplayer — Menu → Server Startup
// =============================================================================

/// Full singleplayer flow: NewGame menu → Confirm → server reaches Running.
#[test]
fn test_singleplayer_server_startup_from_menu() {
    let mut app = common::setup_test_app_singleplayer_confirmed();

    common::server_startup_complete(&mut app);

    common::assert_server_status(&mut app, ServerStatus::Running);
}

/// Singleplayer server starts Private by default (no PendingGoingPublic).
#[test]
fn test_singleplayer_server_starts_private() {
    let mut app = common::setup_test_app_singleplayer_confirmed();

    common::server_startup_complete(&mut app);

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::Private);
}

/// Full singleplayer shutdown: Running → Stopping → AppScope::Menu.
#[test]
fn test_singleplayer_server_shutdown() {
    let mut app = common::setup_test_app_singleplayer_confirmed();
    common::server_startup_complete(&mut app);

    app.world_mut().trigger(SetServerShutdownStep::Start);
    common::update_app(&mut app, 1);
    common::assert_server_status(&mut app, ServerStatus::Stopping);

    for _ in 0..common::SERVER_SHUTDOWN_STEPS {
        app.world_mut().trigger(SetServerShutdownStep::Next);
        common::update_app(&mut app, 1);
    }
    app.world_mut().trigger(SetServerShutdownStep::Done);
    common::update_app(&mut app, 1);

    common::assert_app_scope(&mut app, AppScope::Menu);
    common::assert_session_type(&mut app, SessionType::None);
}

// =============================================================================
// Multiplayer Host — Menu → Server Startup → GoingPublic
// =============================================================================

/// Full multiplayer host flow: HostNewGame → Confirm → server reaches Running → GoingPublic.
#[test]
fn test_multiplayer_host_server_startup_from_menu() {
    let mut app = common::setup_test_app_multiplayer_host_confirmed();

    common::server_startup_complete(&mut app);

    common::assert_server_status(&mut app, ServerStatus::Running);
}

/// Multiplayer host server goes public after startup (PendingGoingPublic present).
#[test]
fn test_multiplayer_host_server_goes_public() {
    let mut app = common::setup_test_app_multiplayer_host_confirmed();
    common::server_startup_complete(&mut app);

    // PendingGoingPublic triggers GoingPublic automatically or manually
    app.world_mut().trigger(SetGoingPublicStep::Start);
    common::update_app(&mut app, 1);

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::GoingPublic);

    for _ in 0..common::SERVER_GOING_PUBLIC_STEPS {
        app.world_mut().trigger(SetGoingPublicStep::Next);
        common::update_app(&mut app, 1);
    }
    app.world_mut().trigger(SetGoingPublicStep::Done);
    common::update_app(&mut app, 1);

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::Public);
}

// =============================================================================
// Singleplayer SavedGame — Menu → Server Startup
// =============================================================================

/// Full singleplayer flow: LoadGame menu → Confirm → server reaches Running.
#[test]
fn test_singleplayer_saved_server_startup_from_menu() {
    let mut app = common::setup_test_app_singleplayer_saved_confirmed();

    common::server_startup_complete(&mut app);

    common::assert_server_status(&mut app, ServerStatus::Running);
}

/// Singleplayer saved-game server starts Private by default.
#[test]
fn test_singleplayer_saved_server_starts_private() {
    let mut app = common::setup_test_app_singleplayer_saved_confirmed();

    common::server_startup_complete(&mut app);

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::Private);
}

// =============================================================================
// Multiplayer HostSavedGame — Menu → Server Startup → GoingPublic
// =============================================================================

/// Full multiplayer saved-game host flow: HostSavedGame → Confirm → server reaches Running.
#[test]
fn test_multiplayer_saved_host_server_startup_from_menu() {
    let mut app = common::setup_test_app_multiplayer_saved_host_confirmed();

    common::server_startup_complete(&mut app);

    common::assert_server_status(&mut app, ServerStatus::Running);
}

/// Multiplayer saved-game host server goes public after startup (PendingGoingPublic present).
#[test]
fn test_multiplayer_saved_host_server_goes_public() {
    let mut app = common::setup_test_app_multiplayer_saved_host_confirmed();
    common::server_startup_complete(&mut app);

    app.world_mut().trigger(SetGoingPublicStep::Start);
    common::update_app(&mut app, 1);

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::GoingPublic);

    for _ in 0..common::SERVER_GOING_PUBLIC_STEPS {
        app.world_mut().trigger(SetGoingPublicStep::Next);
        common::update_app(&mut app, 1);
    }
    app.world_mut().trigger(SetGoingPublicStep::Done);
    common::update_app(&mut app, 1);

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::Public);
}

// =============================================================================
// Observer Guards
// =============================================================================

/// Guard: shutdown is blocked when server is not Running.
#[test]
fn test_server_shutdown_blocked_during_startup() {
    let mut app = common::setup_test_app_singleplayer_confirmed();

    // Server is in Starting — shutdown must be ignored
    app.world_mut().trigger(SetServerShutdownStep::Start);
    common::update_app(&mut app, 1);

    common::assert_server_status(&mut app, ServerStatus::Starting);
}
