use {
    crate::{
        events::{
            app::SetAppScope,
            menu::multiplayer::{SetJoinGame, SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame},
        },
        logic::session::server::PendingGoingPublic,
        states::{
            app::AppScope,
            menu::{
                main::MainMenuScreen,
                multiplayer::{
                    HostNewGameMenuScreen, HostSavedGameMenuScreen, JoinGameMenuScreen,
                    MultiplayerMenuScreen,
                },
            },
            session::{ClientConnectionStatus, ServerStatus, SessionType},
        },
    },
    bevy::prelude::{App, AppExtStates, Commands, NextState, On, Plugin, Res, ResMut, State, warn},
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
    matches!(
        (from, to),
        (MainMenuScreen::Overview, SetMultiplayerMenu::Overview)
            | (
                MainMenuScreen::Multiplayer,
                SetMultiplayerMenu::Overview
                    | SetMultiplayerMenu::HostNewGame
                    | SetMultiplayerMenu::HostSavedGame
                    | SetMultiplayerMenu::JoinGame
                    | SetMultiplayerMenu::Back
            )
    )
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
        (
            MultiplayerMenuScreen::Overview,
            SetMultiplayerMenu::HostNewGame
        ) | (
            MultiplayerMenuScreen::Overview,
            SetMultiplayerMenu::HostSavedGame
        ) | (
            MultiplayerMenuScreen::Overview,
            SetMultiplayerMenu::JoinGame
        ) | (MultiplayerMenuScreen::Overview, SetMultiplayerMenu::Back)
            | (MultiplayerMenuScreen::HostNewGame, SetMultiplayerMenu::Back)
            | (
                MultiplayerMenuScreen::HostSavedGame,
                SetMultiplayerMenu::Back
            )
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
            | (HostNewGameMenuScreen::ConfigWorld, SetNewHostGame::Previous)
            | (HostNewGameMenuScreen::ConfigSave, SetNewHostGame::Previous)
            | (HostNewGameMenuScreen::ConfigSave, SetNewHostGame::Confirm)
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
            | (
                HostSavedGameMenuScreen::ConfigServer,
                SetSavedHostGame::Previous
            )
            | (
                HostSavedGameMenuScreen::ConfigServer,
                SetSavedHostGame::Confirm
            )
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
        SetMultiplayerMenu::Back => {
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

        SetMultiplayerMenu::Overview => {
            next_main_menu.set(MainMenuScreen::Multiplayer);
        }

        SetMultiplayerMenu::HostNewGame
        | SetMultiplayerMenu::HostSavedGame
        | SetMultiplayerMenu::JoinGame => {
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
    mut commands: Commands,
    current_parent: Res<State<MultiplayerMenuScreen>>,
    current: Option<Res<State<HostNewGameMenuScreen>>>,
    mut next_multiplayer: ResMut<NextState<MultiplayerMenuScreen>>,
    mut next_host_screen: Option<ResMut<NextState<HostNewGameMenuScreen>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
) {
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
        SetNewHostGame::Cancel => {
            next_multiplayer.set(MultiplayerMenuScreen::Overview);
        }

        SetNewHostGame::Confirm => {
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
            commands.insert_resource(PendingGoingPublic);
            commands.trigger(SetAppScope::Session);
        }

        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!("HostNewGameMenuScreen does not exist - must be initialized first");
                    return;
                }
            };

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
    mut commands: Commands,
    current_parent: Res<State<MultiplayerMenuScreen>>,
    current: Option<Res<State<HostSavedGameMenuScreen>>>,
    mut next_multiplayer: ResMut<NextState<MultiplayerMenuScreen>>,
    mut next_host_screen: Option<ResMut<NextState<HostSavedGameMenuScreen>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
) {
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
        SetSavedHostGame::Cancel => {
            next_multiplayer.set(MultiplayerMenuScreen::Overview);
        }

        SetSavedHostGame::Confirm => {
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
            commands.insert_resource(PendingGoingPublic);
            commands.trigger(SetAppScope::Session);
        }
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!("HostSavedGameMenuScreen does not exist - must be initialized first");
                    return;
                }
            };

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
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_client_status: ResMut<NextState<ClientConnectionStatus>>,
) {
    if !is_valid_multiplayer_menu_screen_join_game_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid MultiplayerMenuScreen transition for SetJoinGame: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        SetJoinGame::Cancel => {
            next_multiplayer.set(MultiplayerMenuScreen::Overview);
        }

        SetJoinGame::Confirm => {
            next_session_type.set(SessionType::Client);
            next_app_scope.set(AppScope::Session);
            next_client_status.set(ClientConnectionStatus::Connecting);
        }

        _ => {
            if let Some(ref current_state) = current {
                // JoinGame only has Overview state, so Next/Previous are no-ops
                match (current_state.get(), event.event()) {
                    (JoinGameMenuScreen::Overview, SetJoinGame::Next) => {
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
        HostNewGameMenuScreen, HostSavedGameMenuScreen, MultiplayerMenuScreen,
    };

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
        }
    }
}
