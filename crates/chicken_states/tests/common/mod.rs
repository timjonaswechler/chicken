use bevy::{prelude::*, state::app::StatesPlugin};
use chicken_states::{
    ChickenStatePlugin,
    events::menu::{
        multiplayer::SetMultiplayerMenu, settings::SetSettingsMenu,
        singleplayer::SetSingleplayerMenu,
    },
    states::{
        app::AppScope,
        menu::{
            main::MainMenuScreen,
            multiplayer::{
                HostNewGameMenuScreen, HostSavedGameMenuScreen, JoinGameMenuScreen,
                MultiplayerMenuScreen,
            },
            settings::SettingsMenuScreen,
            singleplayer::{NewGameMenuScreen, SavedGameMenuScreen, SingleplayerMenuScreen},
            wiki::WikiMenuScreen,
        },
        session::{ServerStartupStep, ServerStatus, SessionType},
    },
};

#[cfg(feature = "hosted")]
use chicken_states::{
    events::app::SetAppScope,
    events::session::{SetConnectingStep, SetServerStartupStep, SetSyncingStep},
    states::session::{ClientConnectionStatus, ConnectingStep, SyncingStep},
};

pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, ChickenStatePlugin));

    app
}

/// Runs the app for the specified number of update ticks.
pub fn update_app(app: &mut App, i: u8) {
    for _ in 0..i {
        app.update();
    }
}

/// Setup for tests with hosted feature: Starts at Splash, transitions to Session.
#[cfg(feature = "hosted")]
pub fn setup_test_app_hosted() -> App {
    let mut app = test_app();
    update_app(&mut app, 1);

    // Initial state should be Splash
    assert_app_scope(&mut app, AppScope::Splash);
    // Transition to Session
    app.world_mut().trigger(SetAppScope::Menu);
    update_app(&mut app, 1);

    assert_app_scope(&mut app, AppScope::Menu);
    assert_session_type(&mut app, SessionType::None);

    app
}

/// Setup helper: Sets MainMenuScreen to Singleplayer and SingleplayerMenuScreen to Overview.
#[cfg(feature = "hosted")]
pub fn setup_test_app_in_singleplayer() -> App {
    let mut app = setup_test_app_hosted();
    update_app(&mut app, 1);

    app.world_mut().trigger(SetSingleplayerMenu::Overview);
    update_app(&mut app, 1);

    assert_main_menu_screen(&mut app, MainMenuScreen::Singleplayer);
    assert_singleplayer_menu_screen(&mut app, SingleplayerMenuScreen::Overview);

    app
}

/// Setup helper: Sets SingleplayerMenuScreen to NewGame with ConfigPlayer screen.
#[cfg(feature = "hosted")]
pub fn setup_test_app_in_new_game() -> App {
    let mut app = setup_test_app_in_singleplayer();

    // Navigate to NewGame
    app.world_mut().trigger(SetSingleplayerMenu::NewGame);
    update_app(&mut app, 1);

    // Verify state
    assert_singleplayer_menu_screen(&mut app, SingleplayerMenuScreen::NewGame);
    assert_new_game_screen(&mut app, NewGameMenuScreen::ConfigPlayer);

    app
}

/// Setup helper: Sets SingleplayerMenuScreen to LoadGame.
#[cfg(feature = "hosted")]
pub fn setup_test_app_in_load_game() -> App {
    let mut app = setup_test_app_in_singleplayer();

    // Navigate to LoadGame
    app.world_mut().trigger(SetSingleplayerMenu::LoadGame);
    update_app(&mut app, 1);

    // Verify state
    assert_singleplayer_menu_screen(&mut app, SingleplayerMenuScreen::LoadGame);
    assert_load_game_screen(&mut app, SavedGameMenuScreen::SelectSaveGame);

    app
}

/// Setup helper: Sets MainMenuScreen to Settings and SettingsMenuScreen to Overview.
#[cfg(feature = "hosted")]
pub fn setup_test_app_in_settings() -> App {
    let mut app = setup_test_app_hosted();

    // Navigate to Settings
    app.world_mut().trigger(SetSettingsMenu::Overview);
    update_app(&mut app, 1);

    // Verify we're in Settings
    assert_main_menu_screen(&mut app, MainMenuScreen::Settings);
    assert_settings_screen(&mut app, SettingsMenuScreen::Overview);

    app
}

