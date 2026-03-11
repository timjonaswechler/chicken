#![cfg(feature = "hosted")]
#[path = "../common/mod.rs"]
mod common;

use chicken_states::{
    events::menu::singleplayer::{SetSingleplayerMenu, SetSingleplayerNewGame, SetSingleplayerSavedGame},
    states::{
        menu::{
            main::MainMenuScreen,
            singleplayer::{NewGameMenuScreen, SavedGameMenuScreen, SingleplayerMenuScreen},
        },
        session::{ServerStatus, SessionType},
    },
};

// =============================================================================
// SetSingleplayerMenu — Bevy Plumbing
// =============================================================================

/// Back from SingleplayerMenuScreen::Overview returns to MainMenuScreen::Overview.
#[test]
fn test_singleplayer_back_returns_to_main_menu() {
    let mut app = common::setup_test_app_in_singleplayer();

    app.world_mut().trigger(SetSingleplayerMenu::Back);
    common::update_app(&mut app, 1);

    common::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
}

// =============================================================================
// NewGame — Happy Path & Navigation
// =============================================================================

/// Full flow: ConfigPlayer -> ConfigWorld -> ConfigSave -> Confirm starts session.
#[test]
fn test_new_game_full_flow() {
    let mut app = common::setup_test_app_in_new_game();

    app.world_mut().trigger(SetSingleplayerNewGame::Next);
    common::update_app(&mut app, 1);
    common::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);

    app.world_mut().trigger(SetSingleplayerNewGame::Next);
    common::update_app(&mut app, 1);
    common::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigSave);

    app.world_mut().trigger(SetSingleplayerNewGame::Confirm);
    common::update_app(&mut app, 1);

    common::assert_session_type(&mut app, SessionType::Singleplayer);
    common::assert_server_status(&mut app, ServerStatus::Starting);
}

/// Previous navigation: ConfigWorld -> Previous -> ConfigPlayer.
#[test]
fn test_new_game_previous_navigation() {
    let mut app = common::setup_test_app_in_new_game();

    app.world_mut().trigger(SetSingleplayerNewGame::Next);
    common::update_app(&mut app, 1);
    common::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);

    app.world_mut().trigger(SetSingleplayerNewGame::Previous);
    common::update_app(&mut app, 1);
    common::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);
}

/// Cancel from mid-flow returns to SingleplayerMenuScreen::Overview.
#[test]
fn test_new_game_cancel_returns_to_overview() {
    let mut app = common::setup_test_app_in_new_game();

    app.world_mut().trigger(SetSingleplayerNewGame::Next);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetSingleplayerNewGame::Cancel);
    common::update_app(&mut app, 1);

    common::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
}

/// Observer guard: Confirm from ConfigPlayer is ignored (only valid from ConfigSave).
#[test]
fn test_new_game_confirm_blocked_before_config_save() {
    let mut app = common::setup_test_app_in_new_game();

    // Still on ConfigPlayer — Confirm must be ignored
    app.world_mut().trigger(SetSingleplayerNewGame::Confirm);
    common::update_app(&mut app, 1);

    // Session must not have started
    common::assert_session_type(&mut app, SessionType::None);
}

// =============================================================================
// LoadGame — Happy Path & Navigation
// =============================================================================

/// Confirm from SelectSaveGame starts session.
#[test]
fn test_load_game_confirm_starts_session() {
    let mut app = common::setup_test_app_in_load_game();

    app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
    common::update_app(&mut app, 1);

    common::assert_session_type(&mut app, SessionType::Singleplayer);
    common::assert_server_status(&mut app, ServerStatus::Starting);
}

/// Cancel returns to SingleplayerMenuScreen::Overview.
#[test]
fn test_load_game_cancel_returns_to_overview() {
    let mut app = common::setup_test_app_in_load_game();

    app.world_mut().trigger(SetSingleplayerSavedGame::Cancel);
    common::update_app(&mut app, 1);

    common::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
}

/// Previous is a no-op in LoadGame (SelectSaveGame is the only step).
#[test]
fn test_load_game_previous_is_noop() {
    let mut app = common::setup_test_app_in_load_game();

    app.world_mut().trigger(SetSingleplayerSavedGame::Previous);
    common::update_app(&mut app, 1);

    common::assert_load_game_screen(&mut app, SavedGameMenuScreen::SelectSaveGame);
}
