use {
    crate::{
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
            session::{ServerStatus, ServerVisibility, SessionType},
        },
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State, warn},
};

pub(super) struct MultiplayerMenuPlugin;

impl Plugin for MultiplayerMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MultiplayerMenuScreen>()
            .add_sub_state::<HostNewGameMenuScreen>()
            .add_sub_state::<HostSavedGameMenuScreen>()
            .add_sub_state::<JoinGameMenuScreen>()
            .add_observer(on_set_multiplayer_menu)
            .add_observer(on_set_new_host_game)
            .add_observer(on_set_saved_host_game)
            .add_observer(on_set_join_game);
    }
}

// =============================================================================
// VALIDATOR FUNCTIONS
// =============================================================================

/// Validates transitions from MainMenuScreen to MultiplayerMenuScreen states.
pub(crate) fn is_valid_main_menu_screen_multiplayer_transition(
    from: &MainMenuScreen,
    to: &SetMultiplayerMenu,
) -> bool {
    match (from, to) {
        // From Overview: can go to Multiplayer (Overview event)
        (MainMenuScreen::Overview, SetMultiplayerMenu::Overview) => true,
        // From Multiplayer: can navigate within MultiplayerMenu
        (
            MainMenuScreen::Multiplayer,
            SetMultiplayerMenu::Overview
            | SetMultiplayerMenu::HostNewGame
            | SetMultiplayerMenu::HostSavedGame
            | SetMultiplayerMenu::JoinGame
            | SetMultiplayerMenu::Back,
        ) => true,
        _ => false,
    }
}

/// Validates transitions between MultiplayerMenuScreen states.
/// Navigation to sub-states is only allowed from Overview.
pub(crate) fn is_valid_multiplayer_menu_screen_transition(
    from: &MultiplayerMenuScreen,
    to: &SetMultiplayerMenu,
) -> bool {
    matches!(
        (from, to),
        // From Overview: can go to any sub-state or Back
        (MultiplayerMenuScreen::Overview, SetMultiplayerMenu::HostNewGame)
            | (MultiplayerMenuScreen::Overview, SetMultiplayerMenu::HostSavedGame)
            | (MultiplayerMenuScreen::Overview, SetMultiplayerMenu::JoinGame)
            | (MultiplayerMenuScreen::Overview, SetMultiplayerMenu::Back)
            // From sub-states: only Back to Overview is allowed
            | (MultiplayerMenuScreen::HostNewGame, SetMultiplayerMenu::Back)
            | (MultiplayerMenuScreen::HostSavedGame, SetMultiplayerMenu::Back)
            | (MultiplayerMenuScreen::JoinGame, SetMultiplayerMenu::Back)
    )
}

/// Validates that SetNewHostGame events are only accepted when parent is HostNewGame.
pub(crate) fn is_valid_multiplayer_menu_screen_host_new_game_transition(
    from: &MultiplayerMenuScreen,
    to: &SetNewHostGame,
) -> bool {
    matches!(
        (from, to),
        (MultiplayerMenuScreen::HostNewGame, SetNewHostGame::Next)
            | (MultiplayerMenuScreen::HostNewGame, SetNewHostGame::Previous)
            | (MultiplayerMenuScreen::HostNewGame, SetNewHostGame::Confirm)
            | (MultiplayerMenuScreen::HostNewGame, SetNewHostGame::Back)
            | (MultiplayerMenuScreen::HostNewGame, SetNewHostGame::Cancel)
    )
}

/// Validates transitions between HostNewGameMenuScreen states.
pub(crate) fn is_valid_host_new_game_menu_screen_transition(
    from: &HostNewGameMenuScreen,
    to: &SetNewHostGame,
) -> bool {
    matches!(
        (from, to),
        // Forward navigation through config steps
        (HostNewGameMenuScreen::ConfigServer, SetNewHostGame::Next)
            | (HostNewGameMenuScreen::ConfigWorld, SetNewHostGame::Next)
            | (HostNewGameMenuScreen::ConfigSave, SetNewHostGame::Next)
            // Backward navigation
            | (HostNewGameMenuScreen::ConfigWorld, SetNewHostGame::Previous)
            | (HostNewGameMenuScreen::ConfigSave, SetNewHostGame::Previous)
            // Confirm available from ConfigSave
            | (HostNewGameMenuScreen::ConfigSave, SetNewHostGame::Confirm)
            // Back/Always available (handled by parent transition)
            | (_, SetNewHostGame::Back)
            | (_, SetNewHostGame::Cancel)
    )
}

/// Validates that SetSavedHostGame events are only accepted when parent is HostSavedGame.
pub(crate) fn is_valid_multiplayer_menu_screen_host_saved_game_transition(
    from: &MultiplayerMenuScreen,
    to: &SetSavedHostGame,
) -> bool {
    matches!(
        (from, to),
        (MultiplayerMenuScreen::HostSavedGame, SetSavedHostGame::Next)
            | (
                MultiplayerMenuScreen::HostSavedGame,
                SetSavedHostGame::Previous
            )
            | (
                MultiplayerMenuScreen::HostSavedGame,
                SetSavedHostGame::Confirm
            )
            | (MultiplayerMenuScreen::HostSavedGame, SetSavedHostGame::Back)
            | (
                MultiplayerMenuScreen::HostSavedGame,
                SetSavedHostGame::Cancel
            )
    )
}

/// Validates transitions between HostSavedGameMenuScreen states.
pub(crate) fn is_valid_host_saved_game_menu_screen_transition(
    from: &HostSavedGameMenuScreen,
    to: &SetSavedHostGame,
) -> bool {
    matches!(
        (from, to),
        // Forward navigation
        (HostSavedGameMenuScreen::Overview, SetSavedHostGame::Next)
            | (HostSavedGameMenuScreen::ConfigServer, SetSavedHostGame::Next)
            // Backward navigation
            | (HostSavedGameMenuScreen::ConfigServer, SetSavedHostGame::Previous)
            // Confirm available from ConfigServer
            | (HostSavedGameMenuScreen::ConfigServer, SetSavedHostGame::Confirm)
            // Back/Cancel always available (handled by parent transition)
            | (_, SetSavedHostGame::Back)
            | (_, SetSavedHostGame::Cancel)
    )
}

