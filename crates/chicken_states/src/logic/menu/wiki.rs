//! Wiki/help menu navigation logic.
//!
//! Handles state transitions for the in-game documentation browser:
//! - Navigation between wiki categories (Creatures, Weapons, Armor)
//! - Back navigation to main menu
//!
//! Provides free navigation between wiki categories without requiring
//! return to the overview screen.

use {
    crate::{
        events::menu::wiki::SetWikiMenu,
        states::menu::{main::MainMenuScreen, wiki::WikiMenuScreen},
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State, warn},
};

pub(super) struct WikiMenuPlugin;

impl Plugin for WikiMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<WikiMenuScreen>()
            .add_observer(on_set_wiki_menu);
    }
}

// --- VALIDATOR FUNCTIONS ---

/// Validates transitions from MainMenuScreen::Wiki.
/// Checks if the event is valid when the parent state is Wiki.
pub(crate) fn is_valid_main_menu_screen_wiki_transition(
    from: &MainMenuScreen,
    to: &SetWikiMenu,
) -> bool {
    matches!(
        (from, to),
        (MainMenuScreen::Overview, SetWikiMenu::Overview) | (MainMenuScreen::Wiki, _)
    )
}

/// Validates transitions between WikiMenuScreen states.
/// Navigation to categories is only allowed from Overview.
pub(crate) fn is_valid_wiki_menu_screen_transition(
    from: &WikiMenuScreen,
    to: &SetWikiMenu,
) -> bool {
    matches!(
        (from, to),
        // From Overview: can go to any category or Back (to MainMenu)
        (_, SetWikiMenu::Creatures)
            | (_, SetWikiMenu::Weapons)
            | (_, SetWikiMenu::Armor)
            | (_, SetWikiMenu::Back)
    )
}

// --- OBSERVER FUNCTIONS ---

fn on_set_wiki_menu(
    event: On<SetWikiMenu>,
    current_parent: Res<State<MainMenuScreen>>,
    current: Option<Res<State<WikiMenuScreen>>>,
    mut next_main_screen: ResMut<NextState<MainMenuScreen>>,
    mut next_wiki_screen: ResMut<NextState<WikiMenuScreen>>,
) {
    // Step 1: Validate parent state transition
    if !is_valid_main_menu_screen_wiki_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid MainMenuScreen transition for SetWikiMenu: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        SetWikiMenu::Overview => {
            next_main_screen.set(MainMenuScreen::Wiki);
        }
        SetWikiMenu::Creatures => {
            next_wiki_screen.set(WikiMenuScreen::Creatures);
        }
        SetWikiMenu::Weapons => {
            next_wiki_screen.set(WikiMenuScreen::Weapons);
        }
        SetWikiMenu::Armor => {
            next_wiki_screen.set(WikiMenuScreen::Armor);
        }
        SetWikiMenu::Back => {
            if let Some(ref current_state) = current {
                if !is_valid_wiki_menu_screen_transition(current_state.get(), event.event()) {
                    warn!(
                        "Invalid WikiMenuScreen transition: {:?} -> {:?}",
                        current_state,
                        event.event()
                    );
                    return;
                }
            } else {
                warn!("WikiMenuScreen does not exist - cannot process Back event");
                return;
            }
            next_main_screen.set(MainMenuScreen::Overview);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die Wiki Menu Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Validator-Funktionen (ob Übergänge gültig/ungültig sind)
    //! 2. Observer-Logik (ob Events korrekt verarbeitet werden)
    //! 3. Integration-Tests (komplette Workflows)

    use crate::events::menu::wiki::SetWikiMenu;
    use crate::states::menu::{main::MainMenuScreen, wiki::WikiMenuScreen};

    mod validator_tests {
        use super::*;
        use crate::logic::menu::wiki::{
            is_valid_main_menu_screen_wiki_transition, is_valid_wiki_menu_screen_transition,
        };

        /// Test: Gültige MainMenuScreen::Wiki-Übergänge werden akzeptiert.
        #[test]
        fn test_valid_main_menu_screen_wiki_transitions() {
            // All category events are valid from Wiki
            assert!(is_valid_main_menu_screen_wiki_transition(
                &MainMenuScreen::Wiki,
                &SetWikiMenu::Creatures
            ));
            assert!(is_valid_main_menu_screen_wiki_transition(
                &MainMenuScreen::Wiki,
                &SetWikiMenu::Weapons
            ));
            assert!(is_valid_main_menu_screen_wiki_transition(
                &MainMenuScreen::Wiki,
                &SetWikiMenu::Armor
            ));
            assert!(is_valid_main_menu_screen_wiki_transition(
                &MainMenuScreen::Wiki,
                &SetWikiMenu::Back
            ));
        }

        /// Test: Ungültige MainMenuScreen-Übergänge werden blockiert.
        #[test]
        fn test_invalid_main_menu_screen_wiki_transitions() {
            // Events are not valid from other MainMenuScreen states
            assert!(!is_valid_main_menu_screen_wiki_transition(
                &MainMenuScreen::Overview,
                &SetWikiMenu::Creatures
            ));
            assert!(!is_valid_main_menu_screen_wiki_transition(
                &MainMenuScreen::Singleplayer,
                &SetWikiMenu::Back
            ));
            assert!(!is_valid_main_menu_screen_wiki_transition(
                &MainMenuScreen::Settings,
                &SetWikiMenu::Weapons
            ));
            assert!(!is_valid_main_menu_screen_wiki_transition(
                &MainMenuScreen::Multiplayer,
                &SetWikiMenu::Armor
            ));
        }

        /// Test: Gültige WikiMenuScreen-Übergänge von Overview werden akzeptiert.
        #[test]
        fn test_valid_wiki_transitions_from_overview() {
            // From Overview: can go to any category or Back
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Overview,
                &SetWikiMenu::Creatures
            ));
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Overview,
                &SetWikiMenu::Weapons
            ));
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Overview,
                &SetWikiMenu::Armor
            ));
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Overview,
                &SetWikiMenu::Back
            ));
        }

        /// Test: Gültige WikiMenuScreen-Übergänge von Kategorien werden akzeptiert.
        #[test]
        fn test_valid_wiki_transitions_from_categories() {
            // From categories: Back is always valid (returns to MainMenu)
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Creatures,
                &SetWikiMenu::Back
            ));
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Weapons,
                &SetWikiMenu::Back
            ));
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Armor,
                &SetWikiMenu::Back
            ));
        }

        /// Test: Wiki allows free navigation between any categories.
        #[test]
        fn test_wiki_free_navigation() {
            // Wiki allows browsing between categories without returning to Overview
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Creatures,
                &SetWikiMenu::Weapons
            ));
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Weapons,
                &SetWikiMenu::Armor
            ));
            assert!(is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Armor,
                &SetWikiMenu::Creatures
            ));
        }
    }
}
