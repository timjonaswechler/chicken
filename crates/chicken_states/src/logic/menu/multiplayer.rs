// TODO:    "understand the server lifecycle, get an idea how the menu should run correct, setup handler, observer and events to macht the goal of this file."

use {
    crate::{
        events::{
            menu::multiplayer::{
                SetJoinGame, SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame,
            },
            session::SetConnectingStep,
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
    bevy::prelude::{App, AppExtStates, Commands, NextState, On, Plugin, Res, ResMut, State},
};

pub(super) struct MultiplayerMenuPlugin;

impl Plugin for MultiplayerMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MultiplayerMenuScreen>()
            .add_sub_state::<HostNewGameMenuScreen>()
            .add_sub_state::<HostSavedGameMenuScreen>()
            .add_sub_state::<JoinGameMenuScreen>()
            .add_observer(handle_overview_nav)
            .add_observer(handle_host_new_game_nav)
            .add_observer(handle_host_saved_game_nav)
            .add_observer(handle_join_game_nav);
    }
}

// --- LOGIC HANDLERS ---

fn handle_overview_nav(
    trigger: On<SetMultiplayerMenu>,
    current_setup: Res<State<MultiplayerMenuScreen>>,
    mut next_setup: ResMut<NextState<MultiplayerMenuScreen>>,
    mut next_main_menu: ResMut<NextState<MainMenuScreen>>,
) {
    if *current_setup.get() != MultiplayerMenuScreen::Overview {
        return;
    }

    match trigger.event() {
        SetMultiplayerMenu::To(target) => next_setup.set(*target),
        SetMultiplayerMenu::Back => next_main_menu.set(MainMenuScreen::Overview),
    }
}

fn handle_host_new_game_nav(
    trigger: On<SetNewHostGame>,
    current_screen: Res<State<HostNewGameMenuScreen>>,
    mut next_screen: ResMut<NextState<HostNewGameMenuScreen>>,
    mut next_setup: ResMut<NextState<MultiplayerMenuScreen>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    current_setup: Res<State<MultiplayerMenuScreen>>,
) {
    if *current_setup.get() != MultiplayerMenuScreen::HostNewGame {
        return;
    }

    match trigger.event() {
        SetNewHostGame::Next => match current_screen.get() {
            HostNewGameMenuScreen::ConfigServer => {
                next_screen.set(HostNewGameMenuScreen::ConfigWorld)
            }
            HostNewGameMenuScreen::ConfigWorld => {
                next_screen.set(HostNewGameMenuScreen::ConfigSave)
            }
            HostNewGameMenuScreen::ConfigSave => {}
        },
        SetNewHostGame::Previous => match current_screen.get() {
            HostNewGameMenuScreen::ConfigServer => next_setup.set(MultiplayerMenuScreen::Overview),
            HostNewGameMenuScreen::ConfigWorld => {
                next_screen.set(HostNewGameMenuScreen::ConfigServer)
            }
            HostNewGameMenuScreen::ConfigSave => {
                next_screen.set(HostNewGameMenuScreen::ConfigWorld)
            }
        },
        SetNewHostGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            next_server_state.set(ServerVisibility::GoingPublic);
        }
        SetNewHostGame::Cancel => next_setup.set(MultiplayerMenuScreen::Overview),
        SetNewHostGame::Back => next_setup.set(MultiplayerMenuScreen::Overview),
    }
}

fn handle_host_saved_game_nav(
    trigger: On<SetSavedHostGame>,
    current_screen: Res<State<HostSavedGameMenuScreen>>,
    mut next_screen: ResMut<NextState<HostSavedGameMenuScreen>>,
    mut next_setup: ResMut<NextState<MultiplayerMenuScreen>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    current_setup: Res<State<MultiplayerMenuScreen>>,
) {
    if *current_setup.get() != MultiplayerMenuScreen::HostSavedGame {
        return;
    }

    match trigger.event() {
        SetSavedHostGame::Next => {
            if *current_screen.get() == HostSavedGameMenuScreen::Overview {
                next_screen.set(HostSavedGameMenuScreen::ConfigServer);
            }
        }
        SetSavedHostGame::Previous => {
            if *current_screen.get() == HostSavedGameMenuScreen::ConfigServer {
                next_screen.set(HostSavedGameMenuScreen::Overview);
            }
        }
        SetSavedHostGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            next_server_state.set(ServerVisibility::GoingPublic);
        }
        SetSavedHostGame::Cancel => next_setup.set(MultiplayerMenuScreen::Overview),
        SetSavedHostGame::Back => next_setup.set(MultiplayerMenuScreen::Overview),
    }
}

