//! Client connection lifecycle and pause menu logic.
//!
//! Provides [`ClientSessionPlugin`] which registers all client-side substates and
//! observers for connection flow events. Handles the full client lifecycle from
//! initial connection through gameplay to clean disconnection, including pause menu
//! navigation and Escape key handling during active sessions.

use {
    crate::{
        events::session::{
            SetConnectingStep, SetDisconnectingStep, SetPauseMenu, SetServerShutdownStep,
            SetSyncingStep,
        },
        states::{
            app::AppScope,
            session::{
                ClientConnectionStatus, ConnectingStep, DisconnectingStep, PauseMenu, SessionState,
                SessionType, SyncingStep,
            },
        },
    },
    bevy::prelude::{
        App, AppExtStates, ButtonInput, Commands, IntoScheduleConfigs, KeyCode, NextState, On,
        Plugin, Res, ResMut, State, SystemCondition, Update, in_state, warn,
    },
};

/// Plugin that manages client connection lifecycle and pause menu.
///
/// Registers `ClientConnectionStatus`, `ConnectingStep`, `SyncingStep`, `DisconnectingStep`,
/// and `PauseMenu` substates. Sets up observers for connection flow events (`SetConnectingStep`,
/// `SetSyncingStep`, `SetDisconnectingStep`) and pause menu navigation (`SetPauseMenu`).
/// Also adds the Escape key system for toggling the pause menu during active sessions.
pub struct ClientSessionPlugin;

