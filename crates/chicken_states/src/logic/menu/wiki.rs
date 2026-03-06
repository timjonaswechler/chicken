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
        (MainMenuScreen::Wiki, SetWikiMenu::Creatures)
            | (MainMenuScreen::Wiki, SetWikiMenu::Weapons)
            | (MainMenuScreen::Wiki, SetWikiMenu::Armor)
            | (MainMenuScreen::Wiki, SetWikiMenu::Back)
    )
}

/// Validates transitions between WikiMenuScreen states.
/// Navigation to categories is only allowed from Overview.
pub(crate) fn is_valid_wiki_menu_screen_transition(
    from: &WikiMenuScreen,
    to: &SetWikiMenu,
) -> bool {
    match (from, to) {
        // From Overview: can go to any category or Back (to MainMenu)
        (WikiMenuScreen::Overview, SetWikiMenu::Creatures) => true,
        (WikiMenuScreen::Overview, SetWikiMenu::Weapons) => true,
        (WikiMenuScreen::Overview, SetWikiMenu::Armor) => true,
        (WikiMenuScreen::Overview, SetWikiMenu::Back) => true,

        // From categories: only Back to Overview is allowed
        (WikiMenuScreen::Creatures, SetWikiMenu::Back) => true,
        (WikiMenuScreen::Weapons, SetWikiMenu::Back) => true,
        (WikiMenuScreen::Armor, SetWikiMenu::Back) => true,

        // All other transitions are invalid
        _ => false,
    }
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

    // Step 2: Get current substate (must exist when parent is Wiki)
    let current_state = match current {
        Some(c) => *c.get(),
        None => {
            warn!("WikiMenuScreen does not exist - MainMenuScreen must be Wiki first");
            return;
        }
    };

    // Step 3: Validate substate transition
    if !is_valid_wiki_menu_screen_transition(&current_state, event.event()) {
        warn!(
            "Invalid WikiMenuScreen transition: {:?} -> {:?}",
            current_state,
            event.event()
        );
        return;
    }

    // Step 4: Execute state transition
    match *event.event() {
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
            // From Overview: Back goes to MainMenu
            // From categories: Back goes to Overview
            if current_state == WikiMenuScreen::Overview {
                next_main_screen.set(MainMenuScreen::Overview);
            } else {
                next_wiki_screen.set(WikiMenuScreen::Overview);
            }
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
            // From categories: only Back to Overview is allowed
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

        /// Test: Ungültige WikiMenuScreen-Übergänge werden blockiert.
        #[test]
        fn test_invalid_wiki_transitions() {
            // From Creatures: cannot go directly to Weapons
            assert!(!is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Creatures,
                &SetWikiMenu::Weapons
            ));

            // From Weapons: cannot go directly to Armor
            assert!(!is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Weapons,
                &SetWikiMenu::Armor
            ));

            // From Armor: cannot go directly to Creatures
            assert!(!is_valid_wiki_menu_screen_transition(
                &WikiMenuScreen::Armor,
                &SetWikiMenu::Creatures
            ));
        }
    }

    mod observer_tests {
        use super::*;

        pub mod helpers {
            use crate::{
                events::menu::wiki::SetWikiMenu,
                logic::menu::wiki::WikiMenuPlugin,
                states::menu::{main::MainMenuScreen, wiki::WikiMenuScreen},
            };
            use bevy::{prelude::*, state::app::StatesPlugin};

            /// Creates a test app with all required plugins for Wiki Menu tests.
            pub fn test_app() -> App {
                let mut app = App::new();
                app.add_plugins((MinimalPlugins, StatesPlugin, WikiMenuPlugin));

                // Initialize parent state required for sub-state
                app.init_state::<MainMenuScreen>();

                app
            }

            /// Runs the app for a specified number of update ticks.
            pub fn update_app(app: &mut App, ticks: u8) {
                for _ in 0..ticks {
                    app.update();
                }
            }

            /// Setup helper: Sets MainMenuScreen to Wiki and WikiMenuScreen to Overview.
            pub fn setup_test_app_in_wiki_overview() -> App {
                let mut app = test_app();
                update_app(&mut app, 1);

                // Set MainMenuScreen to Wiki (parent of WikiMenuScreen)
                let mut next_main_menu =
                    app.world_mut().resource_mut::<NextState<MainMenuScreen>>();
                next_main_menu.set(MainMenuScreen::Wiki);
                update_app(&mut app, 1);

                // Verify initial state
                let wiki = app.world().resource::<State<WikiMenuScreen>>();
                assert_eq!(wiki.get(), &WikiMenuScreen::Overview);

                app
            }

            /// Setup helper: Sets WikiMenuScreen to Creatures.
            pub fn setup_test_app_in_creatures() -> App {
                let mut app = setup_test_app_in_wiki_overview();

                // Navigate to Creatures
                app.world_mut().trigger(SetWikiMenu::Creatures);
                update_app(&mut app, 1);

                // Verify state
                let wiki = app.world().resource::<State<WikiMenuScreen>>();
                assert_eq!(wiki.get(), &WikiMenuScreen::Creatures);

                app
            }

            /// Asserts that WikiMenuScreen state matches expected value.
            pub fn assert_wiki_state(app: &mut App, expected: WikiMenuScreen) {
                let wiki = app.world().resource::<State<WikiMenuScreen>>();
                assert_eq!(wiki.get(), &expected);
            }

            /// Asserts that MainMenuScreen state matches expected value.
            pub fn assert_main_menu_state(app: &mut App, expected: MainMenuScreen) {
                let main = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(main.get(), &expected);
            }
        }

        /// Test: Overview -> Creatures transition works.
        #[test]
        fn test_observer_overview_to_creatures() {
            let mut app = helpers::setup_test_app_in_wiki_overview();

            app.world_mut().trigger(SetWikiMenu::Creatures);
            helpers::update_app(&mut app, 1);

            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Creatures);
        }

        /// Test: Overview -> Weapons transition works.
        #[test]
        fn test_observer_overview_to_weapons() {
            let mut app = helpers::setup_test_app_in_wiki_overview();

            app.world_mut().trigger(SetWikiMenu::Weapons);
            helpers::update_app(&mut app, 1);

            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Weapons);
        }

        /// Test: Overview -> Armor transition works.
        #[test]
        fn test_observer_overview_to_armor() {
            let mut app = helpers::setup_test_app_in_wiki_overview();

            app.world_mut().trigger(SetWikiMenu::Armor);
            helpers::update_app(&mut app, 1);

            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Armor);
        }

        /// Test: Back from Overview returns to MainMenu.
        #[test]
        fn test_observer_overview_back_to_main_menu() {
            let mut app = helpers::setup_test_app_in_wiki_overview();

            app.world_mut().trigger(SetWikiMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_state(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Back from Creatures returns to Overview.
        #[test]
        fn test_observer_creatures_back_to_overview() {
            let mut app = helpers::setup_test_app_in_creatures();

            app.world_mut().trigger(SetWikiMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Overview);
            // MainMenuScreen should still be Wiki
            helpers::assert_main_menu_state(&mut app, MainMenuScreen::Wiki);
        }

        /// Test: Invalid transitions are ignored (e.g., Creatures -> Weapons).
        #[test]
        fn test_observer_invalid_transition_ignored() {
            let mut app = helpers::setup_test_app_in_creatures();

            // Try to go directly from Creatures to Weapons (should be ignored)
            app.world_mut().trigger(SetWikiMenu::Weapons);
            helpers::update_app(&mut app, 1);

            // Should still be in Creatures
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Creatures);
        }

        /// Test: Events are ignored when parent is not Wiki.
        #[test]
        fn test_observer_events_ignored_without_wiki_parent() {
            let mut app = helpers::test_app();
            helpers::update_app(&mut app, 1);

            // MainMenuScreen is still Overview (not Wiki)
            // Try to trigger Wiki event
            app.world_mut().trigger(SetWikiMenu::Creatures);
            helpers::update_app(&mut app, 1);

            // Should still be in Overview (MainMenu), not Creatures
            helpers::assert_main_menu_state(&mut app, MainMenuScreen::Overview);
        }
    }

    mod integration_tests {
        use super::*;

        pub mod helpers {
            pub use super::super::observer_tests::helpers::*;
        }

        /// Test: Complete Wiki workflow -> Overview -> Creatures -> Back -> MainMenu.
        #[test]
        fn test_wiki_workflow() {
            let mut app = helpers::setup_test_app_in_wiki_overview();

            // Go to Creatures
            app.world_mut().trigger(SetWikiMenu::Creatures);
            helpers::update_app(&mut app, 1);
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Creatures);

            // Back to Overview
            app.world_mut().trigger(SetWikiMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Overview);

            // Back to MainMenu
            app.world_mut().trigger(SetWikiMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_main_menu_state(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Navigation between categories only via Overview.
        #[test]
        fn test_wiki_category_navigation_via_overview() {
            let mut app = helpers::setup_test_app_in_wiki_overview();

            // Go to Creatures
            app.world_mut().trigger(SetWikiMenu::Creatures);
            helpers::update_app(&mut app, 1);
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Creatures);

            // Cannot go directly to Weapons - must go via Overview
            app.world_mut().trigger(SetWikiMenu::Weapons);
            helpers::update_app(&mut app, 1);
            // Should still be in Creatures
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Creatures);

            // Go back to Overview first
            app.world_mut().trigger(SetWikiMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Overview);

            // Now can go to Weapons
            app.world_mut().trigger(SetWikiMenu::Weapons);
            helpers::update_app(&mut app, 1);
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Weapons);
        }

        /// Test: Complete cycle through all categories.
        #[test]
        fn test_wiki_all_categories_cycle() {
            let mut app = helpers::setup_test_app_in_wiki_overview();

            // Overview -> Creatures
            app.world_mut().trigger(SetWikiMenu::Creatures);
            helpers::update_app(&mut app, 1);
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Creatures);

            // Creatures -> Overview -> Weapons
            app.world_mut().trigger(SetWikiMenu::Back);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetWikiMenu::Weapons);
            helpers::update_app(&mut app, 1);
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Weapons);

            // Weapons -> Overview -> Armor
            app.world_mut().trigger(SetWikiMenu::Back);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetWikiMenu::Armor);
            helpers::update_app(&mut app, 1);
            helpers::assert_wiki_state(&mut app, WikiMenuScreen::Armor);
        }
    }
}
