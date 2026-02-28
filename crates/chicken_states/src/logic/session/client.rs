use {
    crate::{
        events::{
            menu::PauseMenuEvent,
            session::{
                SetClientConnectionStatus, SetConnectingStep, SetDisconnectingStep, SetSyncingStep,
            },
        },
        states::{
            app::AppScope,
            menu::{PauseMenu, main::MainMenuContext},
            session::{
                ClientConnectionStatus, ConnectingStep, DisconnectingStep, ServerShutdownStep,
                ServerStatus, SessionState, SessionType, SyncingStep,
            },
        },
    },
    bevy::prelude::{
        App, AppExtStates, ButtonInput, IntoScheduleConfigs, KeyCode, NextState, On, Plugin, Res,
        ResMut, State, SystemCondition, Update, in_state, warn,
    },
};

pub struct ClientSessionPlugin;

impl Plugin for ClientSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ClientConnectionStatus>()
            .add_sub_state::<ConnectingStep>()
            .add_sub_state::<SyncingStep>()
            .add_sub_state::<DisconnectingStep>()
            .add_sub_state::<PauseMenu>()

            .add_observer(on_client_connection_status_event)
            .add_observer(on_connecting_step)
            .add_observer(on_syncing_step)
            .add_observer(on_set_disconnecting_step)

            .add_observer(handle_pause_menu_nav) // TODO: evtl. falscher platz.
            .add_systems(
                Update,
                toggle_game_menu // TODO: evtl. falscher platz.
                    .run_if(in_state(SessionState::Active).or(in_state(SessionState::Paused))),
            );
    }
}

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
    mut next_client_status: ResMut<NextState<ClientConnectionStatus>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_server_shutdown_step: ResMut<NextState<ServerShutdownStep>>,
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
                next_server_status.set(ServerStatus::Stopping);
                next_server_shutdown_step.set(ServerShutdownStep::SaveWorld);
            }
            SessionType::Client => {
                next_client_status.set(ClientConnectionStatus::Disconnecting);
            }
            SessionType::None => {}
            #[cfg(feature = "headless")]
            SessionType::DedicatedServer => {}
        },
    }
}

fn is_valid_client_connection_transition(
    from: &ClientConnectionStatus,
    to: &ClientConnectionStatus,
) -> bool {
    matches!(
        (from, to),
        (
            ClientConnectionStatus::Disconnected,
            ClientConnectionStatus::Connecting
        ) | (
            ClientConnectionStatus::Connecting,
            ClientConnectionStatus::Connected
        ) | (
            ClientConnectionStatus::Connecting,
            ClientConnectionStatus::Disconnected
        ) | (
            ClientConnectionStatus::Connected,
            ClientConnectionStatus::Syncing
        ) | (
            ClientConnectionStatus::Syncing,
            ClientConnectionStatus::Playing
        ) | (
            ClientConnectionStatus::Playing,
            ClientConnectionStatus::Disconnecting
        ) | (
            ClientConnectionStatus::Disconnecting,
            ClientConnectionStatus::Disconnected
        )
    )
}

fn on_client_connection_status_event(
    event: On<SetClientConnectionStatus>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_session_state: ResMut<NextState<SessionState>>,
    mut next_state: ResMut<NextState<ClientConnectionStatus>>,
    current: Res<State<ClientConnectionStatus>>,
) {
    if !is_valid_client_connection_transition(current.get(), &event.transition) {
        warn!(
            "Unexpected ClientConnectionStatus transition: {:?} -> {:?}",
            current.get(),
            event.transition
        );
        return;
    }

    match event.transition {
        ClientConnectionStatus::Connecting => {
            next_state.set(ClientConnectionStatus::Connecting);
            next_session_type.set(SessionType::Client);
        }
        ClientConnectionStatus::Connected => {
            next_state.set(ClientConnectionStatus::Connected);
        }
        ClientConnectionStatus::Syncing => {
            next_state.set(ClientConnectionStatus::Syncing);
            next_app_scope.set(AppScope::Session);
        }
        ClientConnectionStatus::Playing => {
            next_state.set(ClientConnectionStatus::Playing);
            next_session_state.set(SessionState::Active);
        }
        ClientConnectionStatus::Disconnecting => {
            next_state.set(ClientConnectionStatus::Disconnecting);
        }
        ClientConnectionStatus::Disconnected => {
            next_state.set(ClientConnectionStatus::Disconnected);
        }
    }
}