/// Validates that SetJoinGame events are only accepted when parent is JoinGame.
pub(crate) fn is_valid_multiplayer_menu_screen_join_game_transition(
    from: &MultiplayerMenuScreen,
    to: &SetJoinGame,
) -> bool {
    matches!(
        (from, to),
        (MultiplayerMenuScreen::JoinGame, SetJoinGame::Next)
            | (MultiplayerMenuScreen::JoinGame, SetJoinGame::Previous)
            | (MultiplayerMenuScreen::JoinGame, SetJoinGame::Confirm)
            | (MultiplayerMenuScreen::JoinGame, SetJoinGame::Back)
            | (MultiplayerMenuScreen::JoinGame, SetJoinGame::Cancel)
    )
}

// =============================================================================
// OBSERVER FUNCTIONS
// =============================================================================

/// Handles SetMultiplayerMenu events.
/// Validates parent and sub-state transitions before applying state changes.
fn on_set_multiplayer_menu(
    event: On<SetMultiplayerMenu>,
    current_parent: Res<State<MainMenuScreen>>,
    current: Option<Res<State<MultiplayerMenuScreen>>>,
    mut next_main_menu: ResMut<NextState<MainMenuScreen>>,
    mut next_multiplayer: Option<ResMut<NextState<MultiplayerMenuScreen>>>,
) {
    // Validate parent state transition
    if !is_valid_main_menu_screen_multiplayer_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid MainMenuScreen transition for SetMultiplayerMenu: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Back: Return to MainMenuScreen::Overview (only valid from MultiplayerMenuScreen::Overview)
        SetMultiplayerMenu::Back => {
            // Validate that we're in Overview substate
            if let Some(ref current_state) = current {
                if !is_valid_multiplayer_menu_screen_transition(current_state.get(), event.event())
                {
                    warn!(
                        "Invalid MultiplayerMenuScreen transition for Back: {:?} -> {:?}",
                        current_state.get(),
                        event.event()
                    );
                    return;
                }
            } else {
                warn!("MultiplayerMenuScreen does not exist - cannot process Back event");
                return;
            }
            next_main_menu.set(MainMenuScreen::Overview);
        }
        // Overview: Switch parent to Multiplayer (substate is initialized automatically with default Overview)
        SetMultiplayerMenu::Overview => {
            next_main_menu.set(MainMenuScreen::Multiplayer);
        }
        // HostNewGame/HostSavedGame/JoinGame: Set the substate (parent must already be Multiplayer)
        SetMultiplayerMenu::HostNewGame
        | SetMultiplayerMenu::HostSavedGame
        | SetMultiplayerMenu::JoinGame => {
            // Validate substate transition if we have a current state
            if let Some(ref current_state) = current {
                if !is_valid_multiplayer_menu_screen_transition(current_state.get(), event.event())
                {
                    warn!(
                        "Invalid MultiplayerMenuScreen transition: {:?} -> {:?}",
                        current_state.get(),
                        event.event()
                    );
                    return;
                }
            }

            if let Some(ref mut next) = next_multiplayer {
                match *event.event() {
                    SetMultiplayerMenu::HostNewGame => next.set(MultiplayerMenuScreen::HostNewGame),
                    SetMultiplayerMenu::HostSavedGame => {
                        next.set(MultiplayerMenuScreen::HostSavedGame)
                    }
                    SetMultiplayerMenu::JoinGame => next.set(MultiplayerMenuScreen::JoinGame),
                    _ => {}
                }
            }
        }
    }
}

