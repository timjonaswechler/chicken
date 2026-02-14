use {
    crate::{
        events::{
            app::ChangeAppScope,
            menu::PauseMenuEvent,
            session::{
                SetClientShutdownStep, SetClientStatus, SetSingleplayerShutdownStep,
                SetSingleplayerStatus,
            },
        },
        states::{
            app::AppScope,
            menu::{PauseMenu, main::MainMenuContext},
            session::{
                ClientShutdownStep, ClientStatus, SessionState, SessionType,
                SingleplayerShutdownStep, SingleplayerStatus,
            },
        },
    },
    bevy::prelude::{
        App, AppExtStates, ButtonInput, IntoScheduleConfigs, KeyCode, NextState, On, Plugin, Res,
        ResMut, State, SystemCondition, Update, in_state,
    },
};

pub struct ClientSessionPlugin;

impl Plugin for ClientSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ClientStatus>()
            .add_sub_state::<ClientShutdownStep>()
            .add_sub_state::<SingleplayerStatus>()
            .add_sub_state::<SingleplayerShutdownStep>()
            .add_sub_state::<PauseMenu>()
            .init_state::<SessionType>() // Used internally by client to know mode
            .add_observer(on_client_state_event)
            .add_observer(on_set_client_shutdown_step)
            .add_observer(on_set_singleplayer_status)
            .add_observer(on_set_singleplayer_shutdown_step)
            .add_observer(handle_pause_menu_nav)
            .add_observer(on_change_app_scope)
            .add_systems(
                Update,
                toggle_game_menu
                    .run_if(in_state(SessionState::Active).or(in_state(SessionState::Paused))),
            );
    }
}

// --- App Scope Logic (moved from app.rs) ---
fn on_change_app_scope(
    event: On<ChangeAppScope>,
    mut state: ResMut<NextState<AppScope>>,
    mut session_type: ResMut<NextState<SessionType>>,
    mut menu_state: ResMut<NextState<MainMenuContext>>,
) {
    if event.transition == AppScope::Menu {
        state.set(AppScope::Menu);
        session_type.set(SessionType::None);
        menu_state.set(MainMenuContext::Main);
    }
}

// --- Pause Menu Logic ---

fn toggle_game_menu(
    current_state: Res<State<SessionState>>,
    mut next_state: ResMut<NextState<SessionState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            SessionState::Active => next_state.set(SessionState::Paused),
            SessionState::Paused => next_state.set(SessionState::Active),
            _ => {}
        }
    }
}

fn handle_pause_menu_nav(
    trigger: On<PauseMenuEvent>,
    mut next_pause_menu: ResMut<NextState<PauseMenu>>,
    mut next_session_state: ResMut<NextState<SessionState>>,
    session_type: Res<State<SessionType>>,
    mut next_client_status: ResMut<NextState<ClientStatus>>,
    mut next_sp_status: ResMut<NextState<SingleplayerStatus>>,
    mut next_singleplayer_shutdown_step: ResMut<NextState<SingleplayerShutdownStep>>,
) {
    match trigger.event() {
        PauseMenuEvent::Resume => {
            next_session_state.set(SessionState::Active);
        }
        PauseMenuEvent::Settings => {
            next_pause_menu.set(PauseMenu::Settings);
        }
        PauseMenuEvent::Save => {
            next_pause_menu.set(PauseMenu::Save);
        }
        PauseMenuEvent::Load => {
            next_pause_menu.set(PauseMenu::Load);
        }
        PauseMenuEvent::Exit => match session_type.get() {
            SessionType::Singleplayer => {
                next_sp_status.set(SingleplayerStatus::Stopping);
                next_singleplayer_shutdown_step
                    .set(SingleplayerShutdownStep::DisconnectRemoteClients);
            }
            SessionType::Client => {
                next_client_status.set(ClientStatus::Disconnecting);
            }
            SessionType::None => {}
        },
    }
}

// --- Client Network Logic ---