impl Plugin for ClientSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<ClientConnectionStatus>()
            .add_sub_state::<ConnectingStep>()
            .add_sub_state::<SyncingStep>()
            .add_sub_state::<DisconnectingStep>()
            .add_sub_state::<PauseMenu>()
            .add_observer(on_connecting_step)
            .add_observer(on_syncing_step)
            .add_observer(on_set_disconnecting_step)
            .add_observer(handle_pause_menu_nav)
            .add_systems(
                Update,
                toggle_game_menu
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
    trigger: On<SetPauseMenu>,
    mut commands: Commands,
    mut next_pause_menu: ResMut<NextState<PauseMenu>>,
    mut next_session_state: ResMut<NextState<SessionState>>,
    session_type: Res<State<SessionType>>,
) {
    match trigger.event() {
        SetPauseMenu::Resume => {
            next_session_state.set(SessionState::Active);
        }
        SetPauseMenu::Settings => {
            next_pause_menu.set(PauseMenu::Settings);
        }
        SetPauseMenu::Save => {
            next_pause_menu.set(PauseMenu::Save);
        }
        SetPauseMenu::Load => {
            next_pause_menu.set(PauseMenu::Load);
        }
        SetPauseMenu::Exit => match session_type.get() {
            SessionType::Singleplayer => {
                commands.trigger(SetServerShutdownStep::Start);
            }
            SessionType::Client => {
                commands.trigger(SetDisconnectingStep::Start);
            }
            _ => {}
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
        (ClientConnectionStatus::Connecting, SetConnectingStep::Next)
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
        (ConnectingStep::OpeningConnection, SetConnectingStep::Next)
            | (ConnectingStep::Authenticating, SetConnectingStep::Next)
            | (ConnectingStep::WaitingForAccept, SetConnectingStep::Next)
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
    current_parent: Option<Res<State<ClientConnectionStatus>>>,
    current: Option<Res<State<ConnectingStep>>>,
    mut next_client_status: Option<ResMut<NextState<ClientConnectionStatus>>>,
    mut next_connecting_step: Option<ResMut<NextState<ConnectingStep>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    mut next_app_scope: ResMut<NextState<AppScope>>,
) {
    let status = match current_parent {
        Some(ref s) => s.get(),
        None => {
            warn!(
                "SetConnectingStep fired but ClientConnectionStatus does not exist (wrong SessionType?)"
            );
            return;
        }
    };

    // Validate parent state transition
    if !is_valid_client_status_connecting_transition(status, event.event()) {
        warn!(
            "Invalid ClientConnectionStatus transition for ConnectingStep: {:?} with parent status {:?}",
            event.event(),
            status
        );
        return;
    }

    match *event.event() {
        // ConnectingStep muss existieren (Status muss Connecting sein)
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
                (ConnectingStep::OpeningConnection, SetConnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_connecting_step {
                        next_step.set(ConnectingStep::Authenticating);
                    }
                }
                (ConnectingStep::Authenticating, SetConnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_connecting_step {
                        next_step.set(ConnectingStep::WaitingForAccept);
                    }
                }
                (ConnectingStep::WaitingForAccept, SetConnectingStep::Next) => {
                    if let Some(ref mut next_step) = next_connecting_step {
                        next_step.set(ConnectingStep::Ready);
                    }
                }
                (ConnectingStep::Ready, SetConnectingStep::Done) => {
                    if let Some(ref mut s) = next_client_status {
                        s.set(ClientConnectionStatus::Connected);
                    }
                }
                (_, SetConnectingStep::Failed) => {
                    if let Some(ref mut s) = next_client_status {
                        s.set(ClientConnectionStatus::Disconnected);
                    }
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
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================
#[cfg(feature = "hosted")]
#[cfg(test)]
mod tests {
    //! Tests für die Client-Session Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Validator-Funktionen (ob Übergänge gültig/ungültig sind)
    //! 2. Observer-Logik (ob Events korrekt verarbeitet werden)
    //! 3. SubState-Übergänge (ob die Schritte korrekt durchlaufen werden)

    use crate::events::session::{SetConnectingStep, SetDisconnectingStep, SetSyncingStep};
    use crate::states::session::ClientConnectionStatus;

    mod helpers {

        use crate::{
            events::session::{SetConnectingStep, SetDisconnectingStep, SetSyncingStep},
            logic::{app::AppLogicPlugin, session::client::ClientSessionPlugin},
            states::{
                app::AppScope,
                session::{
                    ClientConnectionStatus, ConnectingStep, DisconnectingStep, SessionState,
                    SessionType, SyncingStep,
                },
            },
        };

        use bevy::{input::InputPlugin, prelude::*, state::app::StatesPlugin};

        pub const CONNECTING_STEPS: u8 = 4;
        pub const SYNCING_STEPS: u8 = 3;
        pub const DISCONNECTING_STEPS: u8 = 3;

        pub fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins((
                MinimalPlugins,
                StatesPlugin,
                InputPlugin,
                ClientSessionPlugin,
                AppLogicPlugin,
            ));

            app
        }

        /// Runs the app for one update tick.
        pub fn update_app(app: &mut App, i: u8) {
            for _ in 0..i {
                app.update();
            }
        }

        /// Setup für Client-Tests: SessionType ist Client, aber noch nicht verbunden
        pub fn setup_test_app_for_client() -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            // Splash → Menu
            app.world_mut()
                .resource_mut::<NextState<AppScope>>()
                .set(AppScope::Menu);
            update_app(&mut app, 1);

            assert_eq!(
                app.world().resource::<State<AppScope>>().get(),
                &AppScope::Menu
            );

            // Set SessionType::Client before transitioning to Session
            // (ClientConnectionStatus is a SubState of SessionType::Client)
            app.world_mut()
                .resource_mut::<NextState<SessionType>>()
                .set(SessionType::Client);
            app.world_mut()
                .resource_mut::<NextState<AppScope>>()
                .set(AppScope::Session);
            update_app(&mut app, 1);

            assert_eq!(
                app.world().resource::<State<SessionType>>().get(),
                &SessionType::Client
            );
            assert_eq!(
                app.world()
                    .resource::<State<ClientConnectionStatus>>()
                    .get(),
                &ClientConnectionStatus::Disconnected
            );

            app
        }

        /// Startet den Verbindungsprozess
        pub fn start_connecting(app: &mut App) {
            // Set Connecting status directly (like SetJoinGame::Confirm does)
            app.world_mut()
                .resource_mut::<NextState<ClientConnectionStatus>>()
                .set(ClientConnectionStatus::Connecting);
            update_app(app, 1);

            let client_status = app.world().resource::<State<ClientConnectionStatus>>();
            assert_eq!(client_status.get(), &ClientConnectionStatus::Connecting);

            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::Client);

            let connecting_step = app.world().resource::<State<ConnectingStep>>();
            assert_eq!(connecting_step.get(), &ConnectingStep::OpeningConnection);
        }

        /// Führt den Verbindungsprozess fort
        pub fn connecting_next_step(app: &mut App, i: u8) {
            for _ in 0..i {
                app.world_mut().trigger(SetConnectingStep::Next);
                update_app(app, 1);
            }

            {
                let step = app.world().resource::<State<ConnectingStep>>();
                assert_eq!(step.get(), &ConnectingStep::Ready);

                let state = app.world().resource::<State<ClientConnectionStatus>>();
                assert_eq!(state.get(), &ClientConnectionStatus::Connecting);
            }

            app.world_mut().trigger(SetConnectingStep::Done);
            update_app(app, 1);

            {
                let state_after_done = app.world().resource::<State<ClientConnectionStatus>>();
                assert_eq!(state_after_done.get(), &ClientConnectionStatus::Connected);
            }
        }

        /// Startet den Sync-Prozess
        pub fn start_syncing(app: &mut App) {
            app.world_mut().trigger(SetSyncingStep::Start);
            update_app(app, 1);

            let client_status = app.world().resource::<State<ClientConnectionStatus>>();
            assert_eq!(client_status.get(), &ClientConnectionStatus::Syncing);

            #[cfg(feature = "hosted")]
            {
                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Session);
            }

            let syncing_step = app.world().resource::<State<SyncingStep>>();
            assert_eq!(syncing_step.get(), &SyncingStep::RequestWorld);
        }

        /// Führt den Sync-Prozess fort
        pub fn syncing_next_step(app: &mut App, i: u8) {
            for _ in 0..i {
                app.world_mut().trigger(SetSyncingStep::Next);
                update_app(app, 1);
            }

            {
                let step = app.world().resource::<State<SyncingStep>>();
                assert_eq!(step.get(), &SyncingStep::Ready);
                let state = app.world().resource::<State<ClientConnectionStatus>>();
                assert_eq!(state.get(), &ClientConnectionStatus::Syncing);
            }

            app.world_mut().trigger(SetSyncingStep::Done);
            update_app(app, 1);

            {
                let state_after_done = app.world().resource::<State<ClientConnectionStatus>>();
                assert_eq!(state_after_done.get(), &ClientConnectionStatus::Playing);

                let session_state = app.world().resource::<State<SessionState>>();
                assert_eq!(session_state.get(), &SessionState::Active);
            }
        }

        /// Startet den Disconnect-Prozess
        pub fn start_disconnecting(app: &mut App) {
            app.world_mut().trigger(SetDisconnectingStep::Start);
            update_app(app, 1);

            let client_status = app.world().resource::<State<ClientConnectionStatus>>();
            assert_eq!(client_status.get(), &ClientConnectionStatus::Disconnecting);

            let disconnecting_step = app.world().resource::<State<DisconnectingStep>>();
            assert_eq!(disconnecting_step.get(), &DisconnectingStep::SendDisconnect);
        }

        /// Führt den Disconnect-Prozess fort
        pub fn disconnecting_next_step(app: &mut App, i: u8) {
            for _ in 0..i {
                app.world_mut().trigger(SetDisconnectingStep::Next);
                update_app(app, 1);
            }

            {
                let step = app.world().resource::<State<DisconnectingStep>>();
                assert_eq!(step.get(), &DisconnectingStep::Ready);
                let state = app.world().resource::<State<ClientConnectionStatus>>();
                assert_eq!(state.get(), &ClientConnectionStatus::Disconnecting);
            }

            app.world_mut().trigger(SetDisconnectingStep::Done);
            update_app(app, 1);

            {
                let session_type = app.world().resource::<State<SessionType>>();
                assert_eq!(session_type.get(), &SessionType::None);

                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Menu);
            }
        }

        /// Verbindungsprozess an einem bestimmten Step fehlschlagen lassen
        pub fn connecting_fail_at_step(app: &mut App, fail_at_step: u8) {
            println!("Connecting Failure Step: {:?}", fail_at_step);

            for _ in 0..fail_at_step {
                app.world_mut().trigger(SetConnectingStep::Next);
                update_app(app, 1);
            }

            app.world_mut().trigger(SetConnectingStep::Failed);
            update_app(app, 1);

            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::None);

            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Menu);
        }

        /// Sync-Prozess an einem bestimmten Step fehlschlagen lassen
        pub fn syncing_fail_at_step(app: &mut App, fail_at_step: u8) {
            println!("Syncing Failure Step: {:?}", fail_at_step);

            for _ in 0..fail_at_step {
                app.world_mut().trigger(SetSyncingStep::Next);
                update_app(app, 1);
            }

            app.world_mut().trigger(SetSyncingStep::Failed);
            update_app(app, 1);

            let client_status = app.world().resource::<State<ClientConnectionStatus>>();
            assert_eq!(client_status.get(), &ClientConnectionStatus::Disconnected);
        }

        /// Disconnect-Prozess an einem bestimmten Step fehlschlagen lassen
        pub fn disconnecting_fail_at_step(app: &mut App, fail_at_step: u8) {
            println!("Disconnecting Failure Step: {:?}", fail_at_step);

            for _ in 0..fail_at_step {
                app.world_mut().trigger(SetDisconnectingStep::Next);
                update_app(app, 1);
            }

            app.world_mut().trigger(SetDisconnectingStep::Failed);
            update_app(app, 1);

            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::None);

            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Menu);
        }
    }

    // =============================================================================
    // TESTS FÜR CONNECTING STEP
    // =============================================================================

    mod connecting_step_tests {

        use super::*;

        /// Test: Verbindung erfolgreich herstellen
        #[test]
        fn test_connecting_success() {
            let mut app = helpers::setup_test_app_for_client();
            helpers::start_connecting(&mut app);
            helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);
        }

        /// Test: Verbindung kann an verschiedenen Steps fehlschlagen.
        #[test]
        fn test_connecting_failure() {
            for step in 0..helpers::CONNECTING_STEPS {
                let mut app = helpers::setup_test_app_for_client();
                helpers::start_connecting(&mut app);
                helpers::connecting_fail_at_step(&mut app, step);
            }
        }
    }

    // =============================================================================
    // TESTS FÜR SYNCING STEP
    // =============================================================================

    mod syncing_step_tests {

        use super::*;

        /// Test: SyncingStep::Start wechselt zu RequestWorld.
        #[test]
        fn test_syncing_start() {
            let mut app = helpers::setup_test_app_for_client();
            helpers::start_connecting(&mut app);
            helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);
            helpers::start_syncing(&mut app);
        }

        /// Test: Sync erfolgreich abgeschlossen
        #[test]
        fn test_syncing_success() {
            let mut app = helpers::setup_test_app_for_client();
            helpers::start_connecting(&mut app);
            helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);
            helpers::start_syncing(&mut app);
            helpers::syncing_next_step(&mut app, helpers::SYNCING_STEPS);
        }

        /// Test: Sync kann an verschiedenen Steps fehlschlagen.
        #[test]
        fn test_syncing_failure() {
            for step in 0..helpers::SYNCING_STEPS {
                let mut app = helpers::setup_test_app_for_client();
                helpers::start_connecting(&mut app);
                helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);
                helpers::start_syncing(&mut app);
                helpers::syncing_fail_at_step(&mut app, step);
            }
        }
    }

    // =============================================================================
    // TESTS FÜR DISCONNECTING STEP
    // =============================================================================

    mod disconnecting_step_tests {

        use super::*;

        /// Test: DisconnectingStep::Start wechselt zu SendDisconnect.
        #[test]
        fn test_disconnecting_start() {
            let mut app = helpers::setup_test_app_for_client();
            helpers::start_connecting(&mut app);
            helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);
            helpers::start_syncing(&mut app);
            helpers::syncing_next_step(&mut app, helpers::SYNCING_STEPS);
            helpers::start_disconnecting(&mut app);
        }

        /// Test: Disconnect erfolgreich abgeschlossen
        #[test]
        fn test_disconnecting_success() {
            let mut app = helpers::setup_test_app_for_client();
            helpers::start_connecting(&mut app);
            helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);
            helpers::start_syncing(&mut app);
            helpers::syncing_next_step(&mut app, helpers::SYNCING_STEPS);
            helpers::start_disconnecting(&mut app);
            helpers::disconnecting_next_step(&mut app, helpers::DISCONNECTING_STEPS);
        }

        /// Test: Disconnect kann an verschiedenen Steps fehlschlagen.
        #[test]
        fn test_disconnecting_failure() {
            for step in 0..helpers::DISCONNECTING_STEPS {
                let mut app = helpers::setup_test_app_for_client();
                helpers::start_connecting(&mut app);
                helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);
                helpers::start_syncing(&mut app);
                helpers::syncing_next_step(&mut app, helpers::SYNCING_STEPS);
                helpers::start_disconnecting(&mut app);
                helpers::disconnecting_fail_at_step(&mut app, step);
            }
        }

        /// Test: Unerwarteter Server-Disconnect aus Playing heraus (z.B. Server shutdown).
        /// Playing -> SetDisconnectingStep::Start -> Disconnecting -> ... -> Menu
        #[test]
        fn test_unexpected_disconnect_from_playing() {
            use crate::states::{app::AppScope, session::SessionType};
            use bevy::prelude::State;
            let mut app = helpers::setup_test_app_for_client();

            // Zum Playing-State navigieren
            helpers::start_connecting(&mut app);
            helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);
            helpers::start_syncing(&mut app);
            helpers::syncing_next_step(&mut app, helpers::SYNCING_STEPS);

            {
                let status = app.world().resource::<State<ClientConnectionStatus>>();
                assert_eq!(status.get(), &ClientConnectionStatus::Playing);
            }

            // Server-seitiger Disconnect: direkt Start triggern (kein vorheriges Start_disconnecting)
            helpers::start_disconnecting(&mut app);
            helpers::disconnecting_next_step(&mut app, helpers::DISCONNECTING_STEPS);

            // Ergebnis: zurück im Menü
            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Menu);

            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::None);
        }
    }

    // =============================================================================
    // INTEGRATIONSTEST: KOMPLETTER CLIENT LEBENSZYKLUS
    // =============================================================================

    mod integration_tests {

        use super::*;

        /// Test: Kompletter Client-Lebenszyklus.
        /// Disconnected -> Connecting -> Connected -> Syncing -> Playing -> Disconnecting -> Disconnected
        #[test]
        fn test_full_client_lifecycle() {
            let mut app = helpers::setup_test_app_for_client();

            // Connecting
            helpers::start_connecting(&mut app);
            helpers::connecting_next_step(&mut app, helpers::CONNECTING_STEPS);

            // Syncing
            helpers::start_syncing(&mut app);
            helpers::syncing_next_step(&mut app, helpers::SYNCING_STEPS);

            // Disconnecting
            helpers::start_disconnecting(&mut app);
            helpers::disconnecting_next_step(&mut app, helpers::DISCONNECTING_STEPS);
        }
    }

    // =============================================================================
    // TESTS FÜR VALIDATOR-FUNKTIONEN
    // =============================================================================

    mod validator_tests {

        use super::*;
        use crate::states::session::{ConnectingStep, DisconnectingStep, SyncingStep};

        // Importiere alle Validator-Funktionen
        use super::super::is_valid_client_status_connecting_transition;
        use super::super::is_valid_client_status_disconnecting_transition;
        use super::super::is_valid_client_status_syncing_transition;
        use super::super::is_valid_connecting_step_transition;
        use super::super::is_valid_disconnecting_step_transition;
        use super::super::is_valid_syncing_step_transition;

        /// Test: Gültige ClientConnectionStatus-Connecting-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_client_status_connecting_transitions() {
            // Connecting → Next ist gültig (zum nächsten Step)
            assert!(is_valid_client_status_connecting_transition(
                &ClientConnectionStatus::Connecting,
                &SetConnectingStep::Next
            ));

            // Connecting → Done ist gültig (Verbindung hergestellt)
            assert!(is_valid_client_status_connecting_transition(
                &ClientConnectionStatus::Connecting,
                &SetConnectingStep::Done
            ));
        }

        /// Test: Ungültige ClientConnectionStatus-Connecting-Übergänge werden blockiert.
        #[test]
        fn test_invalid_client_status_connecting_transitions() {
            // Disconnected → Next ist ungültig (muss erst Connecting werden)
            assert!(!is_valid_client_status_connecting_transition(
                &ClientConnectionStatus::Disconnected,
                &SetConnectingStep::Next
            ));

            // Disconnected → Done ist ungültig (muss erst Connecting werden)
            assert!(!is_valid_client_status_connecting_transition(
                &ClientConnectionStatus::Disconnected,
                &SetConnectingStep::Done
            ));

            // Connected → Next ist ungültig (bereits verbunden)
            assert!(!is_valid_client_status_connecting_transition(
                &ClientConnectionStatus::Connected,
                &SetConnectingStep::Next
            ));

            // Connected → Done ist ungültig (bereits verbunden)
            assert!(!is_valid_client_status_connecting_transition(
                &ClientConnectionStatus::Connected,
                &SetConnectingStep::Done
            ));

            // Playing → Next ist ungültig (muss erst disconnecten)
            assert!(!is_valid_client_status_connecting_transition(
                &ClientConnectionStatus::Playing,
                &SetConnectingStep::Next
            ));

            // Playing → Done ist ungültig (muss erst disconnecten)
            assert!(!is_valid_client_status_connecting_transition(
                &ClientConnectionStatus::Playing,
                &SetConnectingStep::Done
            ));
        }

        /// Test: Gültige ClientConnectionStatus-Syncing-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_client_status_syncing_transitions() {
            // Connected → Start ist gültig (Sync starten)
            assert!(is_valid_client_status_syncing_transition(
                &ClientConnectionStatus::Connected,
                &SetSyncingStep::Start
            ));
        }

        /// Test: Ungültige ClientConnectionStatus-Syncing-Übergänge werden blockiert.
        #[test]
        fn test_invalid_client_status_syncing_transitions() {
            // Disconnected → Start ist ungültig (nicht verbunden)
            assert!(!is_valid_client_status_syncing_transition(
                &ClientConnectionStatus::Disconnected,
                &SetSyncingStep::Start
            ));

            // Playing → Start ist ungültig (bereits am Spielen)
            assert!(!is_valid_client_status_syncing_transition(
                &ClientConnectionStatus::Playing,
                &SetSyncingStep::Start
            ));
        }

        /// Test: Gültige ClientConnectionStatus-Disconnecting-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_client_status_disconnecting_transitions() {
            // Playing → Start ist gültig (Disconnect starten)
            assert!(is_valid_client_status_disconnecting_transition(
                &ClientConnectionStatus::Playing,
                &SetDisconnectingStep::Start
            ));

            // Connecting → Start ist gültig (Verbindung abbrechen)
            assert!(is_valid_client_status_disconnecting_transition(
                &ClientConnectionStatus::Connecting,
                &SetDisconnectingStep::Start
            ));
        }

        /// Test: Gültige ConnectingStep-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_connecting_step_transitions() {
            // OpeningConnection → Next ist gültig
            assert!(is_valid_connecting_step_transition(
                &ConnectingStep::OpeningConnection,
                &SetConnectingStep::Next
            ));

            // Ready → Done ist gültig
            assert!(is_valid_connecting_step_transition(
                &ConnectingStep::Ready,
                &SetConnectingStep::Done
            ));

            // Jeder Step → Failed ist gültig
            assert!(is_valid_connecting_step_transition(
                &ConnectingStep::OpeningConnection,
                &SetConnectingStep::Failed
            ));
        }

        /// Test: Gültige SyncingStep-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_syncing_step_transitions() {
            // RequestWorld → Next ist gültig
            assert!(is_valid_syncing_step_transition(
                &SyncingStep::RequestWorld,
                &SetSyncingStep::Next
            ));

            // Ready → Done ist gültig
            assert!(is_valid_syncing_step_transition(
                &SyncingStep::Ready,
                &SetSyncingStep::Done
            ));
        }

        /// Test: Gültige DisconnectingStep-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_disconnecting_step_transitions() {
            // SendDisconnect → Next ist gültig
            assert!(is_valid_disconnecting_step_transition(
                &DisconnectingStep::SendDisconnect,
                &SetDisconnectingStep::Next
            ));

            // Ready → Done ist gültig
            assert!(is_valid_disconnecting_step_transition(
                &DisconnectingStep::Ready,
                &SetDisconnectingStep::Done
            ));
        }

        /// Test: Ungültige ClientConnectionStatus-Disconnecting-Übergänge werden blockiert.
        #[test]
        fn test_invalid_client_status_disconnecting_transitions() {
            // Disconnected → Start ist ungültig (Client ist bereits getrennt)
            assert!(!is_valid_client_status_disconnecting_transition(
                &ClientConnectionStatus::Disconnected,
                &SetDisconnectingStep::Start
            ));

            // Syncing → Start ist ungültig (Client synchronisiert gerade)
            assert!(!is_valid_client_status_disconnecting_transition(
                &ClientConnectionStatus::Syncing,
                &SetDisconnectingStep::Start
            ));

            // Disconnecting → Start ist ungültig (Client trennt bereits)
            assert!(!is_valid_client_status_disconnecting_transition(
                &ClientConnectionStatus::Disconnecting,
                &SetDisconnectingStep::Start
            ));
        }

        /// Test: Ungültige ConnectingStep-Übergänge werden blockiert.
        #[test]
        fn test_invalid_connecting_step_transitions() {
            // Ready → Next ist ungültig (Verbindung ist bereits bereit)
            assert!(!is_valid_connecting_step_transition(
                &ConnectingStep::Ready,
                &SetConnectingStep::Next
            ));

            // OpeningConnection → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_connecting_step_transition(
                &ConnectingStep::OpeningConnection,
                &SetConnectingStep::Done
            ));

            // Authenticating → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_connecting_step_transition(
                &ConnectingStep::Authenticating,
                &SetConnectingStep::Done
            ));

            // WaitingForAccept → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_connecting_step_transition(
                &ConnectingStep::WaitingForAccept,
                &SetConnectingStep::Done
            ));
        }

        /// Test: Ungültige SyncingStep-Übergänge werden blockiert.
        #[test]
        fn test_invalid_syncing_step_transitions() {
            // Ready → Next ist ungültig (Synchronisation ist bereits bereit)
            assert!(!is_valid_syncing_step_transition(
                &SyncingStep::Ready,
                &SetSyncingStep::Next
            ));

            // RequestWorld → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_syncing_step_transition(
                &SyncingStep::RequestWorld,
                &SetSyncingStep::Done
            ));

            // ReceiveChunks → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_syncing_step_transition(
                &SyncingStep::ReceiveChunks,
                &SetSyncingStep::Done
            ));

            // SpawnEntities → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_syncing_step_transition(
                &SyncingStep::SpawnEntities,
                &SetSyncingStep::Done
            ));
        }

        /// Test: Ungültige DisconnectingStep-Übergänge werden blockiert.
        #[test]
        fn test_invalid_disconnecting_step_transitions() {
            // Ready → Next ist ungültig (Trennung ist bereits bereit)
            assert!(!is_valid_disconnecting_step_transition(
                &DisconnectingStep::Ready,
                &SetDisconnectingStep::Next
            ));

            // SendDisconnect → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_disconnecting_step_transition(
                &DisconnectingStep::SendDisconnect,
                &SetDisconnectingStep::Done
            ));

            // WaitForAck → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_disconnecting_step_transition(
                &DisconnectingStep::WaitForAck,
                &SetDisconnectingStep::Done
            ));

            // Cleanup → Done ist ungültig (Noch nicht bereit)
            assert!(!is_valid_disconnecting_step_transition(
                &DisconnectingStep::Cleanup,
                &SetDisconnectingStep::Done
            ));
        }
    }
}
