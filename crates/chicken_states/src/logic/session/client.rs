use {
    crate::{
        events::{
            menu::PauseMenuEvent,
            session::{SetConnectingStep, SetDisconnectingStep, SetSyncingStep},
        },
        states::{
            app::AppScope,
            menu::PauseMenu,
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

// =============================================================================
// VALIDATOR-FUNKTIONEN
// =============================================================================

/// Validates transitions for ClientConnectionStatus when starting to connect.
pub(crate) fn is_valid_client_status_connecting_transition(
    from: &ClientConnectionStatus,
    to: &SetConnectingStep,
) -> bool {
    matches!(
        (from, to),
        (
            ClientConnectionStatus::Disconnected,
            SetConnectingStep::Start
        ) | (ClientConnectionStatus::Connecting, SetConnectingStep::Next)
            | (ClientConnectionStatus::Connecting, SetConnectingStep::Done)
            | (_, SetConnectingStep::Failed)
    )
}

/// Validates transitions for ClientConnectionStatus when starting to sync.
pub(crate) fn is_valid_client_status_syncing_transition(
    from: &ClientConnectionStatus,
    to: &SetSyncingStep,
) -> bool {
    matches!(
        (from, to),
        (ClientConnectionStatus::Connected, SetSyncingStep::Start)
            | (ClientConnectionStatus::Syncing, SetSyncingStep::Next)
            | (ClientConnectionStatus::Syncing, SetSyncingStep::Done)
            | (_, SetSyncingStep::Failed)
    )
}

/// Validates transitions for ClientConnectionStatus when starting to disconnect.
pub(crate) fn is_valid_client_status_disconnecting_transition(
    from: &ClientConnectionStatus,
    to: &SetDisconnectingStep,
) -> bool {
    matches!(
        (from, to),
        (ClientConnectionStatus::Playing, SetDisconnectingStep::Start)
            | (
                ClientConnectionStatus::Disconnecting,
                SetDisconnectingStep::Next
            )
            | (
                ClientConnectionStatus::Disconnecting,
                SetDisconnectingStep::Done
            )
            | (_, SetDisconnectingStep::Failed)
            | (
                ClientConnectionStatus::Connecting,
                SetDisconnectingStep::Start
            )
    )
}

/// Validates transitions between ConnectingStep states.
pub(crate) fn is_valid_connecting_step_transition(
    from: &ConnectingStep,
    to: &SetConnectingStep,
) -> bool {
    matches!(
        (from, to),
        (ConnectingStep::ResolveAddress, SetConnectingStep::Next)
            | (ConnectingStep::OpenSocket, SetConnectingStep::Next)
            | (ConnectingStep::SendHandshake, SetConnectingStep::Next)
            | (ConnectingStep::WaitForAccept, SetConnectingStep::Next)
            | (ConnectingStep::Ready, SetConnectingStep::Done)
            | (_, SetConnectingStep::Failed)
    )
}

/// Validates transitions between SyncingStep states.
pub(crate) fn is_valid_syncing_step_transition(from: &SyncingStep, to: &SetSyncingStep) -> bool {
    matches!(
        (from, to),
        (SyncingStep::RequestWorld, SetSyncingStep::Next)
            | (SyncingStep::ReceiveChunks, SetSyncingStep::Next)
            | (SyncingStep::SpawnEntities, SetSyncingStep::Next)
            | (SyncingStep::Ready, SetSyncingStep::Done)
            | (_, SetSyncingStep::Failed)
    )
}

/// Validates transitions between DisconnectingStep states.
pub(crate) fn is_valid_disconnecting_step_transition(
    from: &DisconnectingStep,
    to: &SetDisconnectingStep,
) -> bool {
    matches!(
        (from, to),
        (
            DisconnectingStep::SendDisconnect,
            SetDisconnectingStep::Next
        ) | (DisconnectingStep::WaitForAck, SetDisconnectingStep::Next)
            | (DisconnectingStep::Cleanup, SetDisconnectingStep::Next)
            | (DisconnectingStep::Ready, SetDisconnectingStep::Done)
            | (_, SetDisconnectingStep::Failed)
    )
}

// =============================================================================
// OBSERVER
// =============================================================================

fn on_connecting_step(
    event: On<SetConnectingStep>,
    current_parent: Res<State<ClientConnectionStatus>>,
    current: Option<Res<State<ConnectingStep>>>,
    mut next_client_status: ResMut<NextState<ClientConnectionStatus>>,
    mut next_connecting_step: Option<ResMut<NextState<ConnectingStep>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
) {
    // Validate parent state transition
    if !is_valid_client_status_connecting_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid ClientConnectionStatus transition for ConnectingStep: {:?} with parent status {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Start: Wechselt ClientConnectionStatus zu Connecting UND setzt Step auf ResolveAddress
        SetConnectingStep::Start => {
            next_client_status.set(ClientConnectionStatus::Connecting);
            if let Some(ref mut next_step) = next_connecting_step {
                next_step.set(ConnectingStep::ResolveAddress);
            }
        }
        // Next/Done/Failed: ConnectingStep muss existieren (Status muss Connecting sein)
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!(
                        "ConnectingStep does not exist - ClientConnectionStatus must be Connecting first"
                    );
                    return;
                }
            };

            // Validate step transition
            if !is_valid_connecting_step_transition(&current, event.event()) {
                warn!(
                    "Invalid ConnectingStep transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            match (current, event.event()) {
                (ConnectingStep::ResolveAddress, SetConnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_connecting_step {
                        next_step.set(ConnectingStep::OpenSocket);
                    }
                }
                (ConnectingStep::OpenSocket, SetConnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_connecting_step {
                        next_step.set(ConnectingStep::SendHandshake);
                    }
                }
                (ConnectingStep::SendHandshake, SetConnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_connecting_step {
                        next_step.set(ConnectingStep::WaitForAccept);
                    }
                }
                (ConnectingStep::WaitForAccept, SetConnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_connecting_step {
                        next_step.set(ConnectingStep::Ready);
                    }
                }
                (ConnectingStep::Ready, SetConnectingStep::Done) => {
                    next_client_status.set(ClientConnectionStatus::Connected);
                }
                (_, SetConnectingStep::Failed) => {
                    next_client_status.set(ClientConnectionStatus::Disconnected);
                    next_session_type.set(SessionType::None);
                    next_app_scope.set(AppScope::Menu);
                    // TODO: Notification Error
                }
                _ => {}
            }
        }
    }
}

fn on_syncing_step(
    event: On<SetSyncingStep>,
    current_parent: Res<State<ClientConnectionStatus>>,
    current: Option<Res<State<SyncingStep>>>,
    mut next_client_status: ResMut<NextState<ClientConnectionStatus>>,
    mut next_syncing_step: Option<ResMut<NextState<SyncingStep>>>,
    mut next_session_state: ResMut<NextState<SessionState>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
) {
    // Validate parent state transition
    if !is_valid_client_status_syncing_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid ClientConnectionStatus transition for SyncingStep: {:?} with parent status {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Start: Wechselt ClientConnectionStatus zu Syncing UND setzt Step auf RequestWorld
        SetSyncingStep::Start => {
            next_client_status.set(ClientConnectionStatus::Syncing);
            next_app_scope.set(AppScope::Session);
            if let Some(ref mut next_step) = next_syncing_step {
                next_step.set(SyncingStep::RequestWorld);
            }
        }
        // Next/Done/Failed: SyncingStep muss existieren (Status muss Syncing sein)
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!(
                        "SyncingStep does not exist - ClientConnectionStatus must be Syncing first"
                    );
                    return;
                }
            };

            // Validate step transition
            if !is_valid_syncing_step_transition(&current, event.event()) {
                warn!(
                    "Invalid SyncingStep transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            match (current, event.event()) {
                (SyncingStep::RequestWorld, SetSyncingStep::Next) => {
                    if let Some(ref mut next_step) = next_syncing_step {
                        next_step.set(SyncingStep::ReceiveChunks);
                    }
                }
                (SyncingStep::ReceiveChunks, SetSyncingStep::Next) => {
                    if let Some(ref mut next_step) = next_syncing_step {
                        next_step.set(SyncingStep::SpawnEntities);
                    }
                }
                (SyncingStep::SpawnEntities, SetSyncingStep::Next) => {
                    if let Some(ref mut next_step) = next_syncing_step {
                        next_step.set(SyncingStep::Ready);
                    }
                }
                (SyncingStep::Ready, SetSyncingStep::Done) => {
                    next_client_status.set(ClientConnectionStatus::Playing);
                    next_session_state.set(SessionState::Active);
                }
                (_, SetSyncingStep::Failed) => {
                    next_client_status.set(ClientConnectionStatus::Disconnected);
                    next_app_scope.set(AppScope::Menu);
                    // TODO: Notification Error
                }
                _ => {}
            }
        }
    }
}

