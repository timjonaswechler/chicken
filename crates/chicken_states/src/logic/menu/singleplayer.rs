use {
    crate::{
        events::menu::singleplayer::{
            SetSingleplayerMenu, SetSingleplayerNewGame, SetSingleplayerSavedGame,
        },
        states::{
            menu::{
                main::MainMenuScreen,
                singleplayer::{NewGameMenuScreen, SavedGameMenuScreen, SingleplayerMenuScreen},
            },
            session::{ServerStatus, ServerVisibility, SessionType},
        },
    },
    bevy::prelude::{warn, App, AppExtStates, NextState, On, Plugin, Res, ResMut, State},
};

pub struct SingleplayerMenuPlugin;

impl Plugin for SingleplayerMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SingleplayerMenuScreen>()
            .add_sub_state::<NewGameMenuScreen>()
            .add_sub_state::<SavedGameMenuScreen>()
            .add_observer(on_set_singleplayer_menu)
            .add_observer(on_set_singleplayer_new_game)
            .add_observer(on_set_singleplayer_saved_game);
    }
}

// =============================================================================
// VALIDATOR FUNCTIONS
// =============================================================================

/// Validates transitions from MainMenuScreen to SingleplayerMenuScreen states.
pub(crate) fn is_valid_main_menu_screen_singleplayer_transition(
    from: &MainMenuScreen,
    to: &SetSingleplayerMenu,
) -> bool {
    match (from, to) {
        // From Overview: can go to Singleplayer (Overview event)
        (MainMenuScreen::Overview, SetSingleplayerMenu::Overview) => true,
        // From Singleplayer: can navigate within SingleplayerMenu
        (
            MainMenuScreen::Singleplayer,
            SetSingleplayerMenu::Overview
            | SetSingleplayerMenu::NewGame
            | SetSingleplayerMenu::LoadGame
            | SetSingleplayerMenu::Back,
        ) => true,
        _ => false,
    }
}

/// Validates transitions between SingleplayerMenuScreen states.
pub(crate) fn is_valid_singleplayer_menu_screen_transition(
    from: &SingleplayerMenuScreen,
    to: &SetSingleplayerMenu,
) -> bool {
    matches!(
        (from, to),
        // From Overview: can go to NewGame, LoadGame, or Back (to MainMenu)
        (
            SingleplayerMenuScreen::Overview,
            SetSingleplayerMenu::NewGame
        ) | (
            SingleplayerMenuScreen::Overview,
            SetSingleplayerMenu::LoadGame
        ) | (SingleplayerMenuScreen::Overview, SetSingleplayerMenu::Back) // Note: From NewGame/LoadGame, use SetSingleplayerNewGame::Back or
                                                                          // SetSingleplayerSavedGame::Back to return to Overview.
                                                                          // SetSingleplayerMenu::Back is only valid from Overview.
    )
}

/// Validates transitions from SingleplayerMenuScreen::NewGame to NewGameMenuScreen states.
pub(crate) fn is_valid_singleplayer_menu_screen_new_game_transition(
    from: &SingleplayerMenuScreen,
    to: &SetSingleplayerNewGame,
) -> bool {
    matches!(
        (from, to),
        (
            SingleplayerMenuScreen::NewGame,
            SetSingleplayerNewGame::Next
                | SetSingleplayerNewGame::Previous
                | SetSingleplayerNewGame::Confirm
                | SetSingleplayerNewGame::Back
                | SetSingleplayerNewGame::Cancel
        )
    )
}

/// Validates transitions between NewGameMenuScreen states.
pub(crate) fn is_valid_new_game_menu_screen_transition(
    from: &NewGameMenuScreen,
    to: &SetSingleplayerNewGame,
) -> bool {
    matches!(
        (from, to),
        // ConfigPlayer: Next -> ConfigWorld, Back/Cancel -> parent
        (NewGameMenuScreen::ConfigPlayer, SetSingleplayerNewGame::Next)
            | (NewGameMenuScreen::ConfigPlayer, SetSingleplayerNewGame::Back)
            | (NewGameMenuScreen::ConfigPlayer, SetSingleplayerNewGame::Cancel)
            // ConfigWorld: Next -> ConfigSave, Previous -> ConfigPlayer, Back/Cancel -> parent
            | (NewGameMenuScreen::ConfigWorld, SetSingleplayerNewGame::Next)
            | (NewGameMenuScreen::ConfigWorld, SetSingleplayerNewGame::Previous)
            | (NewGameMenuScreen::ConfigWorld, SetSingleplayerNewGame::Back)
            | (NewGameMenuScreen::ConfigWorld, SetSingleplayerNewGame::Cancel)
            // ConfigSave: Confirm -> start game, Previous -> ConfigWorld, Back/Cancel -> parent
            | (NewGameMenuScreen::ConfigSave, SetSingleplayerNewGame::Confirm)
            | (NewGameMenuScreen::ConfigSave, SetSingleplayerNewGame::Previous)
            | (NewGameMenuScreen::ConfigSave, SetSingleplayerNewGame::Back)
            | (NewGameMenuScreen::ConfigSave, SetSingleplayerNewGame::Cancel)
    )
}

