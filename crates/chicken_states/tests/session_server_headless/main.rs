#![cfg(feature = "headless")]
#[path = "../common/mod.rs"]
mod common;

use bevy::prelude::State;
use chicken_states::{
    events::session::{
        SetGoingPrivateStep, SetGoingPublicStep, SetServerShutdownStep, SetServerStartupStep,
    },
    states::session::{
        ServerShutdownStep, ServerStartupStep, ServerStatus, ServerVisibility, SessionType,
    },
};

// =============================================================================
// Startup Tests
// =============================================================================

/// Startup failure at various steps — server stays in Starting, AppExit::error() is triggered.
/// (No Offline state in headless; the binary exits on failure instead of returning to idle.)
#[test]
fn test_dedicated_server_startup_failure() {
    for fail_step in 0..=common::SERVER_STARTUP_STEPS {
        let mut app = common::setup_test_app_headless();
        // App starts in Starting::Init — no Start trigger needed

        // Advance to the failure point
        for _ in 0..fail_step {
            app.world_mut().trigger(SetServerStartupStep::Next);
            common::update_app(&mut app, 1);
        }

        // Trigger failure
        app.world_mut().trigger(SetServerStartupStep::Failed);
        common::update_app(&mut app, 1);

        // State stays Starting — in production the binary would exit via AppExit::error()
        common::assert_server_status(&mut app, ServerStatus::Starting);
    }
}

// =============================================================================
// Shutdown Tests
// =============================================================================

/// Full dedicated server shutdown: Running → shutdown steps.
/// Note: Headless has NO DespawnLocalClient step (3 Next calls instead of 4).
/// On Done, AppExit::Success is triggered instead of returning to Offline.
#[test]
fn test_dedicated_server_shutdown() {
    let mut app = common::setup_test_app_headless_running();

    app.world_mut().trigger(SetServerShutdownStep::Start);
    common::update_app(&mut app, 1);

    common::assert_server_status(&mut app, ServerStatus::Stopping);
    assert_eq!(
        app.world().resource::<State<ServerShutdownStep>>().get(),
        &ServerShutdownStep::SaveWorld
    );

    for _ in 0..common::SERVER_SHUTDOWN_STEPS_HEADLESS {
        app.world_mut().trigger(SetServerShutdownStep::Next);
        common::update_app(&mut app, 1);
    }
    app.world_mut().trigger(SetServerShutdownStep::Done);
    common::update_app(&mut app, 1);

    // State stays Stopping — in production the binary would exit via AppExit::Success
    common::assert_server_status(&mut app, ServerStatus::Stopping);
}

/// Shutdown failure at various steps — server stays in Stopping, AppExit::error() is triggered.
#[test]
fn test_dedicated_server_shutdown_failure() {
    for fail_step in 0..=common::SERVER_SHUTDOWN_STEPS_HEADLESS {
        let mut app = common::setup_test_app_headless_running();

        app.world_mut().trigger(SetServerShutdownStep::Start);
        common::update_app(&mut app, 1);

        for _ in 0..fail_step {
            app.world_mut().trigger(SetServerShutdownStep::Next);
            common::update_app(&mut app, 1);
        }

        app.world_mut().trigger(SetServerShutdownStep::Failed);
        common::update_app(&mut app, 1);

        // State stays Stopping — in production the binary would exit via AppExit::error()
        common::assert_server_status(&mut app, ServerStatus::Stopping);
    }
}

/// Guard: shutdown during startup is blocked.
#[test]
fn test_dedicated_server_shutdown_blocked_during_startup() {
    let mut app = common::setup_test_app_headless();
    // App starts in Starting::Init — no Start trigger needed

    // Try to shutdown during startup — must be blocked
    app.world_mut().trigger(SetServerShutdownStep::Start);
    common::update_app(&mut app, 1);

    common::assert_server_status(&mut app, ServerStatus::Starting);
}

// =============================================================================
// Visibility Tests
// =============================================================================

/// Server goes from Private → GoingPublic → Public.
#[test]
fn test_dedicated_server_goes_public() {
    let mut app = common::setup_test_app_headless_running();

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::Private);

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

/// Server goes from Public → GoingPrivate → Private.
#[test]
fn test_dedicated_server_goes_private() {
    let mut app = common::setup_test_app_headless_running();

    // First go public
    app.world_mut().trigger(SetGoingPublicStep::Start);
    common::update_app(&mut app, 1);
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

    // Now go private
    app.world_mut().trigger(SetGoingPrivateStep::Start);
    common::update_app(&mut app, 1);

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::GoingPrivate);

    for _ in 0..common::SERVER_GOING_PRIVATE_STEPS {
        app.world_mut().trigger(SetGoingPrivateStep::Next);
        common::update_app(&mut app, 1);
    }
    app.world_mut().trigger(SetGoingPrivateStep::Done);
    common::update_app(&mut app, 1);

    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::Private);
}

// =============================================================================
// Full Lifecycle Test
// =============================================================================

/// Full lifecycle: Starting → Running → GoingPublic → Public → GoingPrivate → Private → Shutdown.
/// (No Start trigger — headless defaults to Starting::Init immediately on launch.)
#[test]
fn test_full_dedicated_server_lifecycle() {
    let mut app = common::setup_test_app_headless();
    // App already in Starting::Init

    // 1. Complete startup
    for _ in 0..common::SERVER_STARTUP_STEPS {
        app.world_mut().trigger(SetServerStartupStep::Next);
        common::update_app(&mut app, 1);
    }
    app.world_mut().trigger(SetServerStartupStep::Done);
    common::update_app(&mut app, 1);
    common::assert_server_status(&mut app, ServerStatus::Running);

    // 2. Go Public
    app.world_mut().trigger(SetGoingPublicStep::Start);
    common::update_app(&mut app, 1);
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

    // 3. Go Private
    app.world_mut().trigger(SetGoingPrivateStep::Start);
    common::update_app(&mut app, 1);
    for _ in 0..common::SERVER_GOING_PRIVATE_STEPS {
        app.world_mut().trigger(SetGoingPrivateStep::Next);
        common::update_app(&mut app, 1);
    }
    app.world_mut().trigger(SetGoingPrivateStep::Done);
    common::update_app(&mut app, 1);
    let visibility = app
        .world()
        .resource::<bevy::prelude::State<ServerVisibility>>();
    assert_eq!(visibility.get(), &ServerVisibility::Private);

    // 4. Shutdown
    app.world_mut().trigger(SetServerShutdownStep::Start);
    common::update_app(&mut app, 1);
    common::assert_server_status(&mut app, ServerStatus::Stopping);

    for _ in 0..common::SERVER_SHUTDOWN_STEPS_HEADLESS {
        app.world_mut().trigger(SetServerShutdownStep::Next);
        common::update_app(&mut app, 1);
    }
    app.world_mut().trigger(SetServerShutdownStep::Done);
    common::update_app(&mut app, 1);

    // State stays Stopping — in production the binary exits via AppExit::Success
    common::assert_server_status(&mut app, ServerStatus::Stopping);
    common::assert_session_type(&mut app, SessionType::DedicatedServer);
}
