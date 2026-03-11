#[path = "../common/mod.rs"]
mod common;

#[cfg(feature = "hosted")]
use chicken_states::{
    events::app::SetAppScope,
    states::{app::AppScope, session::SessionType},
};

// =============================================================================
// AppScope — Bevy Plumbing
// =============================================================================

/// App starts in Splash state.
#[cfg(feature = "hosted")]
#[test]
fn test_app_starts_in_splash() {
    let mut app = common::test_app();
    common::update_app(&mut app, 1);

    common::assert_app_scope(&mut app, AppScope::Splash);
}

/// Splash -> Menu transition works.
#[cfg(feature = "hosted")]
#[test]
fn test_splash_to_menu() {
    let mut app = common::setup_test_app_hosted();

    common::assert_app_scope(&mut app, AppScope::Menu);
}

/// Menu -> Session transition works.
#[cfg(feature = "hosted")]
#[test]
fn test_menu_to_session() {
    let mut app = common::setup_test_app_hosted();

    app.world_mut().trigger(SetAppScope::Session);
    common::update_app(&mut app, 1);

    common::assert_app_scope(&mut app, AppScope::Session);
}

/// Session -> Menu resets SessionType to None (side effect).
#[cfg(feature = "hosted")]
#[test]
fn test_session_to_menu_resets_session_type() {
    let mut app = common::setup_test_app_hosted();

    app.world_mut().trigger(SetAppScope::Session);
    common::update_app(&mut app, 1);

    // Set a non-None session type to verify it gets reset
    app.world_mut()
        .resource_mut::<bevy::prelude::NextState<SessionType>>()
        .set(SessionType::Singleplayer);
    common::update_app(&mut app, 1);
    common::assert_session_type(&mut app, SessionType::Singleplayer);

    app.world_mut().trigger(SetAppScope::Menu);
    common::update_app(&mut app, 1);

    common::assert_app_scope(&mut app, AppScope::Menu);
    common::assert_session_type(&mut app, SessionType::None);
}

/// Observer guard: Splash -> Session is blocked.
#[cfg(feature = "hosted")]
#[test]
fn test_splash_to_session_is_blocked() {
    let mut app = common::test_app();
    common::update_app(&mut app, 1);
    common::assert_app_scope(&mut app, AppScope::Splash);

    app.world_mut().trigger(SetAppScope::Session);
    common::update_app(&mut app, 1);

    // Must still be in Splash
    common::assert_app_scope(&mut app, AppScope::Splash);
}

#[cfg(feature = "headless")]
#[test]
fn test_headless_starts_in_session() {
    let _ = common::setup_test_app_headless();
}
