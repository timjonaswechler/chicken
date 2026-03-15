use {
    crate::{
        events::menu::singleplayer::{
            SetSingleplayerMenu, SetSingleplayerNewGame, SetSingleplayerSavedGame,
        },
        events::app::SetAppScope,
        states::{
            app::AppScope,
            menu::{
                main::MainMenuScreen,
                singleplayer::{NewGameMenuScreen, SavedGameMenuScreen, SingleplayerMenuScreen},
            },
            session::{ServerStatus, SessionType},
        },
    },
    bevy::prelude::{App, AppExtStates, Commands, NextState, On, Plugin, Res, ResMut, State, warn},
};

pub(super) struct SingleplayerMenuPlugin;

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
    matches!(
        (from, to),
        // From Overview: can go to Singleplayer (Overview event)
        (MainMenuScreen::Overview, SetSingleplayerMenu::Overview)
            | (
                MainMenuScreen::Singleplayer,
                SetSingleplayerMenu::Overview
                    | SetSingleplayerMenu::NewGame
                    | SetSingleplayerMenu::LoadGame
                    | SetSingleplayerMenu::Back,
            )
    )
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
        (
            NewGameMenuScreen::ConfigPlayer,
            SetSingleplayerNewGame::Next
        ) | (
            NewGameMenuScreen::ConfigPlayer,
            SetSingleplayerNewGame::Cancel
        ) | (NewGameMenuScreen::ConfigWorld, SetSingleplayerNewGame::Next)
            | (
                NewGameMenuScreen::ConfigWorld,
                SetSingleplayerNewGame::Previous
            )
            | (
                NewGameMenuScreen::ConfigWorld,
                SetSingleplayerNewGame::Cancel
            )
            | (
                NewGameMenuScreen::ConfigSave,
                SetSingleplayerNewGame::Confirm
            )
            | (
                NewGameMenuScreen::ConfigSave,
                SetSingleplayerNewGame::Previous
            )
            | (
                NewGameMenuScreen::ConfigSave,
                SetSingleplayerNewGame::Cancel
            )
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
    mut commands: Commands,
    current_parent: Res<State<SingleplayerMenuScreen>>,
    current: Option<Res<State<NewGameMenuScreen>>>,
    mut next_singleplayer: ResMut<NextState<SingleplayerMenuScreen>>,
    mut next_new_game: Option<ResMut<NextState<NewGameMenuScreen>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
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
        SetSingleplayerNewGame::Cancel => {
            next_singleplayer.set(SingleplayerMenuScreen::Overview);
        }
        // Confirm: Start the game session (only valid from ConfigSave)
        SetSingleplayerNewGame::Confirm => {
            let current_step = match current {
                Some(ref c) => *c.get(),
                None => {
                    warn!("NewGameMenuScreen does not exist - cannot Confirm");
                    return;
                }
            };
            if !is_valid_new_game_menu_screen_transition(&current_step, event.event()) {
                warn!(
                    "Invalid NewGameMenuScreen transition: {:?} -> Confirm",
                    current_step
                );
                return;
            }
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            commands.trigger(SetAppScope::Session);
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
    mut commands: Commands,
    current_parent: Res<State<SingleplayerMenuScreen>>,
    current: Option<Res<State<SavedGameMenuScreen>>>,
    mut next_singleplayer: ResMut<NextState<SingleplayerMenuScreen>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
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
        // Cancel: Return to SingleplayerMenuScreen::Overview
        SetSingleplayerSavedGame::Cancel => {
            next_singleplayer.set(SingleplayerMenuScreen::Overview);
        }
        // Confirm: Load the game and start session
        SetSingleplayerSavedGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            commands.trigger(SetAppScope::Session);
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
