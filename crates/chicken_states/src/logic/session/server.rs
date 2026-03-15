#[cfg(feature = "hosted")]
use crate::states::app::AppScope;

#[cfg(feature = "headless")]
use {
    bevy::{app::AppExit, prelude::MessageWriter},
    chicken_exitcodes::ExitCode,
};

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
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, Resource, State, warn},
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

/// Exists if Multiplayer Host set confirm and will be removed after server is set to running.
/// There the Server will be set to GoingPublic automatically.
#[derive(Resource)]
pub struct PendingGoingPublic;

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
        (ServerStatus::Starting, SetServerStartupStep::Next)
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
        SetServerStartupStep::Failed => {
            #[cfg(feature = "hosted")]
            {
                next_server_status.set(ServerStatus::Offline);
                next_session_type.set(SessionType::None);
                next_app_scope.set(AppScope::Menu);
            }
            #[cfg(feature = "headless")]
            {
                exit_writer.write(AppExit::Error(ExitCode::ServerStartupFailed.nonzero()));
            }
        }
        _ => {
            let current = match current {
                Some(c) => *c.get(),
                None => {
                    warn!("ServerStartupStep does not exist - ServerStatus must be Starting first");
                    return;
                }
            };

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
        SetServerShutdownStep::Failed => {
            #[cfg(feature = "hosted")]
            {
                next_server_status.set(ServerStatus::Offline);
                next_session_type.set(SessionType::None);
                next_app_scope.set(AppScope::Menu);
            }
            #[cfg(feature = "headless")]
            {
                exit_writer.write(AppExit::Error(ExitCode::ServerShutdownFailed.nonzero()));
            }
        }
        // Next/Done: ServerShutdownStep muss existieren (Status muss Stopping sein)
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
                        next_server_status.set(ServerStatus::Offline);
                        next_session_type.set(SessionType::None);
                        next_app_scope.set(AppScope::Menu);
                    }

                    #[cfg(feature = "headless")]
                    {
                        exit_writer.write(AppExit::Success);
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
    #[cfg(feature = "headless")] mut exit_writer: MessageWriter<AppExit>,
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
        SetGoingPublicStep::Failed => {
            #[cfg(feature = "hosted")]
            {
                next_visibility.set(ServerVisibility::Private);
                return;
            }
            #[cfg(feature = "headless")]
            {
                exit_writer.write(AppExit::Error(ExitCode::ServerGoingPublicFailed.nonzero()));
                return;
            }
        }
        // Next/Done: GoingPublicStep muss existieren
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
    #[cfg(feature = "headless")] mut exit_writer: MessageWriter<AppExit>,
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
        SetGoingPrivateStep::Failed => {
            #[cfg(feature = "hosted")]
            {
                // TODO: dont know if this is the correct behavior for hosted
                next_visibility.set(ServerVisibility::Private);
            }
            #[cfg(feature = "headless")]
            {
                // TODO: dont know if we need this?!
                exit_writer.write(AppExit::Error(ExitCode::ServerGoingPrivateFailed.nonzero()));
                return;
            }
        }
        // Next/Done: GoingPrivateStep muss existieren
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

                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Validator-Tests für die Server-Session Logik.
    //!
    //! Observer- und Lifecycle-Tests befinden sich in:
    //! - `tests/session_server/` (hosted)
    //! - `tests/session_server_headless/` (headless)

    use crate::events::session::{
        SetGoingPrivateStep, SetGoingPublicStep, SetServerShutdownStep, SetServerStartupStep,
    };
    use crate::states::session::{ServerStatus, ServerVisibility};

    mod validator_tests {
        use super::*;

        use super::super::is_valid_server_status_shutdown_transition;
        use super::super::is_valid_server_status_startup_transition;
        use super::super::is_valid_server_visibility_private_transition;
        use super::super::is_valid_server_visibility_public_transition;

        /// Gültige ServerStatus-Startup-Übergänge.
        #[test]
        fn test_valid_server_status_startup_transitions() {
            assert!(is_valid_server_status_startup_transition(
                &ServerStatus::Starting,
                &SetServerStartupStep::Next
            ));
            assert!(is_valid_server_status_startup_transition(
                &ServerStatus::Starting,
                &SetServerStartupStep::Done
            ));
            assert!(is_valid_server_status_startup_transition(
                &ServerStatus::Starting,
                &SetServerStartupStep::Failed
            ));
        }

        /// Ungültige ServerStatus-Startup-Übergänge werden blockiert.
        #[test]
        fn test_invalid_server_status_startup_transitions() {
            // Running → Next ist ungültig (Server läuft bereits)
            assert!(!is_valid_server_status_startup_transition(
                &ServerStatus::Running,
                &SetServerStartupStep::Next
            ));
            // Stopping → Done ist ungültig (Server wird heruntergefahren)
            assert!(!is_valid_server_status_startup_transition(
                &ServerStatus::Stopping,
                &SetServerStartupStep::Done
            ));
        }

        /// Gültige ServerStatus-Shutdown-Übergänge.
        #[test]
        fn test_valid_server_status_shutdown_transitions() {
            assert!(is_valid_server_status_shutdown_transition(
                &ServerStatus::Running,
                &SetServerShutdownStep::Start
            ));
            assert!(is_valid_server_status_shutdown_transition(
                &ServerStatus::Stopping,
                &SetServerShutdownStep::Next
            ));
            assert!(is_valid_server_status_shutdown_transition(
                &ServerStatus::Stopping,
                &SetServerShutdownStep::Done
            ));
            assert!(is_valid_server_status_shutdown_transition(
                &ServerStatus::Stopping,
                &SetServerShutdownStep::Failed
            ));
        }

        /// Ungültige ServerStatus-Shutdown-Übergänge werden blockiert.
        #[test]
        fn test_invalid_server_status_shutdown_transitions() {
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

        /// Gültige GoingPublic-Übergänge.
        #[test]
        fn test_valid_going_public_transitions() {
            assert!(is_valid_server_visibility_public_transition(
                &ServerVisibility::Private,
                &SetGoingPublicStep::Start
            ));
            assert!(is_valid_server_visibility_public_transition(
                &ServerVisibility::GoingPublic,
                &SetGoingPublicStep::Next
            ));
            assert!(is_valid_server_visibility_public_transition(
                &ServerVisibility::GoingPublic,
                &SetGoingPublicStep::Done
            ));
            assert!(is_valid_server_visibility_public_transition(
                &ServerVisibility::GoingPublic,
                &SetGoingPublicStep::Failed
            ));
        }

        /// Ungültige GoingPublic-Übergänge werden blockiert.
        #[test]
        fn test_invalid_going_public_transitions() {
            assert!(!is_valid_server_visibility_public_transition(
                &ServerVisibility::Public,
                &SetGoingPublicStep::Start
            ));
            assert!(!is_valid_server_visibility_public_transition(
                &ServerVisibility::GoingPrivate,
                &SetGoingPublicStep::Start
            ));
        }

        /// Gültige GoingPrivate-Übergänge.
        #[test]
        fn test_valid_going_private_transitions() {
            assert!(is_valid_server_visibility_private_transition(
                &ServerVisibility::Public,
                &SetGoingPrivateStep::Start
            ));
            assert!(is_valid_server_visibility_private_transition(
                &ServerVisibility::GoingPrivate,
                &SetGoingPrivateStep::Next
            ));
            assert!(is_valid_server_visibility_private_transition(
                &ServerVisibility::GoingPrivate,
                &SetGoingPrivateStep::Done
            ));
            assert!(is_valid_server_visibility_private_transition(
                &ServerVisibility::GoingPrivate,
                &SetGoingPrivateStep::Failed
            ));
        }

        /// Ungültige GoingPrivate-Übergänge werden blockiert.
        #[test]
        fn test_invalid_going_private_transitions() {
            assert!(!is_valid_server_visibility_private_transition(
                &ServerVisibility::Private,
                &SetGoingPrivateStep::Start
            ));
            assert!(!is_valid_server_visibility_private_transition(
                &ServerVisibility::GoingPublic,
                &SetGoingPrivateStep::Start
            ));
        }
    }
}
