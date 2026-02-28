#[cfg(feature = "hosted")]
use crate::states::app::AppScope;

#[cfg(feature = "headless")]
use bevy::{app::AppExit, prelude::MessageWriter};

use {
    crate::{
        events::session::{
            SetGoingPrivateStep, SetGoingPublicStep, SetServerShutdownStep, SetServerStartupStep,
        },
        states::session::{
            GoingPrivateStep, GoingPublicStep, PhysicsSimulation, ServerShutdownStep,
            ServerStartupStep, ServerStatus, ServerVisibility, SessionState, SessionType,
        },
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State, warn},
};

pub struct ServerSessionPlugin;

impl Plugin for ServerSessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SessionType>()
            .add_sub_state::<SessionState>()
            .add_sub_state::<ServerVisibility>()
            .add_sub_state::<ServerStatus>()
            .add_sub_state::<ServerStartupStep>()
            .add_sub_state::<ServerShutdownStep>()
            .add_sub_state::<GoingPublicStep>()
            .add_sub_state::<GoingPrivateStep>()
            .add_computed_state::<PhysicsSimulation>()
            .add_observer(on_server_startup_step)
            .add_observer(on_server_shutdown_step)
            .add_observer(on_going_public_step)
            .add_observer(on_going_private_step);
    }
}

/// Validates transitions for going public.
pub(crate) fn is_valid_server_visibility_public_transition(
    from: &ServerVisibility,
    to: &SetGoingPublicStep,
) -> bool {
    matches!(
        (from, to),
        (ServerVisibility::Private, SetGoingPublicStep::Start)
            | (ServerVisibility::GoingPublic, SetGoingPublicStep::Next)
            | (ServerVisibility::GoingPublic, SetGoingPublicStep::Done)
            | (ServerVisibility::GoingPublic, SetGoingPublicStep::Failed)
    )
}

/// Validates transitions for going private.
pub(crate) fn is_valid_server_visibility_private_transition(
    from: &ServerVisibility,
    to: &SetGoingPrivateStep,
) -> bool {
    matches!(
        (from, to),
        (ServerVisibility::Public, SetGoingPrivateStep::Start)
            | (ServerVisibility::GoingPrivate, SetGoingPrivateStep::Next)
            | (ServerVisibility::GoingPrivate, SetGoingPrivateStep::Done)
            | (ServerVisibility::GoingPrivate, SetGoingPrivateStep::Failed)
    )
}

/// Validates transitions between GoingPublicStep states.
pub(crate) fn is_valid_going_public_step_transition(
    from: &GoingPublicStep,
    to: &SetGoingPublicStep,
) -> bool {
    matches!(
        (from, to),
        (GoingPublicStep::Validating, SetGoingPublicStep::Next)
            | (GoingPublicStep::StartingServer, SetGoingPublicStep::Next)
            | (GoingPublicStep::StartingDiscovery, SetGoingPublicStep::Next)
            | (GoingPublicStep::Ready, SetGoingPublicStep::Done)
            | (_, SetGoingPublicStep::Failed)
    )
}

/// Validates transitions between GoingPrivateStep states.
pub(crate) fn is_valid_going_private_step_transition(
    from: &GoingPrivateStep,
    to: &SetGoingPrivateStep,
) -> bool {
    matches!(
        (from, to),
        (
            GoingPrivateStep::DisconnectingClients,
            SetGoingPrivateStep::Next
        ) | (GoingPrivateStep::ClosingServer, SetGoingPrivateStep::Next)
            | (GoingPrivateStep::Cleanup, SetGoingPrivateStep::Next)
            | (GoingPrivateStep::Ready, SetGoingPrivateStep::Done)
            | (_, SetGoingPrivateStep::Failed)
    )
}

pub(crate) fn is_valid_server_status_startup_transition(
    from: &ServerStatus,
    to: &SetServerStartupStep,
) -> bool {
    matches!(
        (from, to),
        (ServerStatus::Offline, SetServerStartupStep::Start)
            | (ServerStatus::Starting, SetServerStartupStep::Next)
            | (ServerStatus::Starting, SetServerStartupStep::Failed)
            | (ServerStatus::Starting, SetServerStartupStep::Done)
    )
}

/// Validates transitions between ServerStartupStep states.
pub(crate) fn is_valid_startup_step_transition(
    from: &ServerStartupStep,
    to: &SetServerStartupStep,
) -> bool {
    matches!(
        (from, to),
        (ServerStartupStep::Init, SetServerStartupStep::Next)
            | (ServerStartupStep::LoadWorld, SetServerStartupStep::Next)
            | (ServerStartupStep::SpawnEntities, SetServerStartupStep::Next)
            | (ServerStartupStep::Ready, SetServerStartupStep::Done)
            | (_, SetServerStartupStep::Failed)
    )
}