/// Setup helper: Sets MainMenuScreen to Multiplayer and MultiplayerMenuScreen to Overview.
pub fn setup_test_app_in_multiplayer_overview() -> App {
    let mut app = setup_test_app_hosted();

    app.world_mut().trigger(SetMultiplayerMenu::Overview);
    update_app(&mut app, 1);

    assert_main_menu_screen(&mut app, MainMenuScreen::Multiplayer);
    assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::Overview);

    app
}

/// Setup helper: Sets MultiplayerMenuScreen to HostNewGame with ConfigServer screen.
pub fn setup_test_app_in_host_new_game() -> App {
    let mut app = setup_test_app_in_multiplayer_overview();

    // Navigate to HostNewGame
    app.world_mut().trigger(SetMultiplayerMenu::HostNewGame);
    update_app(&mut app, 1);
    assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostNewGame);
    assert_host_new_game_screen(&mut app, HostNewGameMenuScreen::ConfigServer);

    app
}

/// Setup helper: Sets MultiplayerMenuScreen to HostSavedGame.
pub fn setup_test_app_in_host_saved_game() -> App {
    let mut app = setup_test_app_in_multiplayer_overview();

    // Navigate to HostSavedGame
    app.world_mut().trigger(SetMultiplayerMenu::HostSavedGame);
    update_app(&mut app, 1);

    assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::HostSavedGame);
    assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::Overview);

    app
}

/// Setup helper: Sets MultiplayerMenuScreen to JoinGame.
pub fn setup_test_app_in_join_game() -> App {
    let mut app = setup_test_app_in_multiplayer_overview();

    // Navigate to JoinGame
    app.world_mut().trigger(SetMultiplayerMenu::JoinGame);
    update_app(&mut app, 1);

    // Verify state
    assert_multiplayer_screen(&mut app, MultiplayerMenuScreen::JoinGame);
    assert_join_game_screen(&mut app, JoinGameMenuScreen::Overview);

    app
}

/// Setup for tests with headless feature: Starts directly in Session.
#[cfg(feature = "headless")]
pub fn setup_test_app_headless() -> App {
    let mut app = test_app();
    update_app(&mut app, 1);

    assert_app_scope(&mut app, AppScope::Session);
    assert_session_type(&mut app, SessionType::DedicatedServer);

    app
}

/// Asserts that the current SessionType matches the expected value.
pub fn assert_session_type(app: &mut App, expected: SessionType) {
    let session_type = app.world().resource::<State<SessionType>>();
    assert_eq!(session_type.get(), &expected);
}