fn on_client_state_event(
    event: On<SetClientStatus>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_session_state: ResMut<NextState<SessionState>>,
    mut next_state: ResMut<NextState<ClientStatus>>,
) {
    match *event {
        SetClientStatus::Transition(ClientStatus::Connecting) => {
            next_state.set(ClientStatus::Connecting);
            next_session_type.set(SessionType::Client);
        }
        SetClientStatus::Transition(ClientStatus::Connected) => {
            next_state.set(ClientStatus::Connected);
        }
        SetClientStatus::Transition(ClientStatus::Syncing) => {
            next_state.set(ClientStatus::Syncing);
            next_app_scope.set(AppScope::Session);
        }
        SetClientStatus::Transition(ClientStatus::Running) => {
            next_state.set(ClientStatus::Running);
            next_session_state.set(SessionState::Active);
        }
        SetClientStatus::Transition(ClientStatus::Disconnecting) => {
            next_state.set(ClientStatus::Disconnecting);
        }

        SetClientStatus::Failed => {
            next_session_type.set(SessionType::None);
        }
    }
}

fn on_set_client_shutdown_step(
    event: On<SetClientShutdownStep>,
    shutdown_state: Res<State<ClientShutdownStep>>,
    mut next_main_menu: ResMut<NextState<MainMenuContext>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_state: ResMut<NextState<ClientShutdownStep>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
) {
    match *event {
        SetClientShutdownStep::Start => {
            next_state.set(ClientShutdownStep::DisconnectFromServer);
        }
        SetClientShutdownStep::Next => match **shutdown_state {
            ClientShutdownStep::DisconnectFromServer => {
                next_state.set(ClientShutdownStep::DespawnLocalClient);
            }
            ClientShutdownStep::DespawnLocalClient => {}
        },
        SetClientShutdownStep::Done => {
            next_app_scope.set(AppScope::Menu);
            next_main_menu.set(MainMenuContext::Main);
            next_session_type.set(SessionType::None);
        }
    }
}

// --- Singleplayer Logic ---

fn on_set_singleplayer_status(
    event: On<SetSingleplayerStatus>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_session_state: ResMut<NextState<SessionState>>,
    mut next_state: ResMut<NextState<SingleplayerStatus>>,
) {
    match event.transition {
        SingleplayerStatus::Running => {
            next_state.set(SingleplayerStatus::Running);
            next_app_scope.set(AppScope::Session);
            next_session_state.set(SessionState::Active);
        }
        SingleplayerStatus::Stopping => {
            next_state.set(SingleplayerStatus::Stopping);
        }
        SingleplayerStatus::Starting => {
            next_state.set(SingleplayerStatus::Starting);
        }
    }
}

fn on_set_singleplayer_shutdown_step(
    event: On<SetSingleplayerShutdownStep>,
    shutdown_state: Res<State<SingleplayerShutdownStep>>,
    mut next_main_menu: ResMut<NextState<MainMenuContext>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_state: ResMut<NextState<SingleplayerShutdownStep>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
) {
    match *event {
        SetSingleplayerShutdownStep::Start => {
            next_state.set(SingleplayerShutdownStep::DisconnectRemoteClients);
        }
        SetSingleplayerShutdownStep::Next => match **shutdown_state {
            SingleplayerShutdownStep::DisconnectRemoteClients => {
                next_state.set(SingleplayerShutdownStep::CloseRemoteServer);
            }
            SingleplayerShutdownStep::CloseRemoteServer => {
                next_state.set(SingleplayerShutdownStep::DespawnBots);
            }
            SingleplayerShutdownStep::DespawnBots => {
                next_state.set(SingleplayerShutdownStep::DespawnLocalClient);
            }
            SingleplayerShutdownStep::DespawnLocalClient => {
                next_state.set(SingleplayerShutdownStep::DespawnLocalServer);
            }
            SingleplayerShutdownStep::DespawnLocalServer => {}
        },
        SetSingleplayerShutdownStep::Done => {
            next_app_scope.set(AppScope::Menu);
            next_main_menu.set(MainMenuContext::Main);
            next_session_type.set(SessionType::None);
        }
    }
}