fn on_connecting_step(
    event: On<SetConnectingStep>,
    current: Res<State<ConnectingStep>>,
    mut next_state: ResMut<NextState<ConnectingStep>>,
    mut next_client_status: ResMut<NextState<ClientConnectionStatus>>,
) {
    match *event.event() {
        SetConnectingStep::Start => {
            next_state.set(ConnectingStep::ResolveAddress);
        }
        SetConnectingStep::Next => match current.get() {
            ConnectingStep::ResolveAddress => next_state.set(ConnectingStep::OpenSocket),
            ConnectingStep::OpenSocket => next_state.set(ConnectingStep::SendHandshake),
            ConnectingStep::SendHandshake => next_state.set(ConnectingStep::WaitForAccept),
            ConnectingStep::WaitForAccept => next_state.set(ConnectingStep::Ready),
            ConnectingStep::Ready => {}
        },
        SetConnectingStep::Done => {
            next_client_status.set(ClientConnectionStatus::Connected);
        }
        SetConnectingStep::Failed => {
            next_client_status.set(ClientConnectionStatus::Disconnected);
        }
    }
}

fn on_syncing_step(
    event: On<SetSyncingStep>,
    current: Res<State<SyncingStep>>,
    mut next_state: ResMut<NextState<SyncingStep>>,
    mut next_client_status: ResMut<NextState<ClientConnectionStatus>>,
) {
    match *event.event() {
        SetSyncingStep::Start => {
            next_state.set(SyncingStep::RequestWorld);
        }
        SetSyncingStep::Next => match current.get() {
            SyncingStep::RequestWorld => next_state.set(SyncingStep::ReceiveChunks),
            SyncingStep::ReceiveChunks => next_state.set(SyncingStep::SpawnEntities),
            SyncingStep::SpawnEntities => next_state.set(SyncingStep::Ready),
            SyncingStep::Ready => {}
        },
        SetSyncingStep::Done => {
            next_client_status.set(ClientConnectionStatus::Playing);
        }
        SetSyncingStep::Failed => {
            todo!("Failed to sync");
        }
    }
}

fn on_set_disconnecting_step(
    event: On<SetDisconnectingStep>,
    shutdown_state: Res<State<DisconnectingStep>>,
    mut next_main_menu: ResMut<NextState<MainMenuContext>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_state: ResMut<NextState<DisconnectingStep>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_client_status: ResMut<NextState<ClientConnectionStatus>>,
) {
    match *event {
        SetDisconnectingStep::Start => {
            next_state.set(DisconnectingStep::SendDisconnect);
        }
        SetDisconnectingStep::Next => match **shutdown_state {
            DisconnectingStep::SendDisconnect => {
                next_state.set(DisconnectingStep::WaitForAck);
            }
            DisconnectingStep::WaitForAck => {
                next_state.set(DisconnectingStep::Cleanup);
            }
            DisconnectingStep::Cleanup => {
                next_state.set(DisconnectingStep::Ready);
            }
            DisconnectingStep::Ready => {}
        },
        SetDisconnectingStep::Done => {
            next_client_status.set(ClientConnectionStatus::Disconnected);
            next_app_scope.set(AppScope::Menu);
            next_main_menu.set(MainMenuContext::Main);
            next_session_type.set(SessionType::None);
        }
        SetDisconnectingStep::Failed => {
            todo!("Failed to disconnect");
        }
    }
}