fn on_set_disconnecting_step(
    event: On<SetDisconnectingStep>,
    current_parent: Res<State<ClientConnectionStatus>>,
    shutdown_state: Option<Res<State<DisconnectingStep>>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
    mut next_client_status: ResMut<NextState<ClientConnectionStatus>>,
    mut next_disconnecting_step: Option<ResMut<NextState<DisconnectingStep>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
) {
    // Validate parent state transition
    if !is_valid_client_status_disconnecting_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid ClientConnectionStatus transition for DisconnectingStep: {:?} with parent status {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Start: Wechselt ClientConnectionStatus zu Disconnecting UND setzt Step auf SendDisconnect
        SetDisconnectingStep::Start => {
            next_client_status.set(ClientConnectionStatus::Disconnecting);
            if let Some(ref mut next_step) = next_disconnecting_step {
                next_step.set(DisconnectingStep::SendDisconnect);
            }
        }
        // Next/Done/Failed: DisconnectingStep muss existieren (Status muss Disconnecting sein)
        _ => {
            let current = match shutdown_state {
                Some(c) => *c.get(),
                None => {
                    warn!(
                        "DisconnectingStep does not exist - ClientConnectionStatus must be Disconnecting first"
                    );
                    return;
                }
            };

            // Validate step transition
            if !is_valid_disconnecting_step_transition(&current, event.event()) {
                warn!(
                    "Invalid DisconnectingStep transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            match (current, event.event()) {
                (DisconnectingStep::SendDisconnect, SetDisconnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_disconnecting_step {
                        next_step.set(DisconnectingStep::WaitForAck);
                    }
                }
                (DisconnectingStep::WaitForAck, SetDisconnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_disconnecting_step {
                        next_step.set(DisconnectingStep::Cleanup);
                    }
                }
                (DisconnectingStep::Cleanup, SetDisconnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_disconnecting_step {
                        next_step.set(DisconnectingStep::Ready);
                    }
                }
                (DisconnectingStep::Ready, SetDisconnectingStep::Done) => {
                    next_app_scope.set(AppScope::Menu);
                    next_session_type.set(SessionType::None);
                }
                (_, SetDisconnectingStep::Failed) => {
                    next_app_scope.set(AppScope::Menu);
                    next_session_type.set(SessionType::None);
                    // TODO: Notification Error
                }
                _ => {}
            }
            DisconnectingStep::Cleanup => {
                next_state.set(DisconnectingStep::Ready);
            }
        }

        /// Setup für Client-Tests: SessionType ist Client, aber noch nicht verbunden
        pub fn setup_test_app_for_client() -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            #[cfg(feature = "hosted")]
            {
                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Splash);
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