/// Validates transitions from SingleplayerMenuScreen::LoadGame to SavedGameMenuScreen states.
pub(crate) fn is_valid_singleplayer_menu_screen_saved_game_transition(
    from: &SingleplayerMenuScreen,
    to: &SetSingleplayerSavedGame,
) -> bool {
    matches!(
        (from, to),
        (
            SingleplayerMenuScreen::LoadGame,
            SetSingleplayerSavedGame::Next
                | SetSingleplayerSavedGame::Previous
                | SetSingleplayerSavedGame::Confirm
                | SetSingleplayerSavedGame::Back
                | SetSingleplayerSavedGame::Cancel
        )
    )
}

/// Validates transitions between SavedGameMenuScreen states.
pub(crate) fn is_valid_saved_game_menu_screen_transition(
    from: &SavedGameMenuScreen,
    to: &SetSingleplayerSavedGame,
) -> bool {
    matches!(
        (from, to),
        // SelectSaveGame: Confirm -> load game, Back/Cancel -> parent
        // Next/Previous are allowed but may be no-ops for single-state screen
        (
            SavedGameMenuScreen::SelectSaveGame,
            SetSingleplayerSavedGame::Next
        ) | (
            SavedGameMenuScreen::SelectSaveGame,
            SetSingleplayerSavedGame::Previous
        ) | (
            SavedGameMenuScreen::SelectSaveGame,
            SetSingleplayerSavedGame::Confirm
        ) | (
            SavedGameMenuScreen::SelectSaveGame,
            SetSingleplayerSavedGame::Back
        ) | (
            SavedGameMenuScreen::SelectSaveGame,
            SetSingleplayerSavedGame::Cancel
        )
    )
}

// =============================================================================
// OBSERVER FUNCTIONS
// =============================================================================

/// Handles SetSingleplayerMenu events.
fn on_set_singleplayer_menu(
    event: On<SetSingleplayerMenu>,
    current_parent: Res<State<MainMenuScreen>>,
    current: Option<Res<State<SingleplayerMenuScreen>>>,
    mut next_main_menu: ResMut<NextState<MainMenuScreen>>,
    mut next_singleplayer: Option<ResMut<NextState<SingleplayerMenuScreen>>>,
) {
    // Validate parent state transition
    if !is_valid_main_menu_screen_singleplayer_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid MainMenuScreen transition for SetSingleplayerMenu: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Back: Return to MainMenuScreen::Overview (only valid from SingleplayerMenuScreen::Overview)
        SetSingleplayerMenu::Back => {
            // Validate that we're in Overview substate
            if let Some(ref current_state) = current {
                if !is_valid_singleplayer_menu_screen_transition(current_state.get(), event.event())
                {
                    warn!(
                        "Invalid SingleplayerMenuScreen transition for Back: {:?} -> {:?}",
                        current_state.get(),
                        event.event()
                    );
                    return;
                }
            } else {
                warn!("SingleplayerMenuScreen does not exist - cannot process Back event");
                return;
            }
            next_main_menu.set(MainMenuScreen::Overview);
        }
        // Overview: Switch parent to Singleplayer (substate is initialized automatically with default Overview)
        SetSingleplayerMenu::Overview => {
            next_main_menu.set(MainMenuScreen::Singleplayer);
        }
        // NewGame/LoadGame: Set the substate (parent must already be Singleplayer)
        SetSingleplayerMenu::NewGame | SetSingleplayerMenu::LoadGame => {
            // Validate substate transition if we have a current state
            if let Some(ref current_state) = current {
                if !is_valid_singleplayer_menu_screen_transition(current_state.get(), event.event())
                {
                    warn!(
                        "Invalid SingleplayerMenuScreen transition: {:?} -> {:?}",
                        current_state.get(),
                        event.event()
                    );
                    return;
                }
            }

            if let Some(ref mut next) = next_singleplayer {
                match *event.event() {
                    SetSingleplayerMenu::NewGame => next.set(SingleplayerMenuScreen::NewGame),
                    SetSingleplayerMenu::LoadGame => next.set(SingleplayerMenuScreen::LoadGame),
                    _ => {}
                }
            }
        }
    }
}

