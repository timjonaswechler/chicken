#![cfg(feature = "hosted")]
#[path = "../common/mod.rs"]
mod common;

use chicken_states::{
    events::session::{SetDisconnectingStep, SetSyncingStep},
    states::{
        app::AppScope,
        session::{ClientConnectionStatus, ConnectingStep, SessionType, SyncingStep},
    },
};

// =============================================================================
// Connecting — Menu → Client (JoinGame → Confirm → Connecting)
// =============================================================================

/// JoinGame → Confirm immediately enters Connecting state (first step: ResolveAddress).
/// Verifies all three state transitions set directly by Confirm.
#[test]
fn test_join_game_confirm_starts_connecting() {
    let mut app = common::setup_test_app_for_client();

    common::assert_app_scope(&mut app, AppScope::Session);
    common::assert_session_type(&mut app, SessionType::Client);
    common::assert_client_status(&mut app, ClientConnectionStatus::Connecting);
    common::assert_connecting_step(&mut app, ConnectingStep::ResolveAddress);
}

/// Full connect flow: Connecting → (all steps) → Connected.
#[test]
fn test_client_connect() {
    let mut app = common::setup_test_app_for_client();

    common::client_connect_complete(&mut app);

    common::assert_client_status(&mut app, ClientConnectionStatus::Connected);
}

// =============================================================================
// Syncing — Happy Path
// =============================================================================

/// Full sync flow: Connected → Syncing → Playing.
#[test]
fn test_client_sync() {
    let mut app = common::setup_test_app_for_client();
    common::client_connect_complete(&mut app);

    common::client_sync_complete(&mut app);

    common::assert_client_status(&mut app, ClientConnectionStatus::Playing);
}

/// After Start, client enters first syncing step (RequestWorld).
#[test]
fn test_client_sync_enters_first_step() {
    let mut app = common::setup_test_app_for_client();
    common::client_connect_complete(&mut app);

    app.world_mut().trigger(SetSyncingStep::Start);
    common::update_app(&mut app, 1);

    common::assert_client_status(&mut app, ClientConnectionStatus::Syncing);
    common::assert_syncing_step(&mut app, SyncingStep::RequestWorld);
}

// =============================================================================
// Disconnecting — Happy Path
// =============================================================================

/// Full disconnect flow: Playing → Disconnecting → AppScope::Menu.
#[test]
fn test_client_disconnect() {
    let mut app = common::setup_test_app_for_client();
    common::client_connect_complete(&mut app);
    common::client_sync_complete(&mut app);

    app.world_mut().trigger(SetDisconnectingStep::Start);
    common::update_app(&mut app, 1);
    common::assert_client_status(&mut app, ClientConnectionStatus::Disconnecting);

    common::client_disconnect_complete(&mut app);
    common::assert_app_scope(&mut app, AppScope::Menu);
    common::assert_session_type(&mut app, SessionType::None);
}

// =============================================================================
// Full Lifecycle
// =============================================================================

/// Full client lifecycle from JoinGame: Confirm → Connected → Playing → back to Menu.
#[test]
fn test_full_client_lifecycle() {
    let mut app = common::setup_test_app_for_client();

    common::client_connect_complete(&mut app);
    common::assert_client_status(&mut app, ClientConnectionStatus::Connected);

    common::client_sync_complete(&mut app);
    common::assert_client_status(&mut app, ClientConnectionStatus::Playing);

    common::client_disconnect_complete(&mut app);
    common::assert_app_scope(&mut app, AppScope::Menu);
    common::assert_session_type(&mut app, SessionType::None);
}

// =============================================================================
// Observer Guards
// =============================================================================

/// Guard: syncing cannot start from Connecting (only from Connected).
#[test]
fn test_client_sync_blocked_when_connecting() {
    let mut app = common::setup_test_app_for_client();

    // Client is in Connecting — syncing must be ignored
    app.world_mut().trigger(SetSyncingStep::Start);
    common::update_app(&mut app, 1);

    common::assert_client_status(&mut app, ClientConnectionStatus::Connecting);
}