fn on_server_startup_step(
    event: On<SetServerStartupStep>,
    current_parent: Res<State<ServerStatus>>,
    current: Option<Res<State<ServerStartupStep>>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_startup_step: Option<ResMut<NextState<ServerStartupStep>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    #[cfg(feature = "hosted")] mut next_app_scope: ResMut<NextState<AppScope>>,
    #[cfg(feature = "headless")] mut exit_writer: MessageWriter<AppExit>,
) {
    // Validate parent state transition
    if !is_valid_server_status_startup_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid ServerStatus transition for ServerStartupStep: {:?} with parent status {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Start: Wechselt ServerStatus zu Starting UND setzt Step auf Init
        // Hier existiert noch kein ServerStartupStep (erst wenn Status = Starting)
        SetServerStartupStep::Start => {
            next_server_status.set(ServerStatus::Starting);
            if let Some(ref mut next_step) = next_startup_step {
                next_step.set(ServerStartupStep::Init);
            }
        }
        // Next/Done/Failed: ServerStartupStep muss existieren (Status muss Starting sein)
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!("ServerStartupStep does not exist - ServerStatus must be Starting first");
                    return;
                }
            };

            // Validate step transition
            if !is_valid_startup_step_transition(&current, event.event()) {
                warn!(
                    "Invalid ServerStartupStep transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            match (current, event.event()) {
                (ServerStartupStep::Init, SetServerStartupStep::Next) => {
                    if let Some(ref mut next_step) = next_startup_step {
                        next_step.set(ServerStartupStep::LoadWorld);
                    }
                }
                (ServerStartupStep::LoadWorld, SetServerStartupStep::Next) => {
                    if let Some(ref mut next_step) = next_startup_step {
                        next_step.set(ServerStartupStep::SpawnEntities);
                    }
                }
                (ServerStartupStep::SpawnEntities, SetServerStartupStep::Next) => {
                    if let Some(ref mut next_step) = next_startup_step {
                        next_step.set(ServerStartupStep::Ready);
                    }
                }
                (ServerStartupStep::Ready, SetServerStartupStep::Done) => {
                    next_server_status.set(ServerStatus::Running);
                }
                (_, SetServerStartupStep::Failed) => {
                    next_server_status.set(ServerStatus::Offline);

                    #[cfg(feature = "hosted")]
                    {
                        next_session_type.set(SessionType::None);
                        next_app_scope.set(AppScope::Menu);
                        // TODO: Notification Error
                    }
                    #[cfg(feature = "headless")]
                    {
                        next_session_type.set(SessionType::None);
                        exit_writer.write(AppExit::error());
                        // TODO: Log Error
                        // TODO: Proper Error Code in AppExit
                    }
                }
                _ => {}
            }
        }
    }
}

pub(crate) fn is_valid_server_status_shutdown_transition(
    from: &ServerStatus,
    to: &SetServerShutdownStep,
) -> bool {
    matches!(
        (from, to),
        (ServerStatus::Running, SetServerShutdownStep::Start)
            | (ServerStatus::Stopping, SetServerShutdownStep::Next)
            | (ServerStatus::Stopping, SetServerShutdownStep::Done)
            | (ServerStatus::Stopping, SetServerShutdownStep::Failed)
    )
}

/// Validates transitions between ServerShutdownStep states.
pub(crate) fn is_valid_shutdown_step_transition(
    from: &ServerShutdownStep,
    to: &SetServerShutdownStep,
) -> bool {
    #[cfg(feature = "hosted")]
    {
        matches!(
            (from, to),
            (ServerShutdownStep::SaveWorld, SetServerShutdownStep::Next)
                | (
                    ServerShutdownStep::DisconnectClients,
                    SetServerShutdownStep::Next
                )
                | (
                    ServerShutdownStep::DespawnLocalClient,
                    SetServerShutdownStep::Next
                )
                | (ServerShutdownStep::Cleanup, SetServerShutdownStep::Next)
                | (ServerShutdownStep::Ready, SetServerShutdownStep::Done)
                | (_, SetServerShutdownStep::Failed)
        )
    }

    #[cfg(feature = "headless")]
    {
        matches!(
            (from, to),
            (ServerShutdownStep::SaveWorld, SetServerShutdownStep::Next)
                | (
                    ServerShutdownStep::DisconnectClients,
                    SetServerShutdownStep::Next
                )
                | (ServerShutdownStep::Cleanup, SetServerShutdownStep::Next)
                | (ServerShutdownStep::Ready, SetServerShutdownStep::Done)
                | (_, SetServerShutdownStep::Failed)
        )
    }
}

