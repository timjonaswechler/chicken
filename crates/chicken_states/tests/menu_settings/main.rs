#![cfg(feature = "hosted")]
#[path = "../common/mod.rs"]
mod common;

use chicken_states::{
    events::menu::settings::SetSettingsMenu,
    states::menu::{main::MainMenuScreen, settings::SettingsMenuScreen},
};

// =============================================================================
// SetSettingsMenu — Bevy Plumbing
// =============================================================================

/// Happy path: Overview -> Audio navigation works.
#[test]
fn test_settings_navigate_to_category() {
    let mut app = common::setup_test_app_in_settings();

    app.world_mut().trigger(SetSettingsMenu::Audio);
    common::update_app(&mut app, 1);

    common::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
}

/// Back from Overview returns to MainMenuScreen::Overview.
#[test]
fn test_settings_back_from_overview_returns_to_main_menu() {
    let mut app = common::setup_test_app_in_settings();

    app.world_mut().trigger(SetSettingsMenu::Back);
    common::update_app(&mut app, 1);

    common::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
}

/// Back from a category returns to SettingsMenuScreen::Overview (not MainMenu).
#[test]
fn test_settings_back_from_category_returns_to_settings_overview() {
    let mut app = common::setup_test_app_in_settings();

    app.world_mut().trigger(SetSettingsMenu::Audio);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetSettingsMenu::Back);
    common::update_app(&mut app, 1);

    common::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);
    // Must still be in Settings, not in MainMenu
    common::assert_main_menu_screen(&mut app, MainMenuScreen::Settings);
}

/// Cancel from a category returns to SettingsMenuScreen::Overview (not MainMenu).
#[test]
fn test_settings_cancel_returns_to_settings_overview() {
    let mut app = common::setup_test_app_in_settings();

    app.world_mut().trigger(SetSettingsMenu::Video);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetSettingsMenu::Cancel);
    common::update_app(&mut app, 1);

    common::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);
    common::assert_main_menu_screen(&mut app, MainMenuScreen::Settings);
}

/// Apply is a no-op (stays in current category).
#[test]
fn test_settings_apply_is_noop() {
    let mut app = common::setup_test_app_in_settings();

    app.world_mut().trigger(SetSettingsMenu::Audio);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetSettingsMenu::Apply);
    common::update_app(&mut app, 1);

    common::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
}

/// Observer guard: direct jump from Audio to Video is blocked.
#[test]
fn test_settings_direct_category_jump_is_blocked() {
    let mut app = common::setup_test_app_in_settings();

    app.world_mut().trigger(SetSettingsMenu::Audio);
    common::update_app(&mut app, 1);

    // Try to jump directly to Video — must be ignored
    app.world_mut().trigger(SetSettingsMenu::Video);
    common::update_app(&mut app, 1);

    common::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
}

/// Observer guard: Back outside of Settings is ignored.
#[test]
fn test_settings_back_ignored_outside_settings() {
    let mut app = common::setup_test_app_hosted();

    // Still in MainMenuScreen::Overview — Back must be ignored
    app.world_mut().trigger(SetSettingsMenu::Back);
    common::update_app(&mut app, 1);

    common::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
}