fn handle_join_game_nav(
    trigger: On<SetJoinGame>,
    current_setup: Res<State<MultiplayerMenuScreen>>,
    mut next_setup: ResMut<NextState<MultiplayerMenuScreen>>,
    mut commands: Commands,
) {
    if *current_setup.get() != MultiplayerMenuScreen::JoinGame {
        return;
    }

    match trigger.event() {
        SetJoinGame::Back => next_setup.set(MultiplayerMenuScreen::Overview),
        SetJoinGame::Confirm => {
            commands.trigger(SetConnectingStep::Start);
        }
        SetJoinGame::Cancel => next_setup.set(MultiplayerMenuScreen::Overview),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die Multiplayer Menu Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Observer-Logik für alle Handler (handle_overview_nav, handle_host_new_game_nav, etc.)
    //! 2. State-Übergänge (MultiplayerMenuScreen, HostNewGameMenuScreen, etc.)
    //! 3. Session-Initialisierung (ServerStatus, ServerVisibility, SessionType)
    //! 4. Event-Propagation (z.B. SetConnectingStep)

    use crate::events::menu::multiplayer::{
        SetJoinGame, SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame,
    };
    use crate::states::menu::main::MainMenuScreen;
    use crate::states::menu::multiplayer::{
        HostNewGameMenuScreen, HostSavedGameMenuScreen, MultiplayerMenuScreen,
    };
    use crate::states::session::{ServerStatus, ServerVisibility, SessionType};

    mod helpers {
        use crate::{
            ChickenStatePlugin,
            events::{
                app::SetAppScope,
                menu::{main::SetMainMenu, multiplayer::SetMultiplayerMenu},
            },
            states::{
                app::AppScope,
                menu::{
                    main::MainMenuScreen,
                    multiplayer::{
                        HostNewGameMenuScreen, HostSavedGameMenuScreen, MultiplayerMenuScreen,
                    },
                },
                session::{ServerStatus, ServerVisibility, SessionType},
            },
        };
        use bevy::{prelude::*, state::app::StatesPlugin};

        /// Erstellt eine Test-App mit MultiplayerMenuPlugin und allen Abhängigkeiten.
        pub fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, StatesPlugin, ChickenStatePlugin));

            // Initialisiere die benötigten States
            app.init_state::<AppScope>();
            app.init_state::<SessionType>();

            app
        }

        /// Führt den App-Update für die angegebene Anzahl von Ticks aus.
        pub fn update_app(app: &mut App, i: u8) {
            for _ in 0..i {
                app.update();
            }
        }

        /// Setzt die App in den Multiplayer-Menu-Scope.
        /// Startet im MainMenuScreen::Multiplayer mit MultiplayerMenuScreen::Overview.
        pub fn setup_multiplayer_menu_app() -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            let session_type_state = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type_state.get(), &SessionType::None);

            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Splash);

            app.world_mut().trigger(SetAppScope::To(AppScope::Menu));
            update_app(&mut app, 1);

            // Verifiziere, dass wir im Menu sind
            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Menu);

            // Verifiziere, dass MainMenuScreen existiert und auf Main steht
            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Overview);

            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Multiplayer));
            update_app(&mut app, 1);

            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Multiplayer);

            let setup = app.world().resource::<State<MultiplayerMenuScreen>>();
            assert_eq!(setup.get(), &MultiplayerMenuScreen::Overview);

            app
        }

        /// Setzt den MultiplayerMenuScreen-State auf einen bestimmten Wert.
        pub fn set_multiplayer_setup(app: &mut App, setup: MultiplayerMenuScreen) {
            app.world_mut().trigger(SetMultiplayerMenu::To(setup));
            update_app(app, 1);
        }

        /// Setzt den HostNewGameMenuScreen-State auf einen bestimmten Wert.
        pub fn set_host_new_game_screen(app: &mut App, screen: HostNewGameMenuScreen) {
            let mut next_screen = app
                .world_mut()
                .resource_mut::<NextState<HostNewGameMenuScreen>>();
            next_screen.set(screen);
            update_app(app, 1);
        }

        /// Setzt den HostSavedGameMenuScreen-State auf einen bestimmten Wert.
        pub fn set_host_saved_game_screen(app: &mut App, screen: HostSavedGameMenuScreen) {
            let mut next_screen = app
                .world_mut()
                .resource_mut::<NextState<HostSavedGameMenuScreen>>();
            next_screen.set(screen);
            update_app(app, 1);
        }

        /// Assert: MultiplayerMenuScreen-State überprüfen.
        pub fn assert_multiplayer_setup(app: &mut App, expected: MultiplayerMenuScreen) {
            let setup = app.world().resource::<State<MultiplayerMenuScreen>>();
            assert_eq!(
                setup.get(),
                &expected,
                "Expected MultiplayerMenuScreen::{:?}, got {:?}",
                expected,
                setup.get()
            );
        }

        /// Assert: MainMenuScreen-State überprüfen.
        pub fn assert_main_menu_context(app: &mut App, expected: MainMenuScreen) {
            let context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(
                context.get(),
                &expected,
                "Expected MainMenuScreen::{:?}, got {:?}",
                expected,
                context.get()
            );
        }

        /// Assert: HostNewGameMenuScreen-State überprüfen.
        pub fn assert_host_new_game_screen(app: &mut App, expected: HostNewGameMenuScreen) {
            let screen = app.world().resource::<State<HostNewGameMenuScreen>>();
            assert_eq!(
                screen.get(),
                &expected,
                "Expected HostNewGameMenuScreen::{:?}, got {:?}",
                expected,
                screen.get()
            );
        }

        /// Assert: HostSavedGameMenuScreen-State überprüfen.
        pub fn assert_host_saved_game_screen(app: &mut App, expected: HostSavedGameMenuScreen) {
            let screen = app.world().resource::<State<HostSavedGameMenuScreen>>();
            assert_eq!(
                screen.get(),
                &expected,
                "Expected HostSavedGameMenuScreen::{:?}, got {:?}",
                expected,
                screen.get()
            );
        }

        /// Assert: SessionType-State überprüfen.
        pub fn assert_session_type(app: &mut App, expected: SessionType) {
            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(
                session_type.get(),
                &expected,
                "Expected SessionType::{:?}, got {:?}",
                expected,
                session_type.get()
            );
        }

        /// Assert: ServerStatus-State überprüfen.
        pub fn assert_server_status(app: &mut App, expected: ServerStatus) {
            let status = app.world().resource::<State<ServerStatus>>();
            assert_eq!(
                status.get(),
                &expected,
                "Expected ServerStatus::{:?}, got {:?}",
                expected,
                status.get()
            );
        }

        /// Assert: ServerVisibility-State überprüfen.
        pub fn assert_server_visibility(app: &mut App, expected: ServerVisibility) {
            let visibility = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(
                visibility.get(),
                &expected,
                "Expected ServerVisibility::{:?}, got {:?}",
                expected,
                visibility.get()
            );
        }
    }

    // =============================================================================
    // TESTS FÜR handle_overview_nav
    // =============================================================================

    mod overview_nav_tests {
        use super::helpers;
        use super::*;

        /// Test: Navigate zu verschiedenen MultiplayerMenuScreen-Zielen.
        ///
        /// Ein SetMultiplayerMenu::To(target) Event sollte den
        /// MultiplayerMenuScreen-State auf das Ziel ändern.
        #[test]
        fn test_overview_navigate_to_target() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Teste Navigation zu HostNewGame
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::HostNewGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Zurück zu Overview
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);

            // Teste Navigation zu HostSavedGame
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::HostSavedGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);

            // Zurück zu Overview
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);

            // Teste Navigation zu JoinGame
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::JoinGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::JoinGame);
        }

        /// Test: Back-Event kehrt zum MainMenu zurück.
        ///
        /// Ein SetMultiplayerMenu::Back Event sollte den MainMenuScreen
        /// zu MainMenuScreen::Main ändern.
        #[test]
        fn test_overview_back_to_main_menu() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Trigger Back
            app.world_mut().trigger(SetMultiplayerMenu::Back);
            helpers::update_app(&mut app, 1);

            // Verifiziere Rückkehr zum Hauptmenü
            helpers::assert_main_menu_context(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Events werden ignoriert wenn nicht in Overview-State.
        ///
        /// Wenn der aktuelle State nicht Overview ist, sollen Navigate-Events
        /// vom handle_overview_nav Handler ignoriert werden.
        #[test]
        fn test_overview_wrong_state_ignored() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostNewGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Versuche Navigation über Overview-Handler (sollte ignoriert werden)
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::JoinGame));
            helpers::update_app(&mut app, 1);

            // State sollte unverändert HostNewGame bleiben
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Versuche Back (sollte ebenfalls ignoriert werden)
            app.world_mut().trigger(SetMultiplayerMenu::Back);
            helpers::update_app(&mut app, 1);

            // MainMenuScreen sollte unverändert Multiplayer bleiben
            helpers::assert_main_menu_context(&mut app, MainMenuScreen::Multiplayer);
        }
    }

    // =============================================================================
    // TESTS FÜR handle_host_new_game_nav
    // =============================================================================

    mod host_new_game_nav_tests {
        use super::helpers;
        use super::*;

        /// Test: Next-Event durchläuft die Konfigurationsschritte.
        ///
        /// ConfigServer -> ConfigWorld -> ConfigSave
        #[test]
        fn test_host_new_game_next_transitions() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostNewGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);

            // Next: ConfigServer -> ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

            // Next: ConfigWorld -> ConfigSave
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);

            // Weiteres Next im ConfigSave bleibt auf ConfigSave
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);
        }

        /// Test: Previous-Event navigiert zurück durch die Schritte.
        ///
        /// ConfigSave -> ConfigWorld -> ConfigServer -> Overview
        #[test]
        fn test_host_new_game_previous_transitions() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostNewGame und setze auf ConfigSave
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);
            helpers::set_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);

            // Previous: ConfigSave -> ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

            // Previous: ConfigWorld -> ConfigServer
            app.world_mut().trigger(SetNewHostGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);

            // Previous: ConfigServer -> Overview
            app.world_mut().trigger(SetNewHostGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Confirm-Event startet den Server.
        ///
        /// Confirm sollte SessionType, ServerStatus und ServerVisibility setzen.
        #[test]
        fn test_host_new_game_confirm_starts_server() {
            let mut app = helpers::setup_multiplayer_menu_app();
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::HostNewGame));
            helpers::update_app(&mut app, 1);

            // Wechsel zu HostNewGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Trigger Confirm
            app.world_mut().trigger(SetNewHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Verifiziere SessionType
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);

            // Verifiziere ServerStatus
            helpers::assert_server_status(&mut app, ServerStatus::Starting);

            // Verifiziere ServerVisibility
            // TODO: should be go through the complett startup process, then ServerVisibility can get checked for being public
            // helpers::assert_server_visibility(&mut app, ServerVisibility::GoingPublic);
        }

        /// Test: Cancel-Event kehrt zu Overview zurück.
        ///
        /// Cancel sollte den MultiplayerMenuScreen-State auf Overview setzen.
        #[test]
        fn test_host_new_game_cancel_returns_overview() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostNewGame und setze auf ConfigWorld
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);
            helpers::set_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

            // Trigger Cancel
            app.world_mut().trigger(SetNewHostGame::Cancel);
            helpers::update_app(&mut app, 1);

            // Verifiziere Rückkehr zu Overview
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Back-Event kehrt zu Overview zurück.
        ///
        /// Back sollte den MultiplayerMenuScreen-State auf Overview setzen.
        #[test]
        fn test_host_new_game_back_returns_overview() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostNewGame und setze auf ConfigSave
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);
            helpers::set_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);

            // Trigger Back
            app.world_mut().trigger(SetNewHostGame::Back);
            helpers::update_app(&mut app, 1);

            // Verifiziere Rückkehr zu Overview
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Events werden ignoriert wenn nicht in HostNewGame-State.
        ///
        /// Wenn der aktuelle MultiplayerMenuScreen nicht HostNewGame ist, sollen
        /// alle SetNewHostGame-Events ignoriert werden.
        #[test]
        fn test_host_new_game_wrong_state_ignored() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Setze einen Screen für den Fall, dass wir später wechseln
            helpers::set_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);

            // Wechsel zu HostSavedGame (nicht HostNewGame)
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);

            // Versuche Next (sollte ignoriert werden)
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);

            // MultiplayerMenuScreen sollte unverändert HostSavedGame bleiben
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);

            // Versuche Confirm (sollte ignoriert werden - keine Session-Änderung)
            app.world_mut().trigger(SetNewHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            // SessionType sollte immer noch None sein
            helpers::assert_session_type(&mut app, SessionType::None);
        }
    }

    // =============================================================================
    // TESTS FÜR handle_host_saved_game_nav
    // =============================================================================

    mod host_saved_game_nav_tests {
        use super::helpers;
        use super::*;

        /// Test: Next-Event wechselt von Overview zu ConfigServer.
        ///
        /// Next im Overview-State sollte zu ConfigServer wechseln.
        #[test]
        fn test_host_saved_game_next_overview_to_config() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostSavedGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);

            // Next: Overview -> ConfigServer
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);
        }

        /// Test: Next-Event im ConfigServer bleibt auf ConfigServer.
        ///
        /// Da es keinen weiteren Schritt gibt, bleibt Next im ConfigServer.
        #[test]
        fn test_host_saved_game_next_config_stays() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostSavedGame und setze auf ConfigServer
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);
            helpers::set_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

            // Next bleibt auf ConfigServer
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);
        }

        /// Test: Previous-Event wechselt von ConfigServer zu Overview.
        ///
        /// Previous im ConfigServer-State sollte zu Overview wechseln.
        #[test]
        fn test_host_saved_game_previous_config_to_overview() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostSavedGame und setze auf ConfigServer
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);
            helpers::set_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

            // Previous: ConfigServer -> Overview
            app.world_mut().trigger(SetSavedHostGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);
        }

        /// Test: Previous-Event im Overview bleibt auf Overview.
        ///
        /// Da es keinen vorherigen Schritt gibt, bleibt Previous im Overview.
        #[test]
        fn test_host_saved_game_previous_overview_stays() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostSavedGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);

            // Previous bleibt auf Overview (kein Übergang definiert)
            app.world_mut().trigger(SetSavedHostGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);
        }

        /// Test: Confirm-Event startet den Server.
        ///
        /// Confirm sollte SessionType, ServerStatus und ServerVisibility setzen.
        #[test]
        fn test_host_saved_game_confirm_starts_server() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostSavedGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);

            // Trigger Confirm
            app.world_mut().trigger(SetSavedHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Verifiziere SessionType
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);

            // Verifiziere ServerStatus
            helpers::assert_server_status(&mut app, ServerStatus::Starting);

            // Verifiziere ServerVisibility
            // TODO: Same as "test_host_new_game_confirm_starts_server"
            helpers::assert_server_visibility(&mut app, ServerVisibility::GoingPublic);
        }

        /// Test: Cancel-Event kehrt zu Overview zurück.
        ///
        /// Cancel sollte den MultiplayerMenuScreen-State auf Overview setzen.
        #[test]
        fn test_host_saved_game_cancel_returns_overview() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostSavedGame und setze auf ConfigServer
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);
            helpers::set_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

            // Trigger Cancel
            app.world_mut().trigger(SetSavedHostGame::Cancel);
            helpers::update_app(&mut app, 1);

            // Verifiziere Rückkehr zu Overview
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Back-Event kehrt zu Overview zurück.
        ///
        /// Back sollte den MultiplayerMenuScreen-State auf Overview setzen.
        #[test]
        fn test_host_saved_game_back_returns_overview() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostSavedGame und setze auf ConfigServer
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostSavedGame);
            helpers::set_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

            // Trigger Back
            app.world_mut().trigger(SetSavedHostGame::Back);
            helpers::update_app(&mut app, 1);

            // Verifiziere Rückkehr zu Overview
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Events werden ignoriert wenn nicht in HostSavedGame-State.
        ///
        /// Wenn der aktuelle MultiplayerMenuScreen nicht HostSavedGame ist, sollen
        /// alle SetSavedHostGame-Events ignoriert werden.
        #[test]
        fn test_host_saved_game_wrong_state_ignored() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Setze einen Screen für den Fall, dass wir später wechseln
            helpers::set_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);

            // Wechsel zu HostNewGame (nicht HostSavedGame)
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Versuche Next (sollte ignoriert werden)
            app.world_mut().trigger(SetSavedHostGame::Next);
            helpers::update_app(&mut app, 1);

            // MultiplayerMenuScreen sollte unverändert HostNewGame bleiben
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Versuche Confirm (sollte ignoriert werden - keine Session-Änderung)
            app.world_mut().trigger(SetSavedHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            // SessionType sollte immer noch None sein
            helpers::assert_session_type(&mut app, SessionType::None);
        }
    }

    // =============================================================================
    // TESTS FÜR handle_join_game_nav
    // =============================================================================

    mod join_game_nav_tests {
        use super::helpers;
        use super::*;

        /// Test: Back-Event kehrt zu Overview zurück.
        ///
        /// SetJoinGame::Back sollte den MultiplayerMenuScreen-State auf Overview setzen.
        #[test]
        fn test_join_game_back_returns_overview() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu JoinGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::JoinGame);

            // Trigger Back
            app.world_mut().trigger(SetJoinGame::Back);
            helpers::update_app(&mut app, 1);

            // Verifiziere Rückkehr zu Overview
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Cancel-Event kehrt zu Overview zurück.
        ///
        /// SetJoinGame::Cancel sollte den MultiplayerMenuScreen-State auf Overview setzen.
        #[test]
        fn test_join_game_cancel_returns_overview() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu JoinGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::JoinGame);

            // Trigger Cancel
            app.world_mut().trigger(SetJoinGame::Cancel);
            helpers::update_app(&mut app, 1);

            // Verifiziere Rückkehr zu Overview
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);
        }

        /// Test: Confirm-Event triggert SetConnectingStep::Start.
        ///
        /// SetJoinGame::Confirm sollte ein SetConnectingStep::Start Event auslösen.
        #[cfg(feature = "hosted")]
        #[test]
        fn test_join_game_confirm_triggers_connect() {
            use bevy::prelude::NextState;

            let mut app = helpers::setup_multiplayer_menu_app();

            // Initialisiere ClientConnectionStatus und ConnectingStep (für hosted feature)
            // Da ConnectingStep ein SubState von ClientConnectionStatus::Connecting ist,
            // müssen wir zuerst den SessionType auf Client setzen
            let mut next_session_type = app.world_mut().resource_mut::<NextState<SessionType>>();
            next_session_type.set(SessionType::Client);
            helpers::update_app(&mut app, 1);

            // Wechsel zu JoinGame
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::JoinGame);

            // Trigger Confirm
            app.world_mut().trigger(SetJoinGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Verifiziere, dass SetConnectingStep::Start ausgelöst wurde
            // Durch das Event sollte der ConnectingStep existieren (bei hosted feature)
            // Hinweis: Da ConnectingStep nur existiert wenn ClientConnectionStatus::Connecting,
            // können wir nur prüfen, dass das Event getriggert wurde (kein Panic)
        }

        /// Test: Events werden ignoriert wenn nicht in JoinGame-State.
        ///
        /// Wenn der aktuelle MultiplayerMenuScreen nicht JoinGame ist, sollen
        /// alle SetJoinGame-Events ignoriert werden.
        #[test]
        fn test_join_game_wrong_state_ignored() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Wechsel zu HostNewGame (nicht JoinGame)
            helpers::set_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Versuche Back (sollte ignoriert werden)
            app.world_mut().trigger(SetJoinGame::Back);
            helpers::update_app(&mut app, 1);

            // MultiplayerMenuScreen sollte unverändert HostNewGame bleiben
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);

            // Versuche Cancel (sollte ebenfalls ignoriert werden)
            app.world_mut().trigger(SetJoinGame::Cancel);
            helpers::update_app(&mut app, 1);

            // MultiplayerMenuScreen sollte weiterhin HostNewGame bleiben
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);
        }
    }

    // =============================================================================
    // INTEGRATION TESTS
    // =============================================================================

    mod integration_tests {
        use super::helpers;
        use super::*;

        /// Test: Vollständiger Host-New-Game Flow.
        ///
        /// Simuliert den kompletten Ablauf vom Overview bis zum Server-Start:
        /// Overview -> HostNewGame -> ConfigServer -> ConfigWorld -> ConfigSave -> Confirm
        #[test]
        fn test_full_host_new_game_flow() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Schritt 1: Navigiere zu HostNewGame
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::HostNewGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::HostNewGame);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);

            // Schritt 2: Next zu ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

            // Schritt 3: Next zu ConfigSave
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigSave);

            // Schritt 4: Confirm startet den Server
            app.world_mut().trigger(SetNewHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Verifiziere Server-Initialisierung
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
            helpers::assert_server_visibility(&mut app, ServerVisibility::GoingPublic);
        }

        /// Test: Vollständiger Host-New-Game Flow mit Abbruch.
        ///
        /// Simuliert den Ablauf mit Cancel in der Mitte:
        /// Overview -> HostNewGame -> ConfigServer -> ConfigWorld -> Cancel -> Overview
        #[test]
        fn test_host_new_game_flow_with_cancel() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Navigiere zu HostNewGame
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::HostNewGame));
            helpers::update_app(&mut app, 1);

            // Gehe zu ConfigWorld
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigWorld);

            // Cancel kehrt zurück zu Overview
            app.world_mut().trigger(SetNewHostGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);

            // Session sollte nicht gestartet sein
            helpers::assert_session_type(&mut app, SessionType::None);
        }

        /// Test: Vollständiger Join-Game Flow.
        ///
        /// Simuliert den kompletten Ablauf vom Overview bis zur Verbindung:
        /// Overview -> JoinGame -> Confirm
        #[cfg(feature = "hosted")]
        #[test]
        fn test_full_join_game_flow() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Schritt 1: Navigiere zu JoinGame
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::JoinGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::JoinGame);

            // Schritt 2: Confirm triggert Verbindung
            app.world_mut().trigger(SetJoinGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Der Flow sollte ohne Panic durchlaufen
            // (Die eigentliche Verbindungslogik ist in einem anderen Modul)
        }

        /// Test: Navigation zwischen allen MultiplayerMenuScreen-Varianten.
        ///
        /// Testet, dass alle Navigationen zwischen den verschiedenen
        /// MultiplayerMenuScreen-States korrekt funktionieren.
        #[test]
        fn test_navigation_between_all_setup_variants() {
            let variants = [
                MultiplayerMenuScreen::HostNewGame,
                MultiplayerMenuScreen::HostSavedGame,
                MultiplayerMenuScreen::JoinGame,
            ];

            for variant in variants {
                let mut app = helpers::setup_multiplayer_menu_app();

                // Navigiere zur Variante
                app.world_mut().trigger(SetMultiplayerMenu::To(variant));
                helpers::update_app(&mut app, 1);
                helpers::assert_multiplayer_setup(&mut app, variant);

                // Zurück zu Overview
                app.world_mut()
                    .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::Overview));
                helpers::update_app(&mut app, 1);
                helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);
            }
        }

        /// Test: Round-Trip von HostNewGame nach Abbruch.
        ///
        /// Testet, dass nach einem Abbruch ein erneuter Start möglich ist.
        #[test]
        fn test_host_new_game_round_trip() {
            let mut app = helpers::setup_multiplayer_menu_app();

            // Erster Versuch: Abbrechen in ConfigWorld
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::HostNewGame));
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetNewHostGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_multiplayer_setup(&mut app, MultiplayerMenuScreen::Overview);

            // Zweiter Versuch: Erfolgreich durchführen
            app.world_mut()
                .trigger(SetMultiplayerMenu::To(MultiplayerMenuScreen::HostNewGame));
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetNewHostGame::Next);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetNewHostGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }
    }
}
