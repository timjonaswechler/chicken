use {
    crate::{
        events::menu::multiplayer::{
            SetJoinGame, SetMultiplayerMenu, SetNewHostGame, SetSavedHostGame,
        },
        states::{
            menu::{
                main::MainMenuContext,
                multiplayer::{
                    HostNewGameMenuScreen, HostSavedGameMenuScreen, JoinGameMenuScreen,
                    MultiplayerSetup,
                },
            },
            session::{ClientConnectionStatus, ServerStatus, ServerVisibility, SessionType},
        },
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State},
};

pub(super) struct MultiplayerMenuPlugin;

impl Plugin for MultiplayerMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MultiplayerSetup>()
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
    current_setup: Res<State<MultiplayerSetup>>,
    mut next_setup: ResMut<NextState<MultiplayerSetup>>,
    mut next_main_menu: ResMut<NextState<MainMenuContext>>,
) {
    if *current_setup.get() != MultiplayerSetup::Overview {
        return;
    }

    match trigger.event() {
        SetMultiplayerMenu::Navigate(target) => next_setup.set(*target),
        SetMultiplayerMenu::Back => next_main_menu.set(MainMenuContext::Main),
    }
}

fn handle_host_new_game_nav(
    trigger: On<SetNewHostGame>,
    current_screen: Res<State<HostNewGameMenuScreen>>,
    mut next_screen: ResMut<NextState<HostNewGameMenuScreen>>,
    mut next_setup: ResMut<NextState<MultiplayerSetup>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    current_setup: Res<State<MultiplayerSetup>>,
) {
    if *current_setup.get() != MultiplayerSetup::HostNewGame {
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
            HostNewGameMenuScreen::ConfigServer => next_setup.set(MultiplayerSetup::Overview),
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
        SetNewHostGame::Cancel => next_setup.set(MultiplayerSetup::Overview),
        SetNewHostGame::Back => next_setup.set(MultiplayerSetup::Overview),
    }
}

fn handle_host_saved_game_nav(
    trigger: On<SetSavedHostGame>,
    current_screen: Res<State<HostSavedGameMenuScreen>>,
    mut next_screen: ResMut<NextState<HostSavedGameMenuScreen>>,
    mut next_setup: ResMut<NextState<MultiplayerSetup>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_state: ResMut<NextState<ServerVisibility>>,
    current_setup: Res<State<MultiplayerSetup>>,
) {
    if *current_setup.get() != MultiplayerSetup::HostSavedGame {
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
        SetSavedHostGame::Cancel => next_setup.set(MultiplayerSetup::Overview),
        SetSavedHostGame::Back => next_setup.set(MultiplayerSetup::Overview),
    }
}

fn handle_join_game_nav(
    trigger: On<SetJoinGame>,
    current_setup: Res<State<MultiplayerSetup>>,
    mut next_setup: ResMut<NextState<MultiplayerSetup>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_client_state: ResMut<NextState<ClientConnectionStatus>>,
) {
    if *current_setup.get() != MultiplayerSetup::JoinGame {
        return;
    }

    match trigger.event() {
        SetJoinGame::Back => next_setup.set(MultiplayerSetup::Overview),
        SetJoinGame::Confirm => {
            next_session_type.set(SessionType::Client);
            next_client_state.set(ClientConnectionStatus::Connecting);
        }
        SetJoinGame::Cancel => next_setup.set(MultiplayerSetup::Overview),
        _ => {}
    }
}