/// Handles SetSingleplayerNewGame events.
fn on_set_singleplayer_new_game(
    event: On<SetSingleplayerNewGame>,
    current_parent: Res<State<SingleplayerMenuScreen>>,
    current: Option<Res<State<NewGameMenuScreen>>>,
    mut next_singleplayer: ResMut<NextState<SingleplayerMenuScreen>>,
    mut next_new_game: Option<ResMut<NextState<NewGameMenuScreen>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_visibility: ResMut<NextState<ServerVisibility>>,
) {
    // Validate parent state transition
    if !is_valid_singleplayer_menu_screen_new_game_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid SingleplayerMenuScreen transition for SetSingleplayerNewGame: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Back/Cancel: Return to SingleplayerMenuScreen::Overview
        SetSingleplayerNewGame::Back | SetSingleplayerNewGame::Cancel => {
            next_singleplayer.set(SingleplayerMenuScreen::Overview);
        }
        // Confirm: Start the game session
        SetSingleplayerNewGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            next_server_visibility.set(ServerVisibility::Private);
        }
        // Next/Previous: Navigate through config steps
        _ => {
            let current_step = match current {
                Some(c) => *c.get(),
                None => {
                    warn!(
                        "NewGameMenuScreen does not exist - SingleplayerMenuScreen must be NewGame first"
                    );
                    return;
                }
            };

            // Validate step transition
            if !is_valid_new_game_menu_screen_transition(&current_step, event.event()) {
                warn!(
                    "Invalid NewGameMenuScreen transition: {:?} -> {:?}",
                    current_step,
                    event.event()
                );
                return;
            }

            if let Some(ref mut next_step) = next_new_game {
                match (current_step, *event.event()) {
                    (NewGameMenuScreen::ConfigPlayer, SetSingleplayerNewGame::Next) => {
                        next_step.set(NewGameMenuScreen::ConfigWorld);
                    }
                    (NewGameMenuScreen::ConfigWorld, SetSingleplayerNewGame::Next) => {
                        next_step.set(NewGameMenuScreen::ConfigSave);
                    }
                    (NewGameMenuScreen::ConfigWorld, SetSingleplayerNewGame::Previous) => {
                        next_step.set(NewGameMenuScreen::ConfigPlayer);
                    }
                    (NewGameMenuScreen::ConfigSave, SetSingleplayerNewGame::Previous) => {
                        next_step.set(NewGameMenuScreen::ConfigWorld);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Handles SetSingleplayerSavedGame events.
fn on_set_singleplayer_saved_game(
    event: On<SetSingleplayerSavedGame>,
    current_parent: Res<State<SingleplayerMenuScreen>>,
    current: Option<Res<State<SavedGameMenuScreen>>>,
    mut next_singleplayer: ResMut<NextState<SingleplayerMenuScreen>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_visibility: ResMut<NextState<ServerVisibility>>,
) {
    // Validate parent state transition
    if !is_valid_singleplayer_menu_screen_saved_game_transition(current_parent.get(), event.event())
    {
        warn!(
            "Invalid SingleplayerMenuScreen transition for SetSingleplayerSavedGame: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Back/Cancel/Previous: Return to SingleplayerMenuScreen::Overview
        SetSingleplayerSavedGame::Back
        | SetSingleplayerSavedGame::Cancel
        | SetSingleplayerSavedGame::Previous => {
            next_singleplayer.set(SingleplayerMenuScreen::Overview);
        }
        // Confirm: Load the game and start session
        SetSingleplayerSavedGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            next_server_visibility.set(ServerVisibility::Private);
        }
        // Next: Only one state, validate but effectively a no-op
        _ => {
            let current_step = match current {
                Some(c) => *c.get(),
                None => {
                    warn!(
                        "SavedGameMenuScreen does not exist - SingleplayerMenuScreen must be LoadGame first"
                    );
                    return;
                }
            };

            // Validate substate transition
            if !is_valid_saved_game_menu_screen_transition(&current_step, event.event()) {
                warn!(
                    "Invalid SavedGameMenuScreen transition: {:?} -> {:?}",
                    current_step,
                    event.event()
                );
                return;
            }

            // SavedGameMenuScreen has only SelectSaveGame, Next is effectively a no-op
            // or could be used to trigger confirm in the UI layer
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for the Singleplayer Menu state validation and observers.
    //!
    //! These tests verify:
    //! 1. Validator functions (valid/invalid transitions)
    //! 2. Observer logic (events are processed correctly)
    //! 3. Complete workflows (integration tests)

    use crate::events::menu::singleplayer::{
        SetSingleplayerMenu, SetSingleplayerNewGame, SetSingleplayerSavedGame,
    };
    use crate::states::menu::main::MainMenuScreen;
    use crate::states::menu::singleplayer::{
        NewGameMenuScreen, SavedGameMenuScreen, SingleplayerMenuScreen,
    };
    use crate::states::session::{ServerStatus, SessionType};

    mod observer_tests {
        use super::*;

        pub mod helpers {
            use crate::{
                events::{app::SetAppScope, menu::singleplayer::SetSingleplayerMenu},
                states::{
                    app::AppScope,
                    menu::{
                        main::MainMenuScreen,
                        singleplayer::{
                            NewGameMenuScreen, SavedGameMenuScreen, SingleplayerMenuScreen,
                        },
                    },
                    session::{ServerStatus, SessionType},
                },
                ChickenStatePlugin,
            };
            use bevy::{prelude::*, state::app::StatesPlugin};

            /// Creates a test app with all required plugins for Singleplayer Menu tests.
            pub fn test_app() -> App {
                let mut app = App::new();
                app.add_plugins((MinimalPlugins, StatesPlugin, ChickenStatePlugin));

                app
            }

            /// Runs the app for a specified number of update ticks.
            pub fn update_app(app: &mut App, ticks: u8) {
                for _ in 0..ticks {
                    app.update();
                }
            }

            /// Setup helper: Sets MainMenuScreen to Singleplayer and SingleplayerMenuScreen to Overview.
            pub fn setup_test_app_in_overview() -> App {
                let mut app = test_app();
                update_app(&mut app, 1);

                let session_type_state = app.world().resource::<State<SessionType>>();
                assert_eq!(session_type_state.get(), &SessionType::None);

                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Splash);

                app.world_mut().trigger(SetAppScope::Menu);
                update_app(&mut app, 1);

                let setup = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(setup.get(), &MainMenuScreen::Overview);

                app.world_mut().trigger(SetSingleplayerMenu::Overview);
                update_app(&mut app, 1);

                let setup = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(setup.get(), &MainMenuScreen::Singleplayer);

                // Verify initial state
                let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
                assert_eq!(setup.get(), &SingleplayerMenuScreen::Overview);

                app
            }

            /// Setup helper: Sets SingleplayerMenuScreen to NewGame with ConfigPlayer screen.
            pub fn setup_test_app_in_new_game() -> App {
                let mut app = setup_test_app_in_overview();

                // Navigate to NewGame
                app.world_mut().trigger(SetSingleplayerMenu::NewGame);
                update_app(&mut app, 1);

                // Verify state
                let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
                assert_eq!(setup.get(), &SingleplayerMenuScreen::NewGame);

                let screen = app.world().resource::<State<NewGameMenuScreen>>();
                assert_eq!(screen.get(), &NewGameMenuScreen::ConfigPlayer);

                app
            }

            /// Setup helper: Sets SingleplayerMenuScreen to LoadGame.
            pub fn setup_test_app_in_load_game() -> App {
                let mut app = setup_test_app_in_overview();

                // Navigate to LoadGame
                app.world_mut().trigger(SetSingleplayerMenu::LoadGame);
                update_app(&mut app, 1);

                // Verify state
                let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
                assert_eq!(setup.get(), &SingleplayerMenuScreen::LoadGame);

                let screen = app.world().resource::<State<SavedGameMenuScreen>>();
                assert_eq!(screen.get(), &SavedGameMenuScreen::SelectSaveGame);

                app
            }

            /// Asserts that SingleplayerMenuScreen state matches expected value.
            pub fn assert_setup_state(app: &mut App, expected: SingleplayerMenuScreen) {
                let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
                assert_eq!(setup.get(), &expected);
            }

            /// Asserts that MainMenuScreen state matches expected value.
            pub fn assert_main_menu_context(app: &mut App, expected: MainMenuScreen) {
                let context = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(context.get(), &expected);
            }

            /// Asserts that NewGameMenuScreen state matches expected value.
            pub fn assert_new_game_screen(app: &mut App, expected: NewGameMenuScreen) {
                let screen = app.world().resource::<State<NewGameMenuScreen>>();
                assert_eq!(screen.get(), &expected);
            }

            /// Asserts that SavedGameMenuScreen state matches expected value.
            pub fn assert_load_game_screen(app: &mut App, expected: SavedGameMenuScreen) {
                let screen = app.world().resource::<State<SavedGameMenuScreen>>();
                assert_eq!(screen.get(), &expected);
            }

            /// Asserts that SessionType state matches expected value.
            pub fn assert_session_type(app: &mut App, expected: SessionType) {
                let session_type = app.world().resource::<State<SessionType>>();
                assert_eq!(session_type.get(), &expected);
            }

            /// Asserts that ServerStatus state matches expected value.
            pub fn assert_server_status(app: &mut App, expected: ServerStatus) {
                let status = app.world().resource::<State<ServerStatus>>();
                assert_eq!(status.get(), &expected);
            }
        }

        // =============================================================================
        // SetSingleplayerMenu Observer Tests
        // =============================================================================

        /// Test: Overview -> NewGame transition works.
        #[test]

        fn test_observer_overview_to_new_game() {
            let mut app = helpers::setup_test_app_in_overview();

            app.world_mut().trigger(SetSingleplayerMenu::NewGame);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);
        }

        /// Test: Overview -> LoadGame transition works.
        #[test]

        fn test_observer_overview_to_load_game() {
            let mut app = helpers::setup_test_app_in_overview();

            app.world_mut().trigger(SetSingleplayerMenu::LoadGame);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::LoadGame);
        }

        /// Test: Back from Overview returns to MainMenu.
        #[test]

        fn test_observer_overview_back_to_main_menu() {
            let mut app = helpers::setup_test_app_in_overview();

            app.world_mut().trigger(SetSingleplayerMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_context(&mut app, MainMenuScreen::Overview);
        }

        /// Test: SetSingleplayerMenu::Back from NewGame is ignored (must use SetSingleplayerNewGame::Back).
        #[test]
        fn test_observer_set_sp_menu_back_ignored_in_new_game() {
            let mut app = helpers::setup_test_app_in_new_game();

            // SetSingleplayerMenu::Back should be ignored when not in Overview
            app.world_mut().trigger(SetSingleplayerMenu::Back);
            helpers::update_app(&mut app, 1);

            // Should still be in NewGame (not changed)
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);
        }

        /// Test: Proper navigation from NewGame to MainMenu via Overview.
        #[test]
        fn test_observer_new_game_back_to_main_menu_via_overview() {
            let mut app = helpers::setup_test_app_in_new_game();

            // First: Use SetSingleplayerNewGame::Back to return to Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);

            // Then: Use SetSingleplayerMenu::Back to go to MainMenu
            app.world_mut().trigger(SetSingleplayerMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_context(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Invalid transitions are ignored (e.g., NewGame -> LoadGame).
        #[test]

        fn test_observer_invalid_transition_ignored() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Try to go directly from NewGame to LoadGame (should be ignored)
            app.world_mut().trigger(SetSingleplayerMenu::LoadGame);
            helpers::update_app(&mut app, 1);

            // Should still be in NewGame
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);
        }

        // =============================================================================
        // SetSingleplayerNewGame Observer Tests
        // =============================================================================

        /// Test: Next from ConfigPlayer goes to ConfigWorld.
        #[test]

        fn test_observer_new_game_next_config_player_to_config_world() {
            let mut app = helpers::setup_test_app_in_new_game();

            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);
        }

        /// Test: Next from ConfigWorld goes to ConfigSave.
        #[test]

        fn test_observer_new_game_next_config_world_to_config_save() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Go to ConfigWorld first
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Then go to ConfigSave
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigSave);
        }

        /// Test: Previous from ConfigWorld goes back to ConfigPlayer.
        #[test]

        fn test_observer_new_game_previous_config_world_to_config_player() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Go to ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Go back to ConfigPlayer
            app.world_mut().trigger(SetSingleplayerNewGame::Previous);
            helpers::update_app(&mut app, 1);

            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);
        }

        /// Test: Back from ConfigPlayer returns to Overview.
        #[test]

        fn test_observer_new_game_back_from_config_player() {
            let mut app = helpers::setup_test_app_in_new_game();

            app.world_mut().trigger(SetSingleplayerNewGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Cancel returns to Overview from any step.
        #[test]

        fn test_observer_new_game_cancel_returns_to_overview() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Go to ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Cancel should return to Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Confirm starts the singleplayer session.
        #[test]

        fn test_observer_new_game_confirm_starts_session() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Navigate to ConfigSave
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Confirm should start session
            app.world_mut().trigger(SetSingleplayerNewGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Events are ignored when not in NewGame state.
        #[test]

        fn test_observer_new_game_events_ignored_in_wrong_state() {
            let mut app = helpers::setup_test_app_in_overview();

            // Try to trigger NewGame event from Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Should still be in Overview
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        // =============================================================================
        // SetSingleplayerSavedGame Observer Tests
        // =============================================================================

        /// Test: Back from SelectSaveGame returns to Overview.
        #[test]

        fn test_observer_saved_game_back_to_overview() {
            let mut app = helpers::setup_test_app_in_load_game();

            app.world_mut().trigger(SetSingleplayerSavedGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Cancel from SelectSaveGame returns to Overview.
        #[test]

        fn test_observer_saved_game_cancel_to_overview() {
            let mut app = helpers::setup_test_app_in_load_game();

            app.world_mut().trigger(SetSingleplayerSavedGame::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Previous from SelectSaveGame returns to Overview.
        #[test]

        fn test_observer_saved_game_previous_to_overview() {
            let mut app = helpers::setup_test_app_in_load_game();

            app.world_mut().trigger(SetSingleplayerSavedGame::Previous);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Confirm starts the singleplayer session.
        #[test]

        fn test_observer_saved_game_confirm_starts_session() {
            let mut app = helpers::setup_test_app_in_load_game();

            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Events are ignored when not in LoadGame state.
        #[test]

        fn test_observer_saved_game_events_ignored_in_wrong_state() {
            let mut app = helpers::setup_test_app_in_overview();

            // Try to trigger SavedGame event from Overview
            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Should still be in Overview
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);

            // SessionType should still be None
            helpers::assert_session_type(&mut app, SessionType::None);
        }
    }

    mod integration_tests {
        use super::*;

        mod helpers {
            pub use super::super::observer_tests::helpers::*;
        }

        /// Test: Complete NewGame flow from Overview to session start.
        #[test]

        fn test_full_new_game_flow() {
            let mut app = helpers::setup_test_app_in_overview();

            // 1. Navigate to NewGame
            app.world_mut().trigger(SetSingleplayerMenu::NewGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);

            // 2. Next to ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);

            // 3. Next to ConfigSave
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigSave);

            // 4. Confirm starts the session
            app.world_mut().trigger(SetSingleplayerNewGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Complete LoadGame flow from Overview to session start.
        #[test]

        fn test_full_load_game_flow() {
            let mut app = helpers::setup_test_app_in_overview();

            // 1. Navigate to LoadGame
            app.world_mut().trigger(SetSingleplayerMenu::LoadGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::LoadGame);
            helpers::assert_load_game_screen(&mut app, SavedGameMenuScreen::SelectSaveGame);

            // 2. Confirm loads the game and starts session
            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Navigation back from NewGame flow using Previous.
        #[test]

        fn test_new_game_flow_with_back_navigation() {
            let mut app = helpers::setup_test_app_in_overview();

            // Go to NewGame
            app.world_mut().trigger(SetSingleplayerMenu::NewGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);

            // Go to ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);

            // Go back to ConfigPlayer
            app.world_mut().trigger(SetSingleplayerNewGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);

            // Go back to Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Cancel interrupts the NewGame flow.
        #[test]

        fn test_new_game_cancel_mid_flow() {
            let mut app = helpers::setup_test_app_in_overview();

            // Go to NewGame
            app.world_mut().trigger(SetSingleplayerMenu::NewGame);
            helpers::update_app(&mut app, 1);

            // Go to ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Cancel aborts and returns to Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Cancel interrupts the LoadGame flow.
        #[test]
        fn test_load_game_cancel() {
            let mut app = helpers::setup_test_app_in_overview();

            // Go to LoadGame
            app.world_mut().trigger(SetSingleplayerMenu::LoadGame);
            helpers::update_app(&mut app, 1);

            // Cancel aborts and returns to Overview
            app.world_mut().trigger(SetSingleplayerSavedGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Mixed navigation flow - Overview -> NewGame -> Cancel -> LoadGame -> Confirm.
        #[test]
        fn test_mixed_navigation_flow() {
            let mut app = helpers::setup_test_app_in_overview();

            // Go to NewGame
            app.world_mut().trigger(SetSingleplayerMenu::NewGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);

            // Cancel back to Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);

            // Go to LoadGame
            app.world_mut().trigger(SetSingleplayerMenu::LoadGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::LoadGame);

            // Start session
            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Invalid events at each state are properly rejected.
        #[test]
        fn test_invalid_events_rejected_at_each_state() {
            let mut app = helpers::setup_test_app_in_overview();

            // From Overview: Try NewGame event (should be ignored)
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);

            // Go to NewGame
            app.world_mut().trigger(SetSingleplayerMenu::NewGame);
            helpers::update_app(&mut app, 1);

            // From NewGame: Try to go directly to LoadGame (should be ignored)
            app.world_mut().trigger(SetSingleplayerMenu::LoadGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);

            // From NewGame: Try SavedGame event (should be ignored)
            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);
        }
    }

    mod validator_tests {
        use super::*;

        // Import all validator functions
        use super::super::{
            is_valid_main_menu_screen_singleplayer_transition,
            is_valid_new_game_menu_screen_transition, is_valid_saved_game_menu_screen_transition,
            is_valid_singleplayer_menu_screen_new_game_transition,
            is_valid_singleplayer_menu_screen_saved_game_transition,
            is_valid_singleplayer_menu_screen_transition,
        };

        // =============================================================================
        // MainMenuScreen::Singleplayer Validators
        // =============================================================================

        /// Test: Valid transitions from MainMenuScreen::Singleplayer are accepted.
        #[test]
        fn test_valid_main_menu_screen_singleplayer_transitions() {
            // Singleplayer -> Overview (switch to parent state)
            assert!(is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Singleplayer,
                &SetSingleplayerMenu::Overview
            ));

            // Singleplayer -> NewGame
            assert!(is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Singleplayer,
                &SetSingleplayerMenu::NewGame
            ));

            // Singleplayer -> LoadGame
            assert!(is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Singleplayer,
                &SetSingleplayerMenu::LoadGame
            ));

            // Singleplayer -> Back
            assert!(is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Singleplayer,
                &SetSingleplayerMenu::Back
            ));
        }

        /// Test: Valid transition from MainMenuScreen::Overview to Singleplayer.
        #[test]
        fn test_valid_main_menu_overview_to_singleplayer_transition() {
            // Overview -> Overview (switches parent to Singleplayer, substate initialized automatically)
            assert!(is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Overview,
                &SetSingleplayerMenu::Overview
            ));
        }

        /// Test: Invalid transitions from other MainMenuScreen states are rejected.
        #[test]
        fn test_invalid_main_menu_screen_singleplayer_transitions() {
            // Overview -> NewGame/LoadGame/Back is invalid (must go to Overview first to switch parent)
            assert!(!is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Overview,
                &SetSingleplayerMenu::NewGame
            ));
            assert!(!is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Overview,
                &SetSingleplayerMenu::LoadGame
            ));
            assert!(!is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Overview,
                &SetSingleplayerMenu::Back
            ));

            // Multiplayer -> Any SetSingleplayerMenu event is invalid
            assert!(!is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Multiplayer,
                &SetSingleplayerMenu::Overview
            ));
            assert!(!is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Multiplayer,
                &SetSingleplayerMenu::NewGame
            ));

            // Settings -> Any SetSingleplayerMenu event is invalid
            assert!(!is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Settings,
                &SetSingleplayerMenu::Overview
            ));

            // Wiki -> Any SetSingleplayerMenu event is invalid
            assert!(!is_valid_main_menu_screen_singleplayer_transition(
                &MainMenuScreen::Wiki,
                &SetSingleplayerMenu::Back
            ));
        }

        // =============================================================================
        // SingleplayerMenuScreen Validators
        // =============================================================================

        /// Test: Valid transitions from Overview are accepted.
        #[test]

        fn test_valid_singleplayer_menu_screen_transitions_from_overview() {
            // Overview -> NewGame
            assert!(is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::Overview,
                &SetSingleplayerMenu::NewGame
            ));

            // Overview -> LoadGame
            assert!(is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::Overview,
                &SetSingleplayerMenu::LoadGame
            ));

            // Overview -> Back
            assert!(is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::Overview,
                &SetSingleplayerMenu::Back
            ));
        }

        /// Test: SetSingleplayerMenu::Back is invalid from NewGame (must use SetSingleplayerNewGame::Back).
        #[test]
        fn test_invalid_singleplayer_menu_back_from_new_game() {
            // NewGame -> SetSingleplayerMenu::Back is NOT valid
            assert!(!is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerMenu::Back
            ));
        }

        /// Test: SetSingleplayerMenu::Back is invalid from LoadGame (must use SetSingleplayerSavedGame::Back).
        #[test]
        fn test_invalid_singleplayer_menu_back_from_load_game() {
            // LoadGame -> SetSingleplayerMenu::Back is NOT valid
            assert!(!is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerMenu::Back
            ));
        }

        /// Test: Invalid transitions between sub-states are rejected.
        #[test]

        fn test_invalid_singleplayer_menu_screen_transitions() {
            // NewGame cannot go directly to LoadGame
            assert!(!is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerMenu::LoadGame
            ));

            // NewGame cannot go to Overview directly
            assert!(!is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerMenu::Overview
            ));

            // LoadGame cannot go directly to NewGame
            assert!(!is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerMenu::NewGame
            ));

            // LoadGame cannot go to Overview directly
            assert!(!is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerMenu::Overview
            ));

            // Overview -> Overview is not valid (no self-transition)
            assert!(!is_valid_singleplayer_menu_screen_transition(
                &SingleplayerMenuScreen::Overview,
                &SetSingleplayerMenu::Overview
            ));
        }

        // =============================================================================
        // NewGameMenuScreen Validators
        // =============================================================================

        /// Test: Valid transitions for ConfigPlayer are accepted.
        #[test]

        fn test_valid_new_game_transitions_from_config_player() {
            // ConfigPlayer -> Next -> ConfigWorld
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigPlayer,
                &SetSingleplayerNewGame::Next
            ));

            // ConfigPlayer -> Back -> parent
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigPlayer,
                &SetSingleplayerNewGame::Back
            ));

            // ConfigPlayer -> Cancel -> parent
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigPlayer,
                &SetSingleplayerNewGame::Cancel
            ));
        }

        /// Test: Valid transitions for ConfigWorld are accepted.
        #[test]

        fn test_valid_new_game_transitions_from_config_world() {
            // ConfigWorld -> Next -> ConfigSave
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigWorld,
                &SetSingleplayerNewGame::Next
            ));

            // ConfigWorld -> Previous -> ConfigPlayer
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigWorld,
                &SetSingleplayerNewGame::Previous
            ));

            // ConfigWorld -> Back -> parent
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigWorld,
                &SetSingleplayerNewGame::Back
            ));

            // ConfigWorld -> Cancel -> parent
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigWorld,
                &SetSingleplayerNewGame::Cancel
            ));
        }

        /// Test: Valid transitions for ConfigSave are accepted.
        #[test]

        fn test_valid_new_game_transitions_from_config_save() {
            // ConfigSave -> Confirm -> start game
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigSave,
                &SetSingleplayerNewGame::Confirm
            ));

            // ConfigSave -> Previous -> ConfigWorld
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigSave,
                &SetSingleplayerNewGame::Previous
            ));

            // ConfigSave -> Back -> parent
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigSave,
                &SetSingleplayerNewGame::Back
            ));

            // ConfigSave -> Cancel -> parent
            assert!(is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigSave,
                &SetSingleplayerNewGame::Cancel
            ));
        }

        /// Test: Invalid NewGame transitions are rejected.
        #[test]

        fn test_invalid_new_game_transitions() {
            // ConfigPlayer cannot go Previous (already at start)
            assert!(!is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigPlayer,
                &SetSingleplayerNewGame::Previous
            ));

            // ConfigPlayer cannot Confirm (not at final step)
            assert!(!is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigPlayer,
                &SetSingleplayerNewGame::Confirm
            ));

            // ConfigSave cannot go Next (already at end)
            assert!(!is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigSave,
                &SetSingleplayerNewGame::Next
            ));

            // ConfigPlayer cannot Confirm
            assert!(!is_valid_new_game_menu_screen_transition(
                &NewGameMenuScreen::ConfigPlayer,
                &SetSingleplayerNewGame::Confirm
            ));
        }

        /// Test: Parent validation for NewGame events.
        #[test]

        fn test_valid_singleplayer_menu_screen_new_game_transitions() {
            // All events are valid from NewGame state
            assert!(is_valid_singleplayer_menu_screen_new_game_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerNewGame::Next
            ));

            assert!(is_valid_singleplayer_menu_screen_new_game_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerNewGame::Previous
            ));

            assert!(is_valid_singleplayer_menu_screen_new_game_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerNewGame::Confirm
            ));

            assert!(is_valid_singleplayer_menu_screen_new_game_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerNewGame::Back
            ));

            assert!(is_valid_singleplayer_menu_screen_new_game_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerNewGame::Cancel
            ));
        }

        /// Test: Parent validation rejects events from non-NewGame states.
        #[test]

        fn test_invalid_singleplayer_menu_screen_new_game_transitions() {
            // Events from Overview are invalid
            assert!(!is_valid_singleplayer_menu_screen_new_game_transition(
                &SingleplayerMenuScreen::Overview,
                &SetSingleplayerNewGame::Next
            ));

            // Events from LoadGame are invalid
            assert!(!is_valid_singleplayer_menu_screen_new_game_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerNewGame::Confirm
            ));
        }

        // =============================================================================
        // SavedGameMenuScreen Validators
        // =============================================================================

        /// Test: Valid transitions for SelectSaveGame are accepted.
        #[test]

        fn test_valid_saved_game_transitions() {
            // SelectSaveGame -> Confirm -> load game
            assert!(is_valid_saved_game_menu_screen_transition(
                &SavedGameMenuScreen::SelectSaveGame,
                &SetSingleplayerSavedGame::Confirm
            ));

            // SelectSaveGame -> Back -> parent
            assert!(is_valid_saved_game_menu_screen_transition(
                &SavedGameMenuScreen::SelectSaveGame,
                &SetSingleplayerSavedGame::Back
            ));

            // SelectSaveGame -> Cancel -> parent
            assert!(is_valid_saved_game_menu_screen_transition(
                &SavedGameMenuScreen::SelectSaveGame,
                &SetSingleplayerSavedGame::Cancel
            ));

            // SelectSaveGame -> Next (allowed but may be no-op)
            assert!(is_valid_saved_game_menu_screen_transition(
                &SavedGameMenuScreen::SelectSaveGame,
                &SetSingleplayerSavedGame::Next
            ));

            // SelectSaveGame -> Previous (allowed but returns to parent)
            assert!(is_valid_saved_game_menu_screen_transition(
                &SavedGameMenuScreen::SelectSaveGame,
                &SetSingleplayerSavedGame::Previous
            ));
        }

        /// Test: Parent validation for SavedGame events.
        #[test]

        fn test_valid_singleplayer_menu_screen_saved_game_transitions() {
            // All events are valid from LoadGame state
            assert!(is_valid_singleplayer_menu_screen_saved_game_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerSavedGame::Next
            ));

            assert!(is_valid_singleplayer_menu_screen_saved_game_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerSavedGame::Previous
            ));

            assert!(is_valid_singleplayer_menu_screen_saved_game_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerSavedGame::Confirm
            ));

            assert!(is_valid_singleplayer_menu_screen_saved_game_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerSavedGame::Back
            ));

            assert!(is_valid_singleplayer_menu_screen_saved_game_transition(
                &SingleplayerMenuScreen::LoadGame,
                &SetSingleplayerSavedGame::Cancel
            ));
        }

        /// Test: Parent validation rejects events from non-LoadGame states.
        #[test]

        fn test_invalid_singleplayer_menu_screen_saved_game_transitions() {
            // Events from Overview are invalid
            assert!(!is_valid_singleplayer_menu_screen_saved_game_transition(
                &SingleplayerMenuScreen::Overview,
                &SetSingleplayerSavedGame::Confirm
            ));

            // Events from NewGame are invalid
            assert!(!is_valid_singleplayer_menu_screen_saved_game_transition(
                &SingleplayerMenuScreen::NewGame,
                &SetSingleplayerSavedGame::Next
            ));
        }
    }
}