fn on_server_shutdown_step(
    event: On<SetServerShutdownStep>,
    current_parent: Res<State<ServerStatus>>,
    current: Option<Res<State<ServerShutdownStep>>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
    mut next_shutdown_step: Option<ResMut<NextState<ServerShutdownStep>>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
    #[cfg(feature = "hosted")] mut next_app_scope: ResMut<NextState<AppScope>>,
    #[cfg(feature = "headless")] mut exit_writer: MessageWriter<AppExit>,
) {
    // Validate parent state transition
    if !is_valid_server_status_shutdown_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid ServerStatus transition for ServerShutdownStep: {:?} with parent status {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Start: Wechselt ServerStatus zu Stopping UND setzt Step auf SaveWorld
        // Hier existiert noch kein ServerShutdownStep (erst wenn Status = Stopping)
        SetServerShutdownStep::Start => {
            next_server_status.set(ServerStatus::Stopping);
            if let Some(ref mut next_step) = next_shutdown_step {
                next_step.set(ServerShutdownStep::SaveWorld);
            }
        }
        // Next/Done/Failed: ServerShutdownStep muss existieren (Status muss Stopping sein)
        _ => {
            let current = match current {
                Some(c) => *c.get(), // Kopieren statt Referenz behalten
                None => {
                    warn!(
                        "ServerShutdownStep does not exist - ServerStatus must be Stopping first"
                    );
                    return;
                }
            };

            // Validate step transition
            if !is_valid_shutdown_step_transition(&current, event.event()) {
                warn!(
                    "Invalid ServerShutdownStep transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            match (current, event.event()) {
                (ServerShutdownStep::SaveWorld, SetServerShutdownStep::Next) => {
                    if let Some(ref mut next_step) = next_shutdown_step {
                        next_step.set(ServerShutdownStep::DisconnectClients);
                    }
                }
                (ServerShutdownStep::DisconnectClients, SetServerShutdownStep::Next) => {
                    if let Some(ref mut next_step) = next_shutdown_step {
                        #[cfg(feature = "hosted")]
                        next_step.set(ServerShutdownStep::DespawnLocalClient);
                        #[cfg(feature = "headless")]
                        next_step.set(ServerShutdownStep::Cleanup);
                    }
                }
                #[cfg(feature = "hosted")]
                (ServerShutdownStep::DespawnLocalClient, SetServerShutdownStep::Next) => {
                    if let Some(ref mut next_step) = next_shutdown_step {
                        next_step.set(ServerShutdownStep::Cleanup);
                    }
                }
                (ServerShutdownStep::Cleanup, SetServerShutdownStep::Next) => {
                    if let Some(ref mut next_step) = next_shutdown_step {
                        next_step.set(ServerShutdownStep::Ready);
                    }
                }
                (ServerShutdownStep::Ready, SetServerShutdownStep::Done) => {
                    #[cfg(feature = "hosted")]
                    {
                        next_session_type.set(SessionType::None);
                        next_app_scope.set(AppScope::Menu);
                    }

                    #[cfg(feature = "headless")]
                    {
                        next_session_type.set(SessionType::None);
                        exit_writer.write(AppExit::Success);
                    }
                }
                (_, SetServerShutdownStep::Failed) => {
                    next_server_status.set(ServerStatus::Offline);

                    #[cfg(feature = "hosted")]
                    {
                        next_session_type.set(SessionType::None);
                        next_app_scope.set(AppScope::Menu);
                        // TODO: Notification Error
                    }
                    #[cfg(feature = "headless")]
                    {
                        next_session_type.set(SessionType::None);
                        exit_writer.write(AppExit::error());
                        // TODO: Log Error
                        // TODO: Proper Error Code in AppExit
                    }
                }
                _ => {}
            }
        }
    }
}

fn on_going_public_step(
    event: On<SetGoingPublicStep>,
    current_parent: Res<State<ServerVisibility>>,
    current: Option<Res<State<GoingPublicStep>>>,
    mut next_visibility: ResMut<NextState<ServerVisibility>>,
    mut next_public_step: Option<ResMut<NextState<GoingPublicStep>>>,
) {
    // Validate parent state transition
    if !is_valid_server_visibility_public_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid ServerVisibility transition for GoingPublicStep: {:?} with parent visibility {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Start: Wechselt ServerVisibility zu GoingPublic UND setzt Step auf Validating
        SetGoingPublicStep::Start => {
            next_visibility.set(ServerVisibility::GoingPublic);
            if let Some(ref mut next_step) = next_public_step {
                next_step.set(GoingPublicStep::Validating);
            }
        }
        // Next/Done/Failed: GoingPublicStep muss existieren
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!(
                        "GoingPublicStep does not exist - ServerVisibility must be GoingPublic first"
                    );
                    return;
                }
            };

            // Validate step transition
            if !is_valid_going_public_step_transition(&current, event.event()) {
                warn!(
                    "Invalid GoingPublicStep transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            match (current, event.event()) {
                (GoingPublicStep::Validating, SetGoingPublicStep::Next) => {
                    if let Some(ref mut next_step) = next_public_step {
                        next_step.set(GoingPublicStep::StartingServer);
                    }
                }
                (GoingPublicStep::StartingServer, SetGoingPublicStep::Next) => {
                    if let Some(ref mut next_step) = next_public_step {
                        next_step.set(GoingPublicStep::StartingDiscovery);
                    }
                }
                (GoingPublicStep::StartingDiscovery, SetGoingPublicStep::Next) => {
                    if let Some(ref mut next_step) = next_public_step {
                        next_step.set(GoingPublicStep::Ready);
                    }
                }
                (GoingPublicStep::Ready, SetGoingPublicStep::Done) => {
                    next_visibility.set(ServerVisibility::Public);
                }
                (_, SetGoingPublicStep::Failed) => {
                    next_visibility.set(ServerVisibility::Private);
                }
                _ => {}
            }
        }
    }
}