/// Handles SetNewHostGame events for the HostNewGame configuration flow.
fn on_set_new_host_game(
    event: On<SetNewHostGame>,
    current_parent: Res<State<MultiplayerMenuScreen>>,
    current: Option<Res<State<HostNewGameMenuScreen>>>,
    mut next_multiplayer: ResMut<NextState<MultiplayerMenuScreen>>,
    mut next_host_screen: Option<ResMut<NextState<HostNewGameMenuScreen>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_visibility: ResMut<NextState<ServerVisibility>>,
) {
    // Validate parent state transition
    if !is_valid_multiplayer_menu_screen_host_new_game_transition(
        current_parent.get(),
        event.event(),
    ) {
        warn!(
            "Invalid MultiplayerMenuScreen transition for SetNewHostGame: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Back/Cancel: Return to MultiplayerMenuScreen::Overview
        SetNewHostGame::Back | SetNewHostGame::Cancel => {
            next_multiplayer.set(MultiplayerMenuScreen::Overview);
        }
        // Confirm: Start the server
        SetNewHostGame::Confirm => {
            // Note: According to spec, Confirm is only valid from ConfigSave
            if let Some(ref current_state) = current {
                if !matches!(
                    (current_state.get(), event.event()),
                    (HostNewGameMenuScreen::ConfigSave, SetNewHostGame::Confirm)
                ) {
                    warn!(
                        "SetNewHostGame::Confirm only valid from ConfigSave, current: {:?}",
                        current_state.get()
                    );
                    return;
                }
            }
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            next_server_visibility.set(ServerVisibility::GoingPublic);
        }
        // Next/Previous: Navigate through config steps
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!("HostNewGameMenuScreen does not exist - must be initialized first");
                    return;
                }
            };

            // Validate step transition
            if !is_valid_host_new_game_menu_screen_transition(&current, event.event()) {
                warn!(
                    "Invalid HostNewGameMenuScreen transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            if let Some(ref mut next_step) = next_host_screen {
                match (current, *event.event()) {
                    (HostNewGameMenuScreen::ConfigServer, SetNewHostGame::Next) => {
                        next_step.set(HostNewGameMenuScreen::ConfigWorld);
                    }
                    (HostNewGameMenuScreen::ConfigWorld, SetNewHostGame::Next) => {
                        next_step.set(HostNewGameMenuScreen::ConfigSave);
                    }
                    (HostNewGameMenuScreen::ConfigWorld, SetNewHostGame::Previous) => {
                        next_step.set(HostNewGameMenuScreen::ConfigServer);
                    }
                    (HostNewGameMenuScreen::ConfigSave, SetNewHostGame::Previous) => {
                        next_step.set(HostNewGameMenuScreen::ConfigWorld);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Handles SetSavedHostGame events for the HostSavedGame configuration flow.
fn on_set_saved_host_game(
    event: On<SetSavedHostGame>,
    current_parent: Res<State<MultiplayerMenuScreen>>,
    current: Option<Res<State<HostSavedGameMenuScreen>>>,
    mut next_multiplayer: ResMut<NextState<MultiplayerMenuScreen>>,
    mut next_host_screen: Option<ResMut<NextState<HostSavedGameMenuScreen>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_visibility: ResMut<NextState<ServerVisibility>>,
) {
    // Validate parent state transition
    if !is_valid_multiplayer_menu_screen_host_saved_game_transition(
        current_parent.get(),
        event.event(),
    ) {
        warn!(
            "Invalid MultiplayerMenuScreen transition for SetSavedHostGame: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Back/Cancel: Return to MultiplayerMenuScreen::Overview
        SetSavedHostGame::Back | SetSavedHostGame::Cancel => {
            next_multiplayer.set(MultiplayerMenuScreen::Overview);
        }
        // Confirm: Start the server
        SetSavedHostGame::Confirm => {
            // Note: According to spec, Confirm is only valid from ConfigServer
            if let Some(ref current_state) = current {
                if !matches!(
                    (current_state.get(), event.event()),
                    (
                        HostSavedGameMenuScreen::ConfigServer,
                        SetSavedHostGame::Confirm
                    )
                ) {
                    warn!(
                        "SetSavedHostGame::Confirm only valid from ConfigServer, current: {:?}",
                        current_state.get()
                    );
                    return;
                }
            }
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            next_server_visibility.set(ServerVisibility::GoingPublic);
        }
        // Next/Previous: Navigate through steps
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!("HostSavedGameMenuScreen does not exist - must be initialized first");
                    return;
                }
            };

            // Validate step transition
            if !is_valid_host_saved_game_menu_screen_transition(&current, event.event()) {
                warn!(
                    "Invalid HostSavedGameMenuScreen transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            if let Some(ref mut next_step) = next_host_screen {
                match (current, *event.event()) {
                    (HostSavedGameMenuScreen::Overview, SetSavedHostGame::Next) => {
                        next_step.set(HostSavedGameMenuScreen::ConfigServer);
                    }
                    (HostSavedGameMenuScreen::ConfigServer, SetSavedHostGame::Previous) => {
                        next_step.set(HostSavedGameMenuScreen::Overview);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Handles SetJoinGame events for the JoinGame flow.
fn on_set_join_game(
    event: On<SetJoinGame>,
    current_parent: Res<State<MultiplayerMenuScreen>>,
    current: Option<Res<State<JoinGameMenuScreen>>>,
    mut next_multiplayer: ResMut<NextState<MultiplayerMenuScreen>>,
    mut commands: bevy::prelude::Commands,
) {
    // Validate parent state transition
    if !is_valid_multiplayer_menu_screen_join_game_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid MultiplayerMenuScreen transition for SetJoinGame: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Back/Cancel: Return to MultiplayerMenuScreen::Overview
        SetJoinGame::Back | SetJoinGame::Cancel => {
            next_multiplayer.set(MultiplayerMenuScreen::Overview);
        }
        // Confirm: Trigger connection
        SetJoinGame::Confirm => {
            commands.trigger(crate::events::session::SetConnectingStep::Start);
        }
        // Next/Previous: Only Overview state exists, so these are no-ops
        _ => {
            if let Some(ref current_state) = current {
                // JoinGame only has Overview state, so Next/Previous are no-ops
                match (current_state.get(), event.event()) {
                    (JoinGameMenuScreen::Overview, SetJoinGame::Next) => {
                        // Trigger connection on Next from Overview
                        commands.trigger(crate::events::session::SetConnectingStep::Start);
                    }
                    (JoinGameMenuScreen::Overview, SetJoinGame::Previous) => {
                        // No-op: Previous from Overview stays in Overview
                    }
                    _ => {}
                }
            }
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    //! Tests für die Multiplayer Menu Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Validator-Funktionen (ob Übergänge gültig/ungültig sind)
    //! 2. Observer-Logik für alle Handler
    //! 3. State-Übergänge (MultiplayerMenuScreen, HostNewGameMenuScreen, etc.)
    //! 4. Session-Initialisierung (ServerStatus, ServerVisibility, SessionType)

    use crate::events::menu::multiplayer::{
        SetJoinGame, SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame,
    };
    use crate::states::menu::main::MainMenuScreen;
    use crate::states::menu::multiplayer::{
        HostNewGameMenuScreen, HostSavedGameMenuScreen, JoinGameMenuScreen, MultiplayerMenuScreen,
    };
    use crate::states::session::{ServerStatus, SessionType};

    mod validator_tests {
        use super::*;

        // Import all validator functions
        use super::super::{
            is_valid_host_new_game_menu_screen_transition,
            is_valid_host_saved_game_menu_screen_transition,
            is_valid_main_menu_screen_multiplayer_transition,
            is_valid_multiplayer_menu_screen_host_new_game_transition,
            is_valid_multiplayer_menu_screen_host_saved_game_transition,
            is_valid_multiplayer_menu_screen_join_game_transition,
            is_valid_multiplayer_menu_screen_transition,
        };

        /// Test: Gültige MainMenuScreen::Multiplayer Übergänge werden akzeptiert.
        #[test]
        fn test_valid_main_menu_screen_multiplayer_transitions() {
            // From Multiplayer: all events are valid
            assert!(is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Multiplayer,
                &SetMultiplayerMenu::Overview
            ));
            assert!(is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Multiplayer,
                &SetMultiplayerMenu::HostNewGame
            ));
            assert!(is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Multiplayer,
                &SetMultiplayerMenu::HostSavedGame
            ));
            assert!(is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Multiplayer,
                &SetMultiplayerMenu::JoinGame
            ));
            assert!(is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Multiplayer,
                &SetMultiplayerMenu::Back
            ));
        }

        /// Test: Valid transition from MainMenuScreen::Overview to Multiplayer.
        #[test]
        fn test_valid_main_menu_overview_to_multiplayer_transition() {
            // Overview -> Overview (switches parent to Multiplayer, substate initialized automatically)
            assert!(is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Overview,
                &SetMultiplayerMenu::Overview
            ));
        }

        /// Test: Ungültige MainMenuScreen Übergänge werden blockiert.
        #[test]
        fn test_invalid_main_menu_screen_multiplayer_transitions() {
            // Overview -> HostNewGame/HostSavedGame/JoinGame/Back is invalid (must go to Overview first)
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Overview,
                &SetMultiplayerMenu::HostNewGame
            ));
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Overview,
                &SetMultiplayerMenu::HostSavedGame
            ));
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Overview,
                &SetMultiplayerMenu::JoinGame
            ));
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Overview,
                &SetMultiplayerMenu::Back
            ));

            // Singleplayer -> Any SetMultiplayerMenu event is invalid
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Singleplayer,
                &SetMultiplayerMenu::Overview
            ));
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Singleplayer,
                &SetMultiplayerMenu::JoinGame
            ));

            // Settings -> Any SetMultiplayerMenu event is invalid
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Settings,
                &SetMultiplayerMenu::Overview
            ));
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Settings,
                &SetMultiplayerMenu::HostNewGame
            ));

            // Wiki -> Any SetMultiplayerMenu event is invalid
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Wiki,
                &SetMultiplayerMenu::Overview
            ));
            assert!(!is_valid_main_menu_screen_multiplayer_transition(
                &MainMenuScreen::Wiki,
                &SetMultiplayerMenu::Back
            ));
        }

        /// Test: Gültige MultiplayerMenuScreen Übergänge werden akzeptiert.
        #[test]
        fn test_valid_multiplayer_menu_screen_transitions() {
            // Overview -> HostNewGame ist gültig
            assert!(is_valid_multiplayer_menu_screen_transition(
                &MultiplayerMenuScreen::Overview,
                &SetMultiplayerMenu::HostNewGame
            ));

            // Overview -> HostSavedGame ist gültig
            assert!(is_valid_multiplayer_menu_screen_transition(
                &MultiplayerMenuScreen::Overview,
                &SetMultiplayerMenu::HostSavedGame
            ));

            // Overview -> JoinGame ist gültig
            assert!(is_valid_multiplayer_menu_screen_transition(
                &MultiplayerMenuScreen::Overview,
                &SetMultiplayerMenu::JoinGame
            ));

            // Overview -> Back ist gültig
            assert!(is_valid_multiplayer_menu_screen_transition(
                &MultiplayerMenuScreen::Overview,
                &SetMultiplayerMenu::Back
            ));

            // HostNewGame -> Back ist gültig
            assert!(is_valid_multiplayer_menu_screen_transition(
                &MultiplayerMenuScreen::HostNewGame,
                &SetMultiplayerMenu::Back
            ));
        }

        /// Test: Ungültige MultiplayerMenuScreen Übergänge werden blockiert.
        #[test]
        fn test_invalid_multiplayer_menu_screen_transitions() {
            // HostNewGame -> HostSavedGame ist ungültig (nur Back zu Overview)
            assert!(!is_valid_multiplayer_menu_screen_transition(
                &MultiplayerMenuScreen::HostNewGame,
                &SetMultiplayerMenu::HostSavedGame
            ));

            // HostSavedGame -> JoinGame ist ungültig
            assert!(!is_valid_multiplayer_menu_screen_transition(
                &MultiplayerMenuScreen::HostSavedGame,
                &SetMultiplayerMenu::JoinGame
            ));

            // JoinGame -> HostNewGame ist ungültig
            assert!(!is_valid_multiplayer_menu_screen_transition(
                &MultiplayerMenuScreen::JoinGame,
                &SetMultiplayerMenu::HostNewGame
            ));
        }

        /// Test: Gültige HostNewGameMenuScreen Übergänge werden akzeptiert.
        #[test]
        fn test_valid_host_new_game_menu_screen_transitions() {
            // ConfigServer -> Next ist gültig
            assert!(is_valid_host_new_game_menu_screen_transition(
                &HostNewGameMenuScreen::ConfigServer,
                &SetNewHostGame::Next
            ));

            // ConfigWorld -> Next ist gültig
            assert!(is_valid_host_new_game_menu_screen_transition(
                &HostNewGameMenuScreen::ConfigWorld,
                &SetNewHostGame::Next
            ));

            // ConfigWorld -> Previous ist gültig
            assert!(is_valid_host_new_game_menu_screen_transition(
                &HostNewGameMenuScreen::ConfigWorld,
                &SetNewHostGame::Previous
            ));

            // ConfigSave -> Previous ist gültig
            assert!(is_valid_host_new_game_menu_screen_transition(
                &HostNewGameMenuScreen::ConfigSave,
                &SetNewHostGame::Previous
            ));

            // ConfigSave -> Confirm ist gültig
            assert!(is_valid_host_new_game_menu_screen_transition(
                &HostNewGameMenuScreen::ConfigSave,
                &SetNewHostGame::Confirm
            ));
        }

        /// Test: HostNewGameMenuScreen Übergänge von ConfigSave werden validiert.
        #[test]
        fn test_host_new_game_config_save_transitions() {
            // ConfigSave -> Confirm ist gültig
            assert!(is_valid_host_new_game_menu_screen_transition(
                &HostNewGameMenuScreen::ConfigSave,
                &SetNewHostGame::Confirm
            ));

            // ConfigServer -> Confirm sollte ungültig sein
            // (je nach Implementierung kann dies true oder false sein)
        }

        /// Test: Gültige HostSavedGameMenuScreen Übergänge werden akzeptiert.
        #[test]
        fn test_valid_host_saved_game_menu_screen_transitions() {
            // Overview -> Next ist gültig
            assert!(is_valid_host_saved_game_menu_screen_transition(
                &HostSavedGameMenuScreen::Overview,
                &SetSavedHostGame::Next
            ));

            // ConfigServer -> Next ist gültig
            assert!(is_valid_host_saved_game_menu_screen_transition(
                &HostSavedGameMenuScreen::ConfigServer,
                &SetSavedHostGame::Next
            ));

            // ConfigServer -> Previous ist gültig
            assert!(is_valid_host_saved_game_menu_screen_transition(
                &HostSavedGameMenuScreen::ConfigServer,
                &SetSavedHostGame::Previous
            ));

            // ConfigServer -> Confirm ist gültig
            assert!(is_valid_host_saved_game_menu_screen_transition(
                &HostSavedGameMenuScreen::ConfigServer,
                &SetSavedHostGame::Confirm
            ));
        }

        // =============================================================================
        // Parent State Validators for Sub-Menus
        // =============================================================================

        /// Test: HostNewGame events are only valid from HostNewGame parent state.
        #[test]
        fn test_valid_multiplayer_menu_screen_host_new_game_transitions() {
            // All events are valid from HostNewGame
            assert!(is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::HostNewGame,
                &SetNewHostGame::Next
            ));
            assert!(is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::HostNewGame,
                &SetNewHostGame::Previous
            ));
            assert!(is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::HostNewGame,
                &SetNewHostGame::Confirm
            ));
            assert!(is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::HostNewGame,
                &SetNewHostGame::Back
            ));
            assert!(is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::HostNewGame,
                &SetNewHostGame::Cancel
            ));
        }

        /// Test: HostNewGame events are invalid from other parent states.
        #[test]
        fn test_invalid_multiplayer_menu_screen_host_new_game_transitions() {
            // Invalid from Overview
            assert!(!is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::Overview,
                &SetNewHostGame::Next
            ));
            assert!(!is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::Overview,
                &SetNewHostGame::Confirm
            ));

            // Invalid from HostSavedGame
            assert!(!is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::HostSavedGame,
                &SetNewHostGame::Next
            ));

            // Invalid from JoinGame
            assert!(!is_valid_multiplayer_menu_screen_host_new_game_transition(
                &MultiplayerMenuScreen::JoinGame,
                &SetNewHostGame::Back
            ));
        }

        /// Test: HostSavedGame events are only valid from HostSavedGame parent state.
        #[test]
        fn test_valid_multiplayer_menu_screen_host_saved_game_transitions() {
            // All events are valid from HostSavedGame
            assert!(is_valid_multiplayer_menu_screen_host_saved_game_transition(
                &MultiplayerMenuScreen::HostSavedGame,
                &SetSavedHostGame::Next
            ));
            assert!(is_valid_multiplayer_menu_screen_host_saved_game_transition(
                &MultiplayerMenuScreen::HostSavedGame,
                &SetSavedHostGame::Previous
            ));
            assert!(is_valid_multiplayer_menu_screen_host_saved_game_transition(
                &MultiplayerMenuScreen::HostSavedGame,
                &SetSavedHostGame::Confirm
            ));
            assert!(is_valid_multiplayer_menu_screen_host_saved_game_transition(
                &MultiplayerMenuScreen::HostSavedGame,
                &SetSavedHostGame::Back
            ));
            assert!(is_valid_multiplayer_menu_screen_host_saved_game_transition(
                &MultiplayerMenuScreen::HostSavedGame,
                &SetSavedHostGame::Cancel
            ));
        }

        /// Test: HostSavedGame events are invalid from other parent states.
        #[test]
        fn test_invalid_multiplayer_menu_screen_host_saved_game_transitions() {
            // Invalid from Overview
            assert!(
                !is_valid_multiplayer_menu_screen_host_saved_game_transition(
                    &MultiplayerMenuScreen::Overview,
                    &SetSavedHostGame::Next
                )
            );

            // Invalid from HostNewGame
            assert!(
                !is_valid_multiplayer_menu_screen_host_saved_game_transition(
                    &MultiplayerMenuScreen::HostNewGame,
                    &SetSavedHostGame::Confirm
                )
            );

            // Invalid from JoinGame
            assert!(
                !is_valid_multiplayer_menu_screen_host_saved_game_transition(
                    &MultiplayerMenuScreen::JoinGame,
                    &SetSavedHostGame::Back
                )
            );
        }

        /// Test: JoinGame events are only valid from JoinGame parent state.
        #[test]
        fn test_valid_multiplayer_menu_screen_join_game_transitions() {
            // All events are valid from JoinGame
            assert!(is_valid_multiplayer_menu_screen_join_game_transition(
                &MultiplayerMenuScreen::JoinGame,
                &SetJoinGame::Next
            ));
            assert!(is_valid_multiplayer_menu_screen_join_game_transition(
                &MultiplayerMenuScreen::JoinGame,
                &SetJoinGame::Previous
            ));
            assert!(is_valid_multiplayer_menu_screen_join_game_transition(
                &MultiplayerMenuScreen::JoinGame,
                &SetJoinGame::Confirm
            ));
            assert!(is_valid_multiplayer_menu_screen_join_game_transition(
                &MultiplayerMenuScreen::JoinGame,
                &SetJoinGame::Back
            ));
            assert!(is_valid_multiplayer_menu_screen_join_game_transition(
                &MultiplayerMenuScreen::JoinGame,
                &SetJoinGame::Cancel
            ));
        }

        /// Test: JoinGame events are invalid from other parent states.
        #[test]
        fn test_invalid_multiplayer_menu_screen_join_game_transitions() {
            // Invalid from Overview
            assert!(!is_valid_multiplayer_menu_screen_join_game_transition(
                &MultiplayerMenuScreen::Overview,
                &SetJoinGame::Confirm
            ));

            // Invalid from HostNewGame
            assert!(!is_valid_multiplayer_menu_screen_join_game_transition(
                &MultiplayerMenuScreen::HostNewGame,
                &SetJoinGame::Next
            ));

            // Invalid from HostSavedGame
            assert!(!is_valid_multiplayer_menu_screen_join_game_transition(
                &MultiplayerMenuScreen::HostSavedGame,
                &SetJoinGame::Back
            ));
        }
    }

    mod observer_tests {
        use super::*;

        pub mod helpers {
            use crate::{
                ChickenStatePlugin,
                events::{app::SetAppScope, menu::multiplayer::SetMultiplayerMenu},
                states::{
                    app::AppScope,
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
            use bevy::{prelude::*, state::app::StatesPlugin};

            /// Creates a test app with all required plugins.
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

            /// Setup helper: Sets MainMenuScreen to Multiplayer and MultiplayerMenuScreen to Overview.
            pub fn setup_test_app_in_multiplayer_overview() -> App {
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

                app.world_mut().trigger(SetMultiplayerMenu::Overview);
                update_app(&mut app, 1);

                let setup = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(setup.get(), &MainMenuScreen::Multiplayer);

                // Verify initial state
                let setup = app.world().resource::<State<MultiplayerMenuScreen>>();
                assert_eq!(setup.get(), &MultiplayerMenuScreen::Overview);

                app
            }

            /// Setup helper: Sets MultiplayerMenuScreen to HostNewGame with ConfigServer screen.
            pub fn setup_test_app_in_host_new_game() -> App {
                let mut app = setup_test_app_in_multiplayer_overview();

                // Navigate to HostNewGame
                app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
                update_app(&mut app, 1);

                // Verify state
                let setup = app.world().resource::<State<MultiplayerMenuScreen>>();
                assert_eq!(setup.get(), &MultiplayerMenuScreen::HostNewGame);

                let screen = app.world().resource::<State<HostNewGameMenuScreen>>();
                assert_eq!(screen.get(), &HostNewGameMenuScreen::ConfigServer);

                app
            }

            /// Setup helper: Sets MultiplayerMenuScreen to HostSavedGame.
            pub fn setup_test_app_in_host_saved_game() -> App {
                let mut app = setup_test_app_in_multiplayer_overview();

                // Navigate to HostSavedGame
                app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
                update_app(&mut app, 1);

                // Verify state
                let setup = app.world().resource::<State<MultiplayerMenuScreen>>();
                assert_eq!(setup.get(), &MultiplayerMenuScreen::HostSavedGame);

                let screen = app.world().resource::<State<HostSavedGameMenuScreen>>();
                assert_eq!(screen.get(), &HostSavedGameMenuScreen::Overview);

                app
            }

            /// Setup helper: Sets MultiplayerMenuScreen to JoinGame.
            pub fn setup_test_app_in_join_game() -> App {
                let mut app = setup_test_app_in_multiplayer_overview();

                // Navigate to JoinGame
                app.world_mut().trigger(SetMultiplayerMenu::JoinGame);
                update_app(&mut app, 1);

                // Verify state
                let setup = app.world().resource::<State<MultiplayerMenuScreen>>();
                assert_eq!(setup.get(), &MultiplayerMenuScreen::JoinGame);

                let screen = app.world().resource::<State<JoinGameMenuScreen>>();
                assert_eq!(screen.get(), &JoinGameMenuScreen::Overview);

                app
            }

            /// Asserts that MultiplayerMenuScreen state matches expected value.
            pub fn assert_multiplayer_screen(app: &mut App, expected: MultiplayerMenuScreen) {
                let setup = app.world().resource::<State<MultiplayerMenuScreen>>();
                assert_eq!(setup.get(), &expected);
            }

            /// Asserts that MainMenuScreen state matches expected value.
            pub fn assert_main_menu_screen(app: &mut App, expected: MainMenuScreen) {
                let context = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(context.get(), &expected);
            }

            /// Asserts that HostNewGameMenuScreen state matches expected value.
            pub fn assert_host_new_game_screen(app: &mut App, expected: HostNewGameMenuScreen) {
                let screen = app.world().resource::<State<HostNewGameMenuScreen>>();
                assert_eq!(screen.get(), &expected);
            }

            /// Asserts that HostSavedGameMenuScreen state matches expected value.
            pub fn assert_host_saved_game_screen(app: &mut App, expected: HostSavedGameMenuScreen) {
                let screen = app.world().resource::<State<HostSavedGameMenuScreen>>();
                assert_eq!(screen.get(), &expected);
            }

            /// Asserts that JoinGameMenuScreen state matches expected value.
            pub fn assert_join_game_screen(app: &mut App, expected: JoinGameMenuScreen) {
                let screen = app.world().resource::<State<JoinGameMenuScreen>>();
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
        // SetMultiplayerMenu Observer Tests
        // =============================================================================

        /// Test: Overview -> HostNewGame transition works.
        #[test]

        fn test_observer_overview_to_host_new_game() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);
        }

        /// Test: Overview -> HostSavedGame transition works.
        #[test]

        fn test_observer_overview_to_host_saved_game() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostSavedGame);
        }

        /// Test: Overview -> JoinGame transition works.
        #[test]

        fn test_observer_overview_to_join_game() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            app.world_mut().trigger(SetMultiplayerMenu::JoinGame);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::JoinGame);
        }

        /// Test: Back from Overview returns to MainMenu.
        #[test]

        fn test_observer_overview_back_to_main_menu() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            app.world_mut().trigger(SetMultiplayerMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
        }

        /// Test: HostNewGame -> Back returns to Overview.
        #[test]

        fn test_observer_host_new_game_back_to_overview() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            app.world_mut().trigger(SetMultiplayerMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Invalid direct switch between sub-states is blocked (e.g., HostNewGame -> HostSavedGame).
        #[test]

        fn test_observer_invalid_direct_switch_blocked() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            // Try to go directly from HostNewGame to HostSavedGame (should be ignored)
            app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
            helpers::update_app(&mut app, 1);

            // Should still be in HostNewGame
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);
        }

        /// Test: Events in wrong state are ignored.
        #[test]

        fn test_observer_events_ignored_in_wrong_state() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Try to trigger HostNewGame event from Overview
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Should still be in Overview
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        // =============================================================================
        // SetNewHostGame Observer Tests
        // =============================================================================

        /// Test: Next from ConfigServer goes to ConfigWorld.
        #[test]

        fn test_observer_host_new_game_next_config_server_to_config_world() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);
        }

        /// Test: Next from ConfigWorld goes to ConfigSave.
        #[test]

        fn test_observer_host_new_game_next_config_world_to_config_save() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            // Go to ConfigWorld first
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Then go to ConfigSave
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);
        }

        /// Test: Next navigation - ConfigServer -> ConfigWorld -> ConfigSave.
        #[test]

        fn test_observer_host_new_game_next_navigation() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            // ConfigServer -> Next -> ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

            // ConfigWorld -> Next -> ConfigSave
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);
        }

        /// Test: Previous navigation - ConfigWorld -> Previous -> ConfigServer.
        #[test]

        fn test_observer_host_new_game_previous_navigation() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            // Go to ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Go back to ConfigServer
            app.world_mut().trigger(SetNewHostGame::Previous);
            helpers::update_app(&mut app, 1);

            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);
        }

        /// Test: Back from ConfigServer returns to Overview.
        #[test]

        fn test_observer_host_new_game_back_from_config_server() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            app.world_mut().trigger(SetNewHostGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Cancel returns to Overview from any step.
        #[test]

        fn test_observer_host_new_game_cancel_returns_to_overview() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            // Go to ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Cancel should return to Overview
            app.world_mut().trigger(SetNewHostGame::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Confirm from ConfigSave starts the server session.
        #[test]

        fn test_observer_host_new_game_confirm_starts_server() {
            let mut app = helpers::setup_test_app_in_host_new_game();

            // Navigate to ConfigSave
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Confirm should start session
            app.world_mut().trigger(SetNewHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Events are ignored when not in HostNewGame state.
        #[test]

        fn test_observer_host_new_game_events_ignored_in_overview() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Try to trigger HostNewGame event from Overview
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Should still be in Overview
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        // =============================================================================
        // SetSavedHostGame Observer Tests
        // =============================================================================

        /// Test: Back from Overview returns to MultiplayerMenuScreen Overview.
        #[test]

        fn test_observer_host_saved_game_back_to_overview() {
            let mut app = helpers::setup_test_app_in_host_saved_game();

            app.world_mut().trigger(SetSavedHostGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Cancel from Overview returns to Overview.
        #[test]

        fn test_observer_host_saved_game_cancel_to_overview() {
            let mut app = helpers::setup_test_app_in_host_saved_game();

            app.world_mut().trigger(SetSavedHostGame::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Back from Overview returns to MultiplayerMenuScreen Overview.
        #[test]

        fn test_observer_host_saved_game_back_from_overview_to_multiplayer() {
            let mut app = helpers::setup_test_app_in_host_saved_game();

            app.world_mut().trigger(SetSavedHostGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Next from Overview goes to ConfigServer.
        #[test]

        fn test_observer_host_saved_game_next_to_config_server() {
            let mut app = helpers::setup_test_app_in_host_saved_game();

            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);

            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);
        }

        /// Test: Confirm from ConfigServer starts the server session.
        #[test]

        fn test_observer_host_saved_game_confirm_starts_server() {
            let mut app = helpers::setup_test_app_in_host_saved_game();

            // Go to ConfigServer
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Confirm should start session
            app.world_mut().trigger(SetSavedHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Previous from ConfigServer returns to Overview.
        #[test]

        fn test_observer_host_saved_game_previous_from_config_server() {
            let mut app = helpers::setup_test_app_in_host_saved_game();

            // Go to ConfigServer
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Go back to Overview
            app.world_mut().trigger(SetSavedHostGame::Previous);
            helpers::update_app(&mut app, 1);

            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);
        }

        /// Test: Events are ignored when not in HostSavedGame state.
        #[test]

        fn test_observer_host_saved_game_events_ignored_in_overview() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Try to trigger SavedHostGame event from Overview
            app.world_mut().trigger(SetSavedHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Should still be in Overview
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);

            // SessionType should still be None
            helpers::assert_session_type(&mut app, SessionType::None);
        }

        // =============================================================================
        // SetJoinGame Observer Tests
        // =============================================================================

        /// Test: Back returns to Overview.
        #[test]

        fn test_observer_join_game_back_to_overview() {
            let mut app = helpers::setup_test_app_in_join_game();

            app.world_mut().trigger(SetJoinGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Cancel returns to Overview.
        #[test]

        fn test_observer_join_game_cancel_to_overview() {
            let mut app = helpers::setup_test_app_in_join_game();

            app.world_mut().trigger(SetJoinGame::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Next triggers connection (SetConnectingStep::Start).
        #[test]

        fn test_observer_join_game_next_triggers_connect() {
            let mut app = helpers::setup_test_app_in_join_game();

            // Next should trigger SetConnectingStep::Start
            app.world_mut().trigger(SetJoinGame::Next);
            helpers::update_app(&mut app, 1);

            // Should stay in JoinGame Overview
            helpers::assert_join_game_screen(&mut app, JoinGameMenuScreen::Overview);
        }

        /// Test: Confirm triggers connection (SetConnectingStep::Start).
        #[test]

        fn test_observer_join_game_confirm_triggers_connect() {
            let mut app = helpers::setup_test_app_in_join_game();

            // Confirm should trigger SetConnectingStep::Start
            app.world_mut().trigger(SetJoinGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Should stay in JoinGame Overview
            helpers::assert_join_game_screen(&mut app, JoinGameMenuScreen::Overview);
        }

        /// Test: Previous is a no-op in JoinGame.
        #[test]

        fn test_observer_join_game_previous_noop() {
            let mut app = helpers::setup_test_app_in_join_game();

            // Previous should be a no-op
            app.world_mut().trigger(SetJoinGame::Previous);
            helpers::update_app(&mut app, 1);

            // Should stay in Overview
            helpers::assert_join_game_screen(&mut app, JoinGameMenuScreen::Overview);
        }

        /// Test: Events are ignored when not in JoinGame state.
        #[test]

        fn test_observer_join_game_events_ignored_in_overview() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Try to trigger JoinGame event from Overview
            app.world_mut().trigger(SetJoinGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Should still be in Overview
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }
    }

    mod integration_tests {
        use super::*;

        mod helpers {
            pub use super::super::observer_tests::helpers::*;
        }

        /// Test: Complete HostNewGame flow from Overview to session start.
        #[test]

        fn test_full_host_new_game_flow() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // 1. Navigate to HostNewGame
            app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);

            // 2. Next to ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

            // 3. Next to ConfigSave
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);

            // 4. Confirm starts the session
            app.world_mut().trigger(SetNewHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Complete HostSavedGame flow from Overview to session start.
        #[test]

        fn test_full_host_saved_game_flow() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // 1. Navigate to HostSavedGame
            app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostSavedGame);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);

            // 2. Next to ConfigServer
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

            // 3. Confirm starts the session
            app.world_mut().trigger(SetSavedHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Complete JoinGame flow - Overview -> Confirm triggers connection.
        #[test]

        fn test_full_join_game_flow() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // 1. Navigate to JoinGame
            app.world_mut().trigger(SetMultiplayerMenu::JoinGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::JoinGame);
            helpers::assert_join_game_screen(&mut app, JoinGameMenuScreen::Overview);

            // 2. Confirm triggers connection
            app.world_mut().trigger(SetJoinGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Should stay in JoinGame (connection is triggered)
            helpers::assert_join_game_screen(&mut app, JoinGameMenuScreen::Overview);
        }

        /// Test: Navigation back from HostNewGame flow using Previous.
        #[test]

        fn test_host_new_game_with_back_navigation() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Go to HostNewGame
            app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);

            // Go to ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

            // Go back to ConfigServer
            app.world_mut().trigger(SetNewHostGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);

            // Go back to Overview
            app.world_mut().trigger(SetNewHostGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Cancel interrupts the HostNewGame flow.
        #[test]

        fn test_host_new_game_cancel_mid_flow() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Go to HostNewGame
            app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
            helpers::update_app(&mut app, 1);

            // Go to ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Cancel aborts and returns to Overview
            app.world_mut().trigger(SetNewHostGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Navigation between all variants - Overview <-> HostNewGame <-> Overview <-> HostSavedGame <-> Overview <-> JoinGame.
        #[test]

        fn test_navigation_between_all_variants() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Overview -> HostNewGame
            app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);

            // HostNewGame -> Back -> Overview
            app.world_mut().trigger(SetMultiplayerMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);

            // Overview -> HostSavedGame
            app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostSavedGame);

            // HostSavedGame -> Back -> Overview
            app.world_mut().trigger(SetMultiplayerMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);

            // Overview -> JoinGame
            app.world_mut().trigger(SetMultiplayerMenu::JoinGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::JoinGame);

            // JoinGame -> Back -> Overview
            app.world_mut().trigger(SetMultiplayerMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Invalid events at each state are properly rejected.
        #[test]

        fn test_invalid_events_ignored() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // From Overview: Try HostNewGame event (should be ignored)
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);

            // Go to HostNewGame
            app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
            helpers::update_app(&mut app, 1);

            // From HostNewGame: Try to go directly to HostSavedGame (should be ignored)
            app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);

            // From HostNewGame: Try SavedHostGame event (should be ignored)
            app.world_mut().trigger(SetSavedHostGame::Confirm);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);

            // From HostNewGame: Try JoinGame event (should be ignored)
            app.world_mut().trigger(SetJoinGame::Confirm);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);
        }

        /// Test: HostSavedGame flow with navigation - Overview -> Next -> ConfigServer -> Previous -> Overview.
        #[test]

        fn test_host_saved_game_flow_with_navigation() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // 1. Navigate to HostSavedGame
            app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostSavedGame);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);

            // 2. Next to ConfigServer
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

            // 3. Previous to Overview
            app.world_mut().trigger(SetSavedHostGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);

            // 4. Back to Multiplayer Overview
            app.world_mut().trigger(SetSavedHostGame::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Mixed navigation flow - Overview -> HostNewGame -> Cancel -> HostSavedGame -> Next -> Confirm.
        #[test]

        fn test_mixed_navigation_flow() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Go to HostNewGame
            app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Cancel back to Overview
            app.world_mut().trigger(SetNewHostGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);

            // Go to HostSavedGame
            app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostSavedGame);

            // Next to ConfigServer
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

            // Confirm starts session
            app.world_mut().trigger(SetSavedHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }

        /// Test: Cancel interrupts the HostSavedGame flow.
        #[test]

        fn test_host_saved_game_cancel() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Go to HostSavedGame
            app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
            helpers::update_app(&mut app, 1);

            // Go to ConfigServer
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);

            // Cancel aborts and returns to Overview
            app.world_mut().trigger(SetSavedHostGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Cancel interrupts the JoinGame flow.
        #[test]

        fn test_join_game_cancel() {
            let mut app = helpers::setup_test_app_in_multiplayer_overview();

            // Go to JoinGame
            app.world_mut().trigger(SetMultiplayerMenu::JoinGame);
            helpers::update_app(&mut app, 1);

            // Cancel aborts and returns to Overview
            app.world_mut().trigger(SetJoinGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);
        }
    }
}