/// Asserts that the current AppScope matches the expected value.
pub fn assert_app_scope(app: &mut App, expected: AppScope) {
    let app_scope = app.world().resource::<State<AppScope>>();
    assert_eq!(app_scope.get(), &expected);
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

/// Asserts that SingleplayerMenuScreen state matches expected value.
pub fn assert_setup_state(app: &mut App, expected: SingleplayerMenuScreen) {
    let setup = app.world().resource::<State<SingleplayerMenuScreen>>();
    assert_eq!(setup.get(), &expected);
}

/// Asserts that MainMenuScreen state matches expected value.
pub fn assert_main_menu_screen(app: &mut App, expected: MainMenuScreen) {
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

/// Asserts that ServerStatus state matches expected value.
pub fn assert_server_status(app: &mut App, expected: ServerStatus) {
    let status = app.world().resource::<State<ServerStatus>>();
    assert_eq!(status.get(), &expected);
}

pub fn assert_singleplayer_menu_screen(app: &mut App, expected: SingleplayerMenuScreen) {
    let screen = app.world().resource::<State<SingleplayerMenuScreen>>();
    assert_eq!(screen.get(), &expected);
}

/// Asserts that SettingsMenuScreen state matches expected value.
pub fn assert_settings_screen(app: &mut App, expected: SettingsMenuScreen) {
    let settings_state = app.world().resource::<State<SettingsMenuScreen>>();
    assert_eq!(settings_state.get(), &expected);
}

/// Asserts that MultiplayerMenuScreen state matches expected value.
pub fn assert_multiplayer_screen(app: &mut App, expected: MultiplayerMenuScreen) {
    let setup = app.world().resource::<State<MultiplayerMenuScreen>>();
    assert_eq!(setup.get(), &expected);
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

// =============================================================================
// Session — Server Helpers
// =============================================================================

/// Number of Next steps to reach Ready during server startup.
pub const SERVER_STARTUP_STEPS: u8 = 3;
/// Number of Next steps to reach Ready during server shutdown.
pub const SERVER_SHUTDOWN_STEPS: u8 = 5;
/// Number of Next steps to reach Ready during going-public.
pub const SERVER_GOING_PUBLIC_STEPS: u8 = 3;
/// Number of Next steps to reach Ready during going-private.
pub const SERVER_GOING_PRIVATE_STEPS: u8 = 4;

/// Setup: Singleplayer session started via the menu flow (NewGame → Confirm).
/// Returns app with ServerStatus::Starting (first startup step: Init).
#[cfg(feature = "hosted")]
pub fn setup_test_app_singleplayer_confirmed() -> App {
    use chicken_states::events::menu::singleplayer::SetSingleplayerNewGame;

    let mut app = setup_test_app_in_new_game();

    // Navigate to ConfigSave
    app.world_mut().trigger(SetSingleplayerNewGame::Next);
    update_app(&mut app, 1);
    app.world_mut().trigger(SetSingleplayerNewGame::Next);
    update_app(&mut app, 1);

    // Confirm starts the server
    app.world_mut().trigger(SetSingleplayerNewGame::Confirm);
    update_app(&mut app, 1);

    assert_session_type(&mut app, SessionType::Singleplayer);
    assert_server_status(&mut app, ServerStatus::Starting);
    assert_eq!(
        app.world().resource::<State<ServerStartupStep>>().get(),
        &ServerStartupStep::Init
    );

    app
}

/// Setup: Singleplayer session started via the menu flow (LoadGame → Confirm).
/// Returns app with ServerStatus::Starting (first startup step: Init).
#[cfg(feature = "hosted")]
pub fn setup_test_app_singleplayer_saved_confirmed() -> App {
    use chicken_states::events::menu::singleplayer::SetSingleplayerSavedGame;

    let mut app = setup_test_app_in_load_game();

    // Confirm immediately starts the server (only one step: SelectSaveGame)
    app.world_mut().trigger(SetSingleplayerSavedGame::Confirm);
    update_app(&mut app, 1);

    assert_session_type(&mut app, SessionType::Singleplayer);
    assert_server_status(&mut app, ServerStatus::Starting);
    assert_eq!(
        app.world().resource::<State<ServerStartupStep>>().get(),
        &ServerStartupStep::Init
    );

    app
}

/// Setup: Multiplayer host session started via the menu flow (HostNewGame → Confirm).
/// Returns app with ServerStatus::Starting and PendingGoingPublic inserted.
#[cfg(feature = "hosted")]
pub fn setup_test_app_multiplayer_host_confirmed() -> App {
    use chicken_states::events::menu::multiplayer::SetNewHostGame;

    let mut app = setup_test_app_in_host_new_game();

    // Navigate to ConfigSave
    app.world_mut().trigger(SetNewHostGame::Next);
    update_app(&mut app, 1);
    app.world_mut().trigger(SetNewHostGame::Next);
    update_app(&mut app, 1);

    // Confirm starts the server with PendingGoingPublic
    app.world_mut().trigger(SetNewHostGame::Confirm);
    update_app(&mut app, 1);

    assert_session_type(&mut app, SessionType::Singleplayer);
    assert_server_status(&mut app, ServerStatus::Starting);
    assert_eq!(
        app.world().resource::<State<ServerStartupStep>>().get(),
        &ServerStartupStep::Init
    );

    app
}

/// Setup: Multiplayer saved-game host session started via the menu flow (HostSavedGame → Confirm).
/// Returns app with ServerStatus::Starting and PendingGoingPublic inserted.
#[cfg(feature = "hosted")]
pub fn setup_test_app_multiplayer_saved_host_confirmed() -> App {
    use chicken_states::events::menu::multiplayer::SetSavedHostGame;

    let mut app = setup_test_app_in_host_saved_game();

    // Navigate: Overview → ConfigServer
    app.world_mut().trigger(SetSavedHostGame::Next);
    update_app(&mut app, 1);
    assert_host_saved_game_screen(&mut app, HostSavedGameMenuScreen::ConfigServer);

    // Confirm starts the server with PendingGoingPublic
    app.world_mut().trigger(SetSavedHostGame::Confirm);
    update_app(&mut app, 1);

    assert_session_type(&mut app, SessionType::Singleplayer);
    assert_server_status(&mut app, ServerStatus::Starting);
    assert_eq!(
        app.world().resource::<State<ServerStartupStep>>().get(),
        &ServerStartupStep::Init
    );

    app
}

/// Runs through all startup Next steps and triggers Done → server reaches Running.
#[cfg(feature = "hosted")]
pub fn server_startup_complete(app: &mut App) {
    for _ in 0..SERVER_STARTUP_STEPS {
        app.world_mut().trigger(SetServerStartupStep::Next);
        update_app(app, 1);
    }
    app.world_mut().trigger(SetServerStartupStep::Done);
    update_app(app, 1);

    assert_server_status(app, ServerStatus::Running);
}

/// Asserts that ServerStartupStep state matches expected value.
pub fn assert_server_startup_step(app: &mut App, expected: ServerStartupStep) {
    let step = app.world().resource::<State<ServerStartupStep>>();
    assert_eq!(step.get(), &expected);
}

// =============================================================================
// Session — Client Helpers
// =============================================================================

/// Number of Next steps to reach Ready during connecting.
#[cfg(feature = "hosted")]
pub const CLIENT_CONNECTING_STEPS: u8 = 4;
/// Number of Next steps to reach Ready during syncing.
#[cfg(feature = "hosted")]
pub const CLIENT_SYNCING_STEPS: u8 = 3;
/// Number of Next steps to reach Ready during disconnecting.
#[cfg(feature = "hosted")]
pub const CLIENT_DISCONNECTING_STEPS: u8 = 3;

/// Setup: Client session started via the menu flow (JoinGame → Confirm).
/// Confirm sets SessionType::Client + AppScope::Session + ClientConnectionStatus::Connecting.
#[cfg(feature = "hosted")]
pub fn setup_test_app_for_client() -> App {
    use chicken_states::events::menu::multiplayer::SetJoinGame;

    let mut app = setup_test_app_in_join_game();

    app.world_mut().trigger(SetJoinGame::Confirm);
    update_app(&mut app, 1);

    assert_session_type(&mut app, SessionType::Client);
    assert_client_status(&mut app, ClientConnectionStatus::Connecting);

    app
}

/// Runs through all connecting Next steps + Done → ClientConnectionStatus::Connected.
/// Assumes the client is already in Connecting state (e.g. after SetJoinGame::Confirm).
#[cfg(feature = "hosted")]
pub fn client_connect_complete(app: &mut App) {
    for _ in 0..CLIENT_CONNECTING_STEPS {
        app.world_mut().trigger(SetConnectingStep::Next);
        update_app(app, 1);
    }
    app.world_mut().trigger(SetConnectingStep::Done);
    update_app(app, 1);

    assert_client_status(app, ClientConnectionStatus::Connected);
}

/// Triggers Start + all Next steps + Done for syncing → ClientConnectionStatus::Playing.
#[cfg(feature = "hosted")]
pub fn client_sync_complete(app: &mut App) {
    app.world_mut().trigger(SetSyncingStep::Start);
    update_app(app, 1);

    for _ in 0..CLIENT_SYNCING_STEPS {
        app.world_mut().trigger(SetSyncingStep::Next);
        update_app(app, 1);
    }
    app.world_mut().trigger(SetSyncingStep::Done);
    update_app(app, 1);

    assert_eq!(
        app.world()
            .resource::<State<ClientConnectionStatus>>()
            .get(),
        &ClientConnectionStatus::Playing
    );
}

/// Triggers Start + all Next steps + Done for disconnecting.
/// After Done, AppScope returns to Menu and SessionType resets to None.
#[cfg(feature = "hosted")]
pub fn client_disconnect_complete(app: &mut App) {
    use chicken_states::events::session::SetDisconnectingStep;
    app.world_mut().trigger(SetDisconnectingStep::Start);
    update_app(app, 1);

    for _ in 0..CLIENT_DISCONNECTING_STEPS {
        app.world_mut()
            .trigger(SetDisconnectingStep::Next);
        update_app(app, 1);
    }
    app.world_mut().trigger(SetDisconnectingStep::Done);
    update_app(app, 1);

    // After disconnect, app returns to Menu with SessionType::None
    assert_app_scope(app, AppScope::Menu);
    assert_session_type(app, SessionType::None);
}

/// Asserts ClientConnectionStatus matches expected value.
#[cfg(feature = "hosted")]
pub fn assert_client_status(app: &mut App, expected: ClientConnectionStatus) {
    let status = app.world().resource::<State<ClientConnectionStatus>>();
    assert_eq!(status.get(), &expected);
}

/// Asserts ConnectingStep matches expected value.
#[cfg(feature = "hosted")]
pub fn assert_connecting_step(app: &mut App, expected: ConnectingStep) {
    let step = app.world().resource::<State<ConnectingStep>>();
    assert_eq!(step.get(), &expected);
}

/// Asserts SyncingStep matches expected value.
#[cfg(feature = "hosted")]
pub fn assert_syncing_step(app: &mut App, expected: SyncingStep) {
    let step = app.world().resource::<State<SyncingStep>>();
    assert_eq!(step.get(), &expected);
}
