#![cfg(feature = "hosted")]
#[path = "../common/mod.rs"]
mod common;

use chicken_states::{
    events::menu::multiplayer::{
        SetJoinGame, SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame,
    },
    states::{
        menu::{
            main::MainMenuScreen,
            multiplayer::{
                HostNewGameMenuScreen, HostSavedGameMenuScreen, JoinGameMenuScreen,
                MultiplayerMenuScreen,
            },
        },
        session::{ServerStatus, SessionType},
    },
};

// =============================================================================
// SetMultiplayerMenu — Bevy Plumbing
// =============================================================================

/// Back from MultiplayerMenuScreen::Overview returns to MainMenuScreen::Overview.
#[test]
fn test_multiplayer_back_returns_to_main_menu() {
    let mut app = common::setup_test_app_in_multiplayer_overview();

    app.world_mut().trigger(SetMultiplayerMenu::Back);
    common::update_app(&mut app, 1);

    common::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
}

// =============================================================================
// HostNewGame — Happy Path & Navigation
// =============================================================================

/// Full flow: ConfigServer -> ConfigWorld -> ConfigSave -> Confirm starts session.
#[test]
fn test_host_new_game_full_flow() {
    let mut app = common::setup_test_app_in_host_new_game();

    app.world_mut().trigger(SetNewHostGame::Next);
    common::update_app(&mut app, 1);
    common::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

    app.world_mut().trigger(SetNewHostGame::Next);
    common::update_app(&mut app, 1);
    common::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);

    app.world_mut().trigger(SetNewHostGame::Confirm);
    common::update_app(&mut app, 1);

    common::assert_session_type(&mut app, SessionType::Singleplayer);
    common::assert_server_status(&mut app, ServerStatus::Starting);
}

/// Previous navigation: ConfigWorld -> Previous -> ConfigServer.
#[test]
fn test_host_new_game_previous_navigation() {
    let mut app = common::setup_test_app_in_host_new_game();

    app.world_mut().trigger(SetNewHostGame::Next);
    common::update_app(&mut app, 1);
    common::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

    app.world_mut().trigger(SetNewHostGame::Previous);
    common::update_app(&mut app, 1);
    common::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);
}

/// Cancel from mid-flow returns to MultiplayerMenuScreen::Overview.
#[test]
fn test_host_new_game_cancel_returns_to_overview() {
    let mut app = common::setup_test_app_in_host_new_game();

    app.world_mut().trigger(SetNewHostGame::Next);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetNewHostGame::Cancel);
    common::update_app(&mut app, 1);

    common::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
}

/// Observer guard: Confirm from ConfigServer is ignored (only valid from ConfigSave).
#[test]
fn test_host_new_game_confirm_blocked_before_config_save() {
    let mut app = common::setup_test_app_in_host_new_game();

    // Still on ConfigServer — Confirm must be ignored
    app.world_mut().trigger(SetNewHostGame::Confirm);
    common::update_app(&mut app, 1);

    // Session must not have started
    common::assert_session_type(&mut app, SessionType::None);
}

// =============================================================================
// HostSavedGame — Happy Path & Navigation
// =============================================================================

/// Full flow: Overview -> ConfigServer -> Confirm starts session.
#[test]
fn test_host_saved_game_full_flow() {
    let mut app = common::setup_test_app_in_host_saved_game();

    app.world_mut().trigger(SetSavedHostGame::Next);
    common::update_app(&mut app, 1);
    common::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

    app.world_mut().trigger(SetSavedHostGame::Confirm);
    common::update_app(&mut app, 1);

    common::assert_session_type(&mut app, SessionType::Singleplayer);
    common::assert_server_status(&mut app, ServerStatus::Starting);
}

/// Previous navigation: ConfigServer -> Previous -> Overview.
#[test]
fn test_host_saved_game_previous_navigation() {
    let mut app = common::setup_test_app_in_host_saved_game();

    app.world_mut().trigger(SetSavedHostGame::Next);
    common::update_app(&mut app, 1);
    common::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

    app.world_mut().trigger(SetSavedHostGame::Previous);
    common::update_app(&mut app, 1);
    common::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);
}

/// Cancel from ConfigServer returns to MultiplayerMenuScreen::Overview.
#[test]
fn test_host_saved_game_cancel_returns_to_overview() {
    let mut app = common::setup_test_app_in_host_saved_game();

    app.world_mut().trigger(SetSavedHostGame::Next);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetSavedHostGame::Cancel);
    common::update_app(&mut app, 1);

    common::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
}

/// Observer guard: Confirm from Overview is ignored (only valid from ConfigServer).
#[test]
fn test_host_saved_game_confirm_blocked_on_overview() {
    let mut app = common::setup_test_app_in_host_saved_game();

    // Still on Overview — Confirm must be ignored
    app.world_mut().trigger(SetSavedHostGame::Confirm);
    common::update_app(&mut app, 1);

    // Session must not have started
    common::assert_session_type(&mut app, SessionType::None);
}

// =============================================================================
// JoinGame — Happy Path & Navigation
// =============================================================================

/// Confirm stays in JoinGame::Overview (connection is triggered externally).
#[test]
fn test_join_game_confirm_stays_in_overview() {
    let mut app = common::setup_test_app_in_join_game();

    app.world_mut().trigger(SetJoinGame::Confirm);
    common::update_app(&mut app, 1);
}

/// Cancel returns to MultiplayerMenuScreen::Overview.
#[test]
fn test_join_game_cancel_returns_to_overview() {
    let mut app = common::setup_test_app_in_join_game();

    app.world_mut().trigger(SetJoinGame::Cancel);
    common::update_app(&mut app, 1);

    common::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
}

/// Next is a no-op in JoinGame (only Confirm triggers connection).
#[test]
fn test_join_game_next_is_noop() {
    let mut app = common::setup_test_app_in_join_game();

    app.world_mut().trigger(SetJoinGame::Next);
    common::update_app(&mut app, 1);

    common::assert_join_game_screen(&mut app, JoinGameMenuScreen::Overview);
}
