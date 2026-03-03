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
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State, info},
};

pub(super) struct SingleplayerMenuPlugin;

impl Plugin for SingleplayerMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SingleplayerMenuScreen>()
            .add_sub_state::<NewGameMenuScreen>()
            .add_sub_state::<SavedGameMenuScreen>()
            .add_observer(handle_overview_nav)
            .add_observer(handle_new_game_nav)
            .add_observer(handle_load_game_nav);
    }
}

// --- LOGIC HANDLERS ---

fn handle_overview_nav(
    trigger: On<SetSingleplayerMenu>,
    mut next_setup: ResMut<NextState<SingleplayerMenuScreen>>,
    mut next_main_menu: ResMut<NextState<MainMenuScreen>>,
    current_setup: Res<State<SingleplayerMenuScreen>>,
) {
    if *current_setup.get() != SingleplayerMenuScreen::Overview {
        return;
    }

    match trigger.event() {
        SetSingleplayerMenu::To(target) => next_setup.set(*target),
        SetSingleplayerMenu::Back => next_main_menu.set(MainMenuScreen::Overview),
    }
}

fn handle_new_game_nav(
    trigger: On<SetSingleplayerNewGame>,
    current_screen: Option<Res<State<NewGameMenuScreen>>>,
    mut next_screen: ResMut<NextState<NewGameMenuScreen>>,
    mut next_setup: ResMut<NextState<SingleplayerMenuScreen>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    current_setup: Res<State<SingleplayerMenuScreen>>,
) {
    if *current_setup.get() != SingleplayerMenuScreen::NewGame {
        return;
    }

    match trigger.event() {
        SetSingleplayerNewGame::Next => {
            if let Some(screen) = current_screen {
                match *screen.get() {
                    NewGameMenuScreen::ConfigPlayer => {
                        next_screen.set(NewGameMenuScreen::ConfigWorld)
                    }
                    NewGameMenuScreen::ConfigWorld => {
                        next_screen.set(NewGameMenuScreen::ConfigSave)
                    }
                    NewGameMenuScreen::ConfigSave => {}
                }
            }
        }
        SetSingleplayerNewGame::Previous => {
            if let Some(screen) = current_screen {
                match *screen.get() {
                    NewGameMenuScreen::ConfigPlayer => {
                        next_setup.set(SingleplayerMenuScreen::Overview)
                    }
                    NewGameMenuScreen::ConfigWorld => {
                        next_screen.set(NewGameMenuScreen::ConfigPlayer)
                    }
                    NewGameMenuScreen::ConfigSave => {
                        next_screen.set(NewGameMenuScreen::ConfigWorld)
                    }
                }
            }
        }
        SetSingleplayerNewGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            next_server_state.set(ServerVisibility::Private);
        }
        SetSingleplayerNewGame::Cancel => next_setup.set(SingleplayerMenuScreen::Overview),
        SetSingleplayerNewGame::Back => {
            next_setup.set(SingleplayerMenuScreen::Overview);
            info!("Back button clicked");
        }
    }
}

