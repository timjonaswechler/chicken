#![cfg(feature = "hosted")]
#[path = "../common/mod.rs"]
mod common;

use chicken_states::{
    events::menu::wiki::SetWikiMenu,
    states::menu::{main::MainMenuScreen, wiki::WikiMenuScreen},
};

// =============================================================================
// SetWikiMenu — Bevy Plumbing
// =============================================================================

/// Happy path: Overview -> Creatures navigation works.
#[test]
fn test_wiki_navigate_to_category() {
    let mut app = common::setup_test_app_hosted();

    app.world_mut().trigger(SetWikiMenu::Overview);
    common::update_app(&mut app, 1);
    common::assert_wiki_state(&mut app, WikiMenuScreen::Overview);

    app.world_mut().trigger(SetWikiMenu::Creatures);
    common::update_app(&mut app, 1);
    common::assert_wiki_state(&mut app, WikiMenuScreen::Creatures);
}

/// Free navigation between categories without returning to Overview.
#[test]
fn test_wiki_free_navigation_between_categories() {
    let mut app = common::setup_test_app_hosted();

    app.world_mut().trigger(SetWikiMenu::Overview);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetWikiMenu::Creatures);
    common::update_app(&mut app, 1);
    common::assert_wiki_state(&mut app, WikiMenuScreen::Creatures);

    app.world_mut().trigger(SetWikiMenu::Weapons);
    common::update_app(&mut app, 1);
    common::assert_wiki_state(&mut app, WikiMenuScreen::Weapons);

    app.world_mut().trigger(SetWikiMenu::Armor);
    common::update_app(&mut app, 1);
    common::assert_wiki_state(&mut app, WikiMenuScreen::Armor);
}

/// Back from any wiki screen returns to MainMenuScreen::Overview.
#[test]
fn test_wiki_back_returns_to_main_menu() {
    let mut app = common::setup_test_app_hosted();

    app.world_mut().trigger(SetWikiMenu::Overview);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetWikiMenu::Creatures);
    common::update_app(&mut app, 1);

    app.world_mut().trigger(SetWikiMenu::Back);
    common::update_app(&mut app, 1);
    common::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
}

/// Observer guard: Back is ignored when not in Wiki.
#[test]
fn test_wiki_back_ignored_outside_wiki() {
    let mut app = common::setup_test_app_hosted();

    // Still in MainMenuScreen::Overview — Back must be ignored
    app.world_mut().trigger(SetWikiMenu::Back);
    common::update_app(&mut app, 1);

    common::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
}