fn on_going_private_step(
    event: On<SetGoingPrivateStep>,
    current_parent: Res<State<ServerVisibility>>,
    current: Option<Res<State<GoingPrivateStep>>>,
    mut next_visibility: ResMut<NextState<ServerVisibility>>,
    mut next_private_step: Option<ResMut<NextState<GoingPrivateStep>>>,
) {
    // Validate parent state transition
    if !is_valid_server_visibility_private_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid ServerVisibility transition for GoingPrivateStep: {:?} with parent visibility {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Start: Wechselt ServerVisibility zu GoingPrivate UND setzt Step auf DisconnectingClients
        SetGoingPrivateStep::Start => {
            next_visibility.set(ServerVisibility::GoingPrivate);
            if let Some(ref mut next_step) = next_private_step {
                next_step.set(GoingPrivateStep::DisconnectingClients);
            }
        }
        // Next/Done/Failed: GoingPrivateStep muss existieren
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!(
                        "GoingPrivateStep does not exist - ServerVisibility must be GoingPrivate first"
                    );
                    return;
                }
            };

            // Validate step transition
            if !is_valid_going_private_step_transition(&current, event.event()) {
                warn!(
                    "Invalid GoingPrivateStep transition: {:?} -> {:?}",
                    current,
                    event.event()
                );
                return;
            }

            match (current, event.event()) {
                (GoingPrivateStep::DisconnectingClients, SetGoingPrivateStep::Next) => {
                    if let Some(ref mut next_step) = next_private_step {
                        next_step.set(GoingPrivateStep::ClosingServer);
                    }
                }
                (GoingPrivateStep::ClosingServer, SetGoingPrivateStep::Next) => {
                    if let Some(ref mut next_step) = next_private_step {
                        next_step.set(GoingPrivateStep::Cleanup);
                    }
                }
                (GoingPrivateStep::Cleanup, SetGoingPrivateStep::Next) => {
                    if let Some(ref mut next_step) = next_private_step {
                        next_step.set(GoingPrivateStep::Ready);
                    }
                }
                (GoingPrivateStep::Ready, SetGoingPrivateStep::Done) => {
                    next_visibility.set(ServerVisibility::Private);
                }
                (_, SetGoingPrivateStep::Failed) => {
                    // Handle failure - stay at current visibility or go back to Private
                    next_visibility.set(ServerVisibility::Private);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die Server-Session Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Validator-Funktionen (ob Übergänge gültig/ungültig sind)
    //! 2. Observer-Logik (ob Events korrekt verarbeitet werden)
    //! 3. SubState-Übergänge (ob die Schritte korrekt durchlaufen werden)

    // Importiere die States und Events aus dem Parent-Modul
    use crate::events::session::{
        SetGoingPrivateStep, SetGoingPublicStep, SetServerShutdownStep, SetServerStartupStep,
    };
    use crate::states::session::{ServerStatus, ServerVisibility};

    mod helpers {

        #[cfg(feature = "hosted")]
        use crate::logic::app::on_change_app_scope;

        use crate::{
            AppScope, GoingPrivateStep, GoingPublicStep, ServerShutdownStep, ServerStartupStep,
            ServerStatus, ServerVisibility, SessionState, SessionType, SetGoingPrivateStep,
            SetGoingPublicStep, SetServerShutdownStep, SetServerStartupStep,
            logic::session::server::ServerSessionPlugin,
        };

        use bevy::{prelude::*, state::app::StatesPlugin};

        pub const STARTUP_STEPS: u8 = 3;
        pub const SHUTDOWN_STEPS: u8 = 5;

        pub const GOING_PUBLIC_STEPS: u8 = 3;
        pub const GOING_PRIVATE_STEPS: u8 = 4;

        pub fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, StatesPlugin, ServerSessionPlugin))
                .init_state::<AppScope>();
            #[cfg(feature = "hosted")]
            app.add_observer(on_change_app_scope);
            app
        }

        /// Runs the app for one update tick.
        pub fn update_app(app: &mut App, i: u8) {
            for _ in 0..i {
                app.update();
            }
        }

        pub fn setup_test_app(session_type: SessionType) -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            #[cfg(feature = "hosted")]
            {
                let session_type_state = app.world().resource::<State<SessionType>>();
                assert_eq!(session_type_state.get(), &SessionType::None);
            }

            #[cfg(feature = "hosted")]
            {
                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Splash);
            }

            #[cfg(feature = "hosted")]
            {
                let mut next_app_scope = app.world_mut().resource_mut::<NextState<AppScope>>();
                next_app_scope.set(AppScope::Session);
                app.update();
            }

            {
                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Session);
                let session_state = app.world().resource::<State<SessionState>>();
                assert_eq!(session_state.get(), &SessionState::Setup);
            }

            {
                let mut next_session_type =
                    app.world_mut().resource_mut::<NextState<SessionType>>();
                next_session_type.set(session_type);
                app.update();
            }
            {
                let session_type_state = app.world().resource::<State<SessionType>>();
                assert_eq!(session_type_state.get(), &session_type);
                let server_status = app.world().resource::<State<ServerStatus>>();
                assert_eq!(server_status.get(), &ServerStatus::Offline);
            }

            {
                app.world_mut().trigger(SetServerStartupStep::Start);
                app.update();

                let server_status = app.world().resource::<State<ServerStatus>>();
                assert_eq!(server_status.get(), &ServerStatus::Starting);

                let server_startup_step = app.world().resource::<State<ServerStartupStep>>();
                assert_eq!(server_startup_step.get(), &ServerStartupStep::Init);
            }

            app
        }

        #[cfg(feature = "hosted")]
        pub fn test_start_singleplayer() -> App {
            let app = setup_test_app(SessionType::Singleplayer);
            app
        }

        #[cfg(feature = "headless")]
        pub fn test_start_dedicated_server() -> App {
            let app = setup_test_app(SessionType::DedicatedServer);
            app
        }

        pub fn start_server(app: &mut App) {
            app.world_mut().trigger(SetServerStartupStep::Start);

            update_app(app, 1);
            let status = app.world().resource::<State<ServerStatus>>();
            assert_eq!(status.get(), &ServerStatus::Starting);
            let step = app.world().resource::<State<ServerStartupStep>>();
            assert_eq!(step.get(), &ServerStartupStep::Init);
        }

        pub fn server_startup_next_step(app: &mut App, i: u8) {
            for _ in 0..i {
                app.world_mut().trigger(SetServerStartupStep::Next);
                update_app(app, 1);
            }

            {
                let step = app.world().resource::<State<ServerStartupStep>>();
                assert_eq!(step.get(), &ServerStartupStep::Ready);
                let state = app.world().resource::<State<ServerStatus>>();
                assert_eq!(state.get(), &ServerStatus::Starting);
            }

            app.world_mut().trigger(SetServerStartupStep::Done);
            update_app(app, 1);

            {
                let state_after_done = app.world().resource::<State<ServerStatus>>();
                assert_eq!(state_after_done.get(), &ServerStatus::Running);
                let server_visibility = app.world().resource::<State<ServerVisibility>>();
                assert_eq!(server_visibility, &ServerVisibility::Private);
            }
        }

        /// Führt den Startup-Prozess aus und wirft an einem bestimmten Step einen Fehler.
        ///
        /// - fail_at_step: Der Step bei dem das Failed-Event gesendet wird
        ///   - 0 = Failed sofort nach Start
        ///   - 1 = Failed nach einem Next
        ///   - 2 = Failed nach zwei Nexts
        pub fn server_startup_fail_at_step(app: &mut App, fail_at_step: u8) {
            println!("Failure Step : {:?}", fail_at_step);

            for _ in 0..fail_at_step {
                app.world_mut().trigger(SetServerStartupStep::Next);
                update_app(app, 1);
            }

            app.world_mut().trigger(SetServerStartupStep::Failed);
            update_app(app, 1);

            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::None);
        }

        pub fn start_shutdown(app: &mut App) {
            {
                app.world_mut().trigger(SetServerShutdownStep::Start);
                update_app(app, 1);

                let server_status = app.world().resource::<State<ServerStatus>>();
                assert_eq!(server_status.get(), &ServerStatus::Stopping);

                let server_shutdown_step = app.world().resource::<State<ServerShutdownStep>>();
                assert_eq!(server_shutdown_step.get(), &ServerShutdownStep::SaveWorld);
            }
        }

        pub fn server_shutdown_next_step(app: &mut App, i: u8) {
            for _ in 0..i {
                app.world_mut().trigger(SetServerShutdownStep::Next);
                update_app(app, 1);
            }

            {
                let step = app.world().resource::<State<ServerShutdownStep>>();
                assert_eq!(step.get(), &ServerShutdownStep::Ready);
                let state = app.world().resource::<State<ServerStatus>>();
                assert_eq!(state.get(), &ServerStatus::Stopping);
            }

            app.world_mut().trigger(SetServerShutdownStep::Done);
            update_app(app, 1);

            {
                #[cfg(feature = "hosted")]
                {
                    let session_type = app.world().resource::<State<SessionType>>();
                    assert_eq!(session_type.get(), &SessionType::None);

                    let app_scope = app.world().resource::<State<AppScope>>();
                    assert_eq!(app_scope.get(), &AppScope::Menu);
                }

                #[cfg(feature = "headless")]
                {
                    update_app(app, 10);
                    let session_type = app.world().resource::<State<SessionType>>();
                    assert_eq!(session_type.get(), &SessionType::None);
                }
            }
        }

        pub fn server_shutdown_fail_at_step(app: &mut App, fail_at_step: u8) {
            println!("Failure Step : {:?}", fail_at_step);
            for _ in 0..fail_at_step {
                app.world_mut().trigger(SetServerShutdownStep::Next);
                update_app(app, 1);
            }

            app.world_mut().trigger(SetServerShutdownStep::Failed);
            update_app(app, 1);

            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::None);
        }

        pub fn server_goging_public(app: &mut App) {
            app.world_mut().trigger(SetGoingPublicStep::Start);
            update_app(app, 1);

            let server_visibilty = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(server_visibilty.get(), &ServerVisibility::GoingPublic);
            let step = app.world().resource::<State<GoingPublicStep>>();
            assert_eq!(step.get(), &GoingPublicStep::Validating);
        }

        pub fn server_goging_public_next_step(app: &mut App, i: u8) {
            for _ in 0..i {
                app.world_mut().trigger(SetGoingPublicStep::Next);
                update_app(app, 1);
            }

            let step = app.world().resource::<State<GoingPublicStep>>();
            assert_eq!(step.get(), &GoingPublicStep::Ready);
            let server_visibility = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(server_visibility.get(), &ServerVisibility::GoingPublic);

            app.world_mut().trigger(SetGoingPublicStep::Done);
            update_app(app, 1);

            let server_visibility = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(server_visibility.get(), &ServerVisibility::Public);
        }

        pub fn server_goging_private(app: &mut App) {
            app.world_mut().trigger(SetGoingPrivateStep::Start);
            update_app(app, 1);

            let server_visibilty = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(server_visibilty.get(), &ServerVisibility::GoingPrivate);
            let step = app.world().resource::<State<GoingPrivateStep>>();
            assert_eq!(step.get(), &GoingPrivateStep::DisconnectingClients);
        }

        pub fn server_goging_private_next_step(app: &mut App, i: u8) {
            for _ in 0..i {
                app.world_mut().trigger(SetGoingPrivateStep::Next);
                update_app(app, 1);
            }

            let step = app.world().resource::<State<GoingPrivateStep>>();
            assert_eq!(step.get(), &GoingPrivateStep::Ready);

            let server_visibility = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(server_visibility.get(), &ServerVisibility::GoingPrivate);

            app.world_mut().trigger(SetGoingPrivateStep::Done);
            update_app(app, 1);

            let server_visibility = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(server_visibility.get(), &ServerVisibility::Private);
        }

        /// Führt den Going-Public-Prozess aus und wirft an einem bestimmten Step einen Fehler.
        ///
        /// - fail_at_step: Der Step bei dem das Failed-Event gesendet wird
        ///   - 0 = Failed sofort nach Start (bei Validating)
        ///   - 1 = Failed nach einem Next (bei StartingServer)
        ///   - 2 = Failed nach zwei Nexts (bei StartingDiscovery)
        pub fn server_going_public_fail_at_step(app: &mut App, fail_at_step: u8) {
            println!("Going Public Failure Step: {:?}", fail_at_step);

            for _ in 0..fail_at_step {
                app.world_mut().trigger(SetGoingPublicStep::Next);
                update_app(app, 1);
            }

            app.world_mut().trigger(SetGoingPublicStep::Failed);
            update_app(app, 1);

            // Nach einem Failure sollte der Server wieder Private sein
            let server_visibility = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(server_visibility.get(), &ServerVisibility::Private);
        }

        /// Führt den Going-Private-Prozess aus und wirft an einem bestimmten Step einen Fehler.
        ///
        /// - fail_at_step: Der Step bei dem das Failed-Event gesendet wird
        ///   - 0 = Failed sofort nach Start (bei DisconnectingClients)
        ///   - 1 = Failed nach einem Next (bei ClosingServer)
        ///   - 2 = Failed nach zwei Nexts (bei Cleanup)
        pub fn server_going_private_fail_at_step(app: &mut App, fail_at_step: u8) {
            println!("Going Private Failure Step: {:?}", fail_at_step);

            for _ in 0..fail_at_step {
                app.world_mut().trigger(SetGoingPrivateStep::Next);
                update_app(app, 1);
            }

            app.world_mut().trigger(SetGoingPrivateStep::Failed);
            update_app(app, 1);

            // Nach einem Failure sollte der Server wieder Private sein
            let server_visibility = app.world().resource::<State<ServerVisibility>>();
            assert_eq!(server_visibility.get(), &ServerVisibility::Private);
        }
    }

    // =============================================================================
    // TESTS FÜR SERVER STARTUP STEP
    // =============================================================================

    mod startup_step_tests {

        use super::*;

        /// Test: ServerStartupStep::Start wechselt zu Init.
        #[test]
        fn test_start_singleplayer() {
            #[cfg(feature = "hosted")]
            let mut app = helpers::test_start_singleplayer();
            #[cfg(feature = "headless")]
            let mut app = helpers::test_start_dedicated_server();
            helpers::start_server(&mut app);
            helpers::server_startup_next_step(&mut app, helpers::STARTUP_STEPS);
        }

        /// Test: ServerStartup Failure bei verschiedenen Steps
        ///
        /// Testet, dass der Server bei einem Failed-Event korrekt zu Offline wechselt,
        /// unabhängig davon, bei welchem Step das Failure auftritt.
        #[test]
        fn test_server_startup_failure() {
            for step in 0..helpers::STARTUP_STEPS {
                #[cfg(feature = "hosted")]
                let mut app = helpers::test_start_singleplayer();
                #[cfg(feature = "headless")]
                let mut app = helpers::test_start_dedicated_server();
                helpers::start_server(&mut app);
                // Failed sofort nach Start
                helpers::server_startup_fail_at_step(&mut app, step);
            }
        }
    }

    // =============================================================================
    // TESTS FÜR SERVER SHUTDOWN STEP
    // =============================================================================

    mod shutdown_step_tests {
        use super::*;

        /// Test: ServerShutdownStep::Start wechselt zu SaveWorld.
        #[test]
        fn test_shutdown() {
            #[cfg(feature = "hosted")]
            let mut app = helpers::test_start_singleplayer();
            #[cfg(feature = "headless")]
            let mut app = helpers::test_start_dedicated_server();
            helpers::start_server(&mut app);
            helpers::server_startup_next_step(&mut app, helpers::STARTUP_STEPS);

            helpers::start_shutdown(&mut app);
            helpers::server_shutdown_next_step(&mut app, helpers::SHUTDOWN_STEPS);
        }

        #[test]
        fn test_shutdown_failure() {
            for step in 0..helpers::SHUTDOWN_STEPS {
                #[cfg(feature = "hosted")]
                let mut app = helpers::test_start_singleplayer();
                #[cfg(feature = "headless")]
                let mut app = helpers::test_start_dedicated_server();
                helpers::start_server(&mut app);
                helpers::server_startup_next_step(&mut app, helpers::STARTUP_STEPS);

                helpers::start_shutdown(&mut app);
                helpers::server_shutdown_fail_at_step(&mut app, step);
            }
        }
    }

    // =============================================================================
    // TESTS FÜR SERVER VISIBILITY
    // =============================================================================

    mod visibility_tests {

        use super::*;

        #[test]
        fn test_going_public() {
            #[cfg(feature = "hosted")]
            let mut app = helpers::test_start_singleplayer();
            #[cfg(feature = "headless")]
            let mut app = helpers::test_start_dedicated_server();
            helpers::start_server(&mut app);
            helpers::server_startup_next_step(&mut app, helpers::STARTUP_STEPS);

            helpers::server_goging_public(&mut app);
            helpers::server_goging_public_next_step(&mut app, helpers::GOING_PUBLIC_STEPS);
        }

        #[test]
        fn test_going_private() {
            #[cfg(feature = "hosted")]
            let mut app = helpers::test_start_singleplayer();
            #[cfg(feature = "headless")]
            let mut app = helpers::test_start_dedicated_server();
            helpers::server_startup_next_step(&mut app, helpers::STARTUP_STEPS);

            helpers::server_goging_public(&mut app);
            helpers::server_goging_public_next_step(&mut app, helpers::GOING_PUBLIC_STEPS);

            helpers::server_goging_private(&mut app);
            helpers::server_goging_private_next_step(&mut app, helpers::GOING_PRIVATE_STEPS);
        }

        /// Test: Going-Public-Prozess kann an verschiedenen Steps fehlschlagen.
        ///
        /// Testet, dass der Server bei einem Failed-Event korrekt zu Private wechselt,
        /// unabhängig davon, bei welchem Step das Failure auftritt.
        #[test]
        fn test_going_public_failure() {
            for step in 0..helpers::GOING_PUBLIC_STEPS {
                #[cfg(feature = "hosted")]
                let mut app = helpers::test_start_singleplayer();
                #[cfg(feature = "headless")]
                let mut app = helpers::test_start_dedicated_server();
                helpers::start_server(&mut app);
                helpers::server_startup_next_step(&mut app, helpers::STARTUP_STEPS);

                helpers::server_goging_public(&mut app);
                helpers::server_going_public_fail_at_step(&mut app, step);
            }
        }

        /// Test: Going-Private-Prozess kann an verschiedenen Steps fehlschlagen.
        ///
        /// Testet, dass der Server bei einem Failed-Event korrekt zu Private wechselt,
        /// unabhängig davon, bei welchem Step das Failure auftritt.
        #[test]
        fn test_going_private_failure() {
            for step in 0..helpers::GOING_PRIVATE_STEPS {
                #[cfg(feature = "hosted")]
                let mut app = helpers::test_start_singleplayer();
                #[cfg(feature = "headless")]
                let mut app = helpers::test_start_dedicated_server();
                helpers::start_server(&mut app);
                helpers::server_startup_next_step(&mut app, helpers::STARTUP_STEPS);

                // Server muss erst public sein, um private zu werden
                helpers::server_goging_public(&mut app);
                helpers::server_goging_public_next_step(&mut app, helpers::GOING_PUBLIC_STEPS);

                helpers::server_goging_private(&mut app);
                helpers::server_going_private_fail_at_step(&mut app, step);
            }
        }
    }

    // =============================================================================
    // INTEGRATIONSTEST: KOMPLETTER SERVER LIFECYCLE
    // =============================================================================

    mod integration_tests {
        use super::*;

        /// Test: Kompletter Server-Lebenszyklus.
        /// Offline -> Starting -> Running -> Going public -> Public -> Going private -> Stopping -> Offline
        #[test]
        fn test_full_server_lifecycle() {
            #[cfg(feature = "hosted")]
            let mut app = helpers::test_start_singleplayer();
            #[cfg(feature = "headless")]
            let mut app = helpers::test_start_dedicated_server();
            helpers::server_startup_next_step(&mut app, helpers::STARTUP_STEPS);

            helpers::server_goging_public(&mut app);
            helpers::server_goging_public_next_step(&mut app, helpers::GOING_PUBLIC_STEPS);

            helpers::server_goging_private(&mut app);
            helpers::server_goging_private_next_step(&mut app, helpers::GOING_PRIVATE_STEPS);

            helpers::start_shutdown(&mut app);
            helpers::server_shutdown_next_step(&mut app, helpers::SHUTDOWN_STEPS);
        }
    }
    // =============================================================================
    // TESTS FÜR VALIDATOR-FUNKTIONEN
    // =============================================================================

    mod validator_tests {
        use super::*;

        // Importiere alle Validator-Funktionen
        use super::super::is_valid_server_status_shutdown_transition;
        use super::super::is_valid_server_status_startup_transition;
        use super::super::is_valid_server_visibility_private_transition;
        use super::super::is_valid_server_visibility_public_transition;

        /// Test: Gültige ServerStatus-Startup-Übergänge werden als gültig erkannt.
        ///
        /// ServerStatus::Starting kann zu:
        /// - Next (fortschreiten im Startup)
        /// - Done (Startup erfolgreich abgeschlossen)
        /// - Failed (Startup fehlgeschlagen)
        #[test]
        fn test_valid_server_status_startup_transitions() {
            // Starting → Next ist gültig (fortschreiten)
            assert!(is_valid_server_status_startup_transition(
                &ServerStatus::Starting,
                &SetServerStartupStep::Next
            ));

            // Starting → Done ist gültig (erfolgreich abgeschlossen)
            assert!(is_valid_server_status_startup_transition(
                &ServerStatus::Starting,
                &SetServerStartupStep::Done
            ));

            // Starting → Failed ist gültig (fehlgeschlagen)
            assert!(is_valid_server_status_startup_transition(
                &ServerStatus::Starting,
                &SetServerStartupStep::Failed
            ));

            // Offline → Start ist gültig (Server starten)
            assert!(is_valid_server_status_startup_transition(
                &ServerStatus::Offline,
                &SetServerStartupStep::Start
            ));
        }

        /// Test: Ungültige ServerStatus-Startup-Übergänge werden blockiert.
        #[test]
        fn test_invalid_server_status_startup_transitions() {
            // Running → Start ist ungültig (Server läuft bereits)
            assert!(!is_valid_server_status_startup_transition(
                &ServerStatus::Running,
                &SetServerStartupStep::Start
            ));

            // Stopping → Start ist ungültig (Server wird heruntergefahren)
            assert!(!is_valid_server_status_startup_transition(
                &ServerStatus::Stopping,
                &SetServerStartupStep::Start
            ));

            // Starting → Start ist ungültig (Server startet bereits)
            assert!(!is_valid_server_status_startup_transition(
                &ServerStatus::Starting,
                &SetServerStartupStep::Start
            ));
        }

        /// Test: Gültige ServerStatus-Shutdown-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_server_status_shutdown_transitions() {
            // Running → Start ist gültig (Server herunterfahren)
            assert!(is_valid_server_status_shutdown_transition(
                &ServerStatus::Running,
                &SetServerShutdownStep::Start
            ));

            // Stopping → Next ist gültig (fortschreiten)
            assert!(is_valid_server_status_shutdown_transition(
                &ServerStatus::Stopping,
                &SetServerShutdownStep::Next
            ));

            // Stopping → Done ist gültig (erfolgreich heruntergefahren)
            assert!(is_valid_server_status_shutdown_transition(
                &ServerStatus::Stopping,
                &SetServerShutdownStep::Done
            ));

            // Stopping → Failed ist gültig (Fehler beim Herunterfahren)
            assert!(is_valid_server_status_shutdown_transition(
                &ServerStatus::Stopping,
                &SetServerShutdownStep::Failed
            ));
        }

        /// Test: Ungültige ServerStatus-Shutdown-Übergänge werden blockiert.
        #[test]
        fn test_invalid_server_status_shutdown_transitions() {
            // Offline → Start ist ungültig (Server ist nicht gestartet)
            assert!(!is_valid_server_status_shutdown_transition(
                &ServerStatus::Offline,
                &SetServerShutdownStep::Start
            ));

            // Starting → Start ist ungültig (Server startet noch)
            assert!(!is_valid_server_status_shutdown_transition(
                &ServerStatus::Starting,
                &SetServerShutdownStep::Start
            ));

            // Stopping → Start ist ungültig (Server wird bereits heruntergefahren)
            assert!(!is_valid_server_status_shutdown_transition(
                &ServerStatus::Stopping,
                &SetServerShutdownStep::Start
            ));
        }

        /// Test: Gültige GoingPublic-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_going_public_transitions() {
            // Private → Start ist gültig (Server soll public werden)
            assert!(is_valid_server_visibility_public_transition(
                &ServerVisibility::Private,
                &SetGoingPublicStep::Start
            ));

            // GoingPublic → Next ist gültig (fortschreiten)
            assert!(is_valid_server_visibility_public_transition(
                &ServerVisibility::GoingPublic,
                &SetGoingPublicStep::Next
            ));

            // GoingPublic → Done ist gültig (erfolgreich public geworden)
            assert!(is_valid_server_visibility_public_transition(
                &ServerVisibility::GoingPublic,
                &SetGoingPublicStep::Done
            ));

            // GoingPublic → Failed ist gültig (Fehler beim public machen)
            assert!(is_valid_server_visibility_public_transition(
                &ServerVisibility::GoingPublic,
                &SetGoingPublicStep::Failed
            ));
        }

        /// Test: Ungültige GoingPublic-Übergänge werden blockiert.
        #[test]
        fn test_invalid_going_public_transitions() {
            // Public → Start ist ungültig (Server ist bereits public)
            assert!(!is_valid_server_visibility_public_transition(
                &ServerVisibility::Public,
                &SetGoingPublicStep::Start
            ));

            // GoingPrivate → Start ist ungültig (Server wird bereits private)
            assert!(!is_valid_server_visibility_public_transition(
                &ServerVisibility::GoingPrivate,
                &SetGoingPublicStep::Start
            ));
        }

        /// Test: Gültige GoingPrivate-Übergänge werden als gültig erkannt.
        #[test]
        fn test_valid_going_private_transitions() {
            // Public → Start ist gültig (Server soll private werden)
            assert!(is_valid_server_visibility_private_transition(
                &ServerVisibility::Public,
                &SetGoingPrivateStep::Start
            ));

            // GoingPrivate → Next ist gültig (fortschreiten)
            assert!(is_valid_server_visibility_private_transition(
                &ServerVisibility::GoingPrivate,
                &SetGoingPrivateStep::Next
            ));

            // GoingPrivate → Done ist gültig (erfolgreich private geworden)
            assert!(is_valid_server_visibility_private_transition(
                &ServerVisibility::GoingPrivate,
                &SetGoingPrivateStep::Done
            ));

            // GoingPrivate → Failed ist gültig
            assert!(is_valid_server_visibility_private_transition(
                &ServerVisibility::GoingPrivate,
                &SetGoingPrivateStep::Failed
            ));
        }

        /// Test: Ungültige GoingPrivate-Übergänge werden blockiert.
        #[test]
        fn test_invalid_going_private_transitions() {
            // Private → Start ist ungültig (Server ist bereits private)
            assert!(!is_valid_server_visibility_private_transition(
                &ServerVisibility::Private,
                &SetGoingPrivateStep::Start
            ));

            // GoingPublic → Start ist ungültig (Server wird bereits public)
            assert!(!is_valid_server_visibility_private_transition(
                &ServerVisibility::GoingPublic,
                &SetGoingPrivateStep::Start
            ));
        }
    }
}