fn handle_load_game_nav(
    trigger: On<SetSingleplayerSavedGame>,
    current_screen: Option<Res<State<SavedGameMenuScreen>>>,
    current_setup: Res<State<SingleplayerMenuScreen>>,
    mut next_setup: ResMut<NextState<SingleplayerMenuScreen>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
) {
    if *current_setup.get() != SingleplayerMenuScreen::LoadGame {
        return;
    }

    match trigger.event() {
        SetSingleplayerSavedGame::Previous => {
            if let Some(screen) = current_screen {
                if *screen.get() == SavedGameMenuScreen::SelectSaveGame {
                    next_setup.set(SingleplayerMenuScreen::Overview);
                }
            }
        }
        SetSingleplayerSavedGame::Confirm => {
            next_session_type.set(SessionType::Singleplayer);
            next_server_status.set(ServerStatus::Starting);
            next_server_state.set(ServerVisibility::Private);
        }
        SetSingleplayerSavedGame::Cancel => next_setup.set(SingleplayerMenuScreen::Overview),
        SetSingleplayerSavedGame::Back => next_setup.set(SingleplayerMenuScreen::Overview),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die Singleplayer-Menü Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Overview-Navigation (Navigation zu NewGame/LoadGame, Zurück zum Hauptmenü)
    //! 2. NewGame-Navigation (Next/Previous durch Screens, Confirm, Cancel, Back)
    //! 3. LoadGame-Navigation (Previous, Confirm, Cancel, Back)
    //! 4. Vollständige Flows (Integrationstests)

    use crate::events::menu::singleplayer::{
        SetSingleplayerMenu, SetSingleplayerNewGame, SetSingleplayerSavedGame,
    };
    use crate::states::menu::main::MainMenuScreen;
    use crate::states::menu::singleplayer::{
        NewGameMenuScreen, SavedGameMenuScreen, SingleplayerMenuScreen,
    };
    use crate::states::session::{ServerStatus, ServerVisibility, SessionType};

    mod helpers {
        use crate::{
            events::menu::singleplayer::SetSingleplayerMenu,
            logic::menu::singleplayer::SingleplayerMenuPlugin,
            states::menu::main::MainMenuScreen,
            states::menu::singleplayer::{
                NewGameMenuScreen, SavedGameMenuScreen, SingleplayerMenuScreen,
            },
            states::session::{ServerStatus, ServerVisibility, SessionType},
        };
        use bevy::{prelude::*, state::app::StatesPlugin};

        /// Erstellt eine Test-App mit allen benötigten Plugins für Singleplayer-Menu Tests.
        pub fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, StatesPlugin, SingleplayerMenuPlugin));

            // Initialisiere die Parent States, die für SubStates benötigt werden
            app.init_state::<MainMenuScreen>();

            // Initialisiere die Session-States, die von den Handlern benötigt werden
            // Diese werden normalerweise vom ServerSessionPlugin initialisiert
            app.init_state::<SessionType>();
            app.add_sub_state::<ServerStatus>();
            app.add_sub_state::<ServerVisibility>();

            app
        }

        /// Führt die App für eine bestimmte Anzahl von Update-Ticks aus.
        pub fn update_app(app: &mut App, i: u8) {
            for _ in 0..i {
                app.update();
            }
        }

        /// Setup für Tests: Setzt den MainMenuScreen auf Singleplayer und den SingleplayerMenuScreen auf Overview.
        pub fn setup_test_app_in_overview() -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            // Setze MainMenuScreen auf Singleplayer (Parent von SingleplayerMenuScreen)
            let mut next_main_menu = app.world_mut().resource_mut::<NextState<MainMenuScreen>>();
            next_main_menu.set(MainMenuScreen::Singleplayer);
            update_app(&mut app, 1);

            // Setze SingleplayerMenuScreen auf Overview
            let mut next_setup = app
                .world_mut()
                .resource_mut::<NextState<SingleplayerMenuScreen>>();
            next_setup.set(SingleplayerMenuScreen::Overview);
            update_app(&mut app, 1);

            // Verifiziere den initialen Zustand
            let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
            assert_eq!(setup.get(), &SingleplayerMenuScreen::Overview);

            app
        }

        /// Setup für Tests: Setzt den SingleplayerMenuScreen auf NewGame mit ConfigPlayer Screen.
        pub fn setup_test_app_in_new_game() -> App {
            let mut app = setup_test_app_in_overview();

            // Navigiere zu NewGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::NewGame));
            update_app(&mut app, 1);

            // Verifiziere den Zustand
            let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
            assert_eq!(setup.get(), &SingleplayerMenuScreen::NewGame);

            let screen = app.world().resource::<State<NewGameMenuScreen>>();
            assert_eq!(screen.get(), &NewGameMenuScreen::ConfigPlayer);

            app
        }

        /// Setup für Tests: Setzt den SingleplayerMenuScreen auf LoadGame.
        pub fn setup_test_app_in_load_game() -> App {
            let mut app = setup_test_app_in_overview();

            // Navigiere zu LoadGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::LoadGame));
            update_app(&mut app, 1);

            // Verifiziere den Zustand
            let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
            assert_eq!(setup.get(), &SingleplayerMenuScreen::LoadGame);

            let screen = app.world().resource::<State<SavedGameMenuScreen>>();
            assert_eq!(screen.get(), &SavedGameMenuScreen::SelectSaveGame);

            app
        }

        /// Prüft, ob der SingleplayerMenuScreen State dem erwarteten Wert entspricht.
        pub fn assert_setup_state(app: &mut App, expected: SingleplayerMenuScreen) {
            let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
            assert_eq!(setup.get(), &expected);
        }

        /// Prüft, ob der MainMenuScreen State dem erwarteten Wert entspricht.
        pub fn assert_main_menu_context(app: &mut App, expected: MainMenuScreen) {
            let context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(context.get(), &expected);
        }

        /// Prüft, ob der NewGameMenuScreen State dem erwarteten Wert entspricht.
        pub fn assert_new_game_screen(app: &mut App, expected: NewGameMenuScreen) {
            let screen = app.world().resource::<State<NewGameMenuScreen>>();
            assert_eq!(screen.get(), &expected);
        }

        /// Prüft, ob der SavedGameMenuScreen State dem erwarteten Wert entspricht.
        pub fn assert_load_game_screen(app: &mut App, expected: SavedGameMenuScreen) {
            let screen = app.world().resource::<State<SavedGameMenuScreen>>();
            assert_eq!(screen.get(), &expected);
        }

        /// Prüft, ob der SessionType State dem erwarteten Wert entspricht.
        pub fn assert_session_type(app: &mut App, expected: SessionType) {
            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &expected);
        }

        /// Prüft, ob der ServerStatus State dem erwarteten Wert entspricht.
        /// Gibt true zurück wenn der State existiert und übereinstimmt.
        pub fn assert_server_status(app: &mut App, expected: ServerStatus) -> bool {
            app.world()
                .get_resource::<State<ServerStatus>>()
                .map_or(false, |status| status.get() == &expected)
        }

        /// Prüft, ob der ServerVisibility State dem erwarteten Wert entspricht.
        /// Gibt true zurück wenn der State existiert und übereinstimmt.
        pub fn assert_server_visibility(app: &mut App, expected: ServerVisibility) -> bool {
            app.world()
                .get_resource::<State<ServerVisibility>>()
                .map_or(false, |visibility| visibility.get() == &expected)
        }
    }

    // =============================================================================
    // TESTS FÜR OVERVIEW NAVIGATION
    // =============================================================================

    mod overview_nav_tests {
        use super::*;

        /// Test: Navigate zu NewGame wechselt den Setup State zu NewGame.
        #[test]
        fn test_overview_navigate_to_new_game() {
            let mut app = helpers::setup_test_app_in_overview();

            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::NewGame));
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);
        }

        /// Test: Navigate zu LoadGame wechselt den Setup State zu LoadGame.
        #[test]
        fn test_overview_navigate_to_load_game() {
            let mut app = helpers::setup_test_app_in_overview();

            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::LoadGame));
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::LoadGame);
        }

        /// Test: Back in Overview wechselt zurück zum Hauptmenü.
        #[test]
        fn test_overview_back_to_main_menu() {
            let mut app = helpers::setup_test_app_in_overview();

            app.world_mut().trigger(SetSingleplayerMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_context(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Events im falschen State werden ignoriert.
        #[test]
        fn test_overview_wrong_state_ignored() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Versuche Navigate aus NewGame heraus (sollte ignoriert werden)
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::LoadGame));
            helpers::update_app(&mut app, 1);

            // Sollte immer noch NewGame sein
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);
        }
    }

    // =============================================================================
    // TESTS FÜR NEW GAME NAVIGATION
    // =============================================================================

    mod new_game_nav_tests {
        use super::*;

        /// Test: Next von ConfigPlayer wechselt zu ConfigWorld.
        #[test]
        fn test_new_game_next_from_config_player() {
            let mut app = helpers::setup_test_app_in_new_game();

            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);
        }

        /// Test: Next von ConfigWorld wechselt zu ConfigSave.
        #[test]
        fn test_new_game_next_from_config_world() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Gehe zu ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Gehe zu ConfigSave
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigSave);
        }

        /// Test: Next von ConfigSave bleibt bei ConfigSave (Ende erreicht).
        #[test]
        fn test_new_game_next_from_config_save_stays() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Gehe zu ConfigSave
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Versuche weiter zu gehen
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigSave);
        }

        /// Test: Previous von ConfigWorld wechselt zurück zu ConfigPlayer.
        #[test]
        fn test_new_game_previous_from_config_world() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Gehe zu ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Gehe zurück zu ConfigPlayer
            app.world_mut().trigger(SetSingleplayerNewGame::Previous);
            helpers::update_app(&mut app, 1);

            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);
        }

        /// Test: Previous von ConfigSave wechselt zurück zu ConfigWorld.
        #[test]
        fn test_new_game_previous_from_config_save() {
            let mut app = helpers::setup_test_app_in_new_game();

            // Gehe zu ConfigSave
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Gehe zurück zu ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Previous);
            helpers::update_app(&mut app, 1);

            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);
        }

        /// Test: Previous von ConfigPlayer wechselt zurück zu Overview.
        #[test]
        fn test_new_game_previous_from_config_player_returns_overview() {
            let mut app = helpers::setup_test_app_in_new_game();

            app.world_mut().trigger(SetSingleplayerNewGame::Previous);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Confirm startet eine Singleplayer-Session.
        #[test]
        fn test_new_game_confirm_starts_singleplayer() {
            let mut app = helpers::setup_test_app_in_new_game();

            app.world_mut().trigger(SetSingleplayerNewGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            // ServerStatus ist ein SubState von SessionType::Singleplayer
            assert!(helpers::assert_server_status(
                &mut app,
                ServerStatus::Starting
            ));
            // ServerVisibility ist ein SubState von ServerStatus::Running,
            // daher existiert es noch nicht wenn ServerStatus::Starting
        }

        /// Test: Cancel wechselt zurück zu Overview.
        #[test]
        fn test_new_game_cancel_returns_overview() {
            let mut app = helpers::setup_test_app_in_new_game();

            app.world_mut().trigger(SetSingleplayerNewGame::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Back wechselt zurück zu Overview.
        #[test]
        fn test_new_game_back_returns_overview() {
            let mut app = helpers::setup_test_app_in_new_game();

            app.world_mut().trigger(SetSingleplayerNewGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Events im falschen State werden ignoriert.
        #[test]
        fn test_new_game_wrong_state_ignored() {
            let mut app = helpers::setup_test_app_in_overview();

            // Versuche NewGame Event aus Overview heraus
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Sollte immer noch Overview sein
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }
    }

    // =============================================================================
    // TESTS FÜR LOAD GAME NAVIGATION
    // =============================================================================

    mod load_game_nav_tests {
        use super::*;

        /// Test: Previous von SelectSaveGame wechselt zurück zu Overview.
        #[test]
        fn test_load_game_previous_returns_overview() {
            let mut app = helpers::setup_test_app_in_load_game();

            app.world_mut().trigger(SetSingleplayerSavedGame::Previous);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Confirm startet eine Singleplayer-Session.
        #[test]
        fn test_load_game_confirm_starts_singleplayer() {
            let mut app = helpers::setup_test_app_in_load_game();

            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            // ServerStatus ist ein SubState von SessionType::Singleplayer
            assert!(helpers::assert_server_status(
                &mut app,
                ServerStatus::Starting
            ));
            // ServerVisibility ist ein SubState von ServerStatus::Running,
            // daher existiert es noch nicht wenn ServerStatus::Starting
        }

        /// Test: Cancel wechselt zurück zu Overview.
        #[test]
        fn test_load_game_cancel_returns_overview() {
            let mut app = helpers::setup_test_app_in_load_game();

            app.world_mut().trigger(SetSingleplayerSavedGame::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Back wechselt zurück zu Overview.
        #[test]
        fn test_load_game_back_returns_overview() {
            let mut app = helpers::setup_test_app_in_load_game();

            app.world_mut().trigger(SetSingleplayerSavedGame::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Events im falschen State werden ignoriert.
        #[test]
        fn test_load_game_wrong_state_ignored() {
            let mut app = helpers::setup_test_app_in_overview();

            // Versuche LoadGame Event aus Overview heraus
            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);

            // Sollte immer noch Overview sein
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);

            // SessionType sollte None bleiben
            helpers::assert_session_type(&mut app, SessionType::None);
        }
    }

    // =============================================================================
    // INTEGRATIONSTESTS
    // =============================================================================

    mod integration_tests {
        use super::*;

        /// Test: Vollständiger NewGame Flow von Overview bis zum Session-Start.
        #[test]
        fn test_full_new_game_flow() {
            let mut app = helpers::setup_test_app_in_overview();

            // 1. Navigiere zu NewGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::NewGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);

            // 2. Next zu ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);

            // 3. Next zu ConfigSave
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigSave);

            // 4. Confirm startet die Session
            app.world_mut().trigger(SetSingleplayerNewGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            // ServerStatus ist ein SubState von SessionType::Singleplayer
            assert!(helpers::assert_server_status(
                &mut app,
                ServerStatus::Starting
            ));
            // ServerVisibility ist ein SubState von ServerStatus::Running,
            // daher existiert es noch nicht wenn ServerStatus::Starting
        }

        /// Test: Vollständiger LoadGame Flow von Overview bis zum Session-Start.
        #[test]
        fn test_full_load_game_flow() {
            let mut app = helpers::setup_test_app_in_overview();

            // 1. Navigiere zu LoadGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::LoadGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::LoadGame);
            helpers::assert_load_game_screen(&mut app, SavedGameMenuScreen::SelectSaveGame);

            // 2. Confirm lädt das Spiel und startet die Session
            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            // ServerStatus ist ein SubState von SessionType::Singleplayer
            assert!(helpers::assert_server_status(
                &mut app,
                ServerStatus::Starting
            ));
            // ServerVisibility ist ein SubState von ServerStatus::Running,
            // daher existiert es noch nicht wenn ServerStatus::Starting
        }

        /// Test: Navigation zurück vom NewGame Flow.
        #[test]
        fn test_new_game_flow_with_back_navigation() {
            let mut app = helpers::setup_test_app_in_overview();

            // Gehe zu NewGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::NewGame));
            helpers::update_app(&mut app, 1);

            // Gehe zu ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigWorld);

            // Gehe zurück zu ConfigPlayer
            app.world_mut().trigger(SetSingleplayerNewGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);

            // Gehe zurück zu Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Previous);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Cancel unterbricht den NewGame Flow.
        #[test]
        fn test_new_game_cancel_mid_flow() {
            let mut app = helpers::setup_test_app_in_overview();

            // Gehe zu NewGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::NewGame));
            helpers::update_app(&mut app, 1);

            // Gehe zu ConfigWorld
            app.world_mut().trigger(SetSingleplayerNewGame::Next);
            helpers::update_app(&mut app, 1);

            // Cancel bricht ab und geht zurück zu Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Cancel unterbricht den LoadGame Flow.
        #[test]
        fn test_load_game_cancel() {
            let mut app = helpers::setup_test_app_in_overview();

            // Gehe zu LoadGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::LoadGame));
            helpers::update_app(&mut app, 1);

            // Cancel bricht ab und geht zurück zu Overview
            app.world_mut().trigger(SetSingleplayerSavedGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);
        }

        /// Test: Overview -> NewGame -> Cancel -> LoadGame -> Confirm.
        #[test]
        fn test_mixed_navigation_flow() {
            let mut app = helpers::setup_test_app_in_overview();

            // Gehe zu NewGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::NewGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::NewGame);

            // Cancel zurück zu Overview
            app.world_mut().trigger(SetSingleplayerNewGame::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::Overview);

            // Gehe zu LoadGame
            app.world_mut()
                .trigger(SetSingleplayerMenu::To(SingleplayerMenuScreen::LoadGame));
            helpers::update_app(&mut app, 1);
            helpers::assert_setup_state(&mut app, SingleplayerMenuScreen::LoadGame);

            // Starte Session
            app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
            helpers::update_app(&mut app, 1);

            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_server_status(&mut app, ServerStatus::Starting);
        }
    }
}
