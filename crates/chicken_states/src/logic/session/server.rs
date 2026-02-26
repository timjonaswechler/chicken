use {
    crate::{
        events::session::{
            SetGoingPrivateStep, SetGoingPublicStep, SetServerShutdownStep, SetServerStartupStep,
            SetServerStatus, SetServerVisibility,
        },
        states::session::{
            GoingPrivateStep, GoingPublicStep, PhysicsSimulation, ServerShutdownStep,
            ServerStartupStep, ServerStatus, ServerVisibility,
        },
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State, warn},
};

pub struct ServerSessionPlugin;

impl Plugin for ServerSessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ServerVisibility>()
            .add_sub_state::<ServerStatus>()
            .add_sub_state::<ServerStartupStep>()
            .add_sub_state::<ServerShutdownStep>()
            .add_sub_state::<GoingPublicStep>()
            .add_sub_state::<GoingPrivateStep>()
            .add_observer(on_server_visibility_event)
            .add_observer(on_server_status_event)
            .add_observer(on_server_startup_step)
            .add_observer(on_server_shutdown_step)
            .add_observer(on_going_public_step)
            .add_observer(on_going_private_step)
            .add_computed_state::<PhysicsSimulation>();
    }
}

fn is_valid_visibility_transition(from: &ServerVisibility, to: &ServerVisibility) -> bool {
    matches!(
        (from, to),
        (ServerVisibility::Private, ServerVisibility::GoingPublic)
            | (ServerVisibility::GoingPublic, ServerVisibility::Public)
            | (ServerVisibility::GoingPublic, ServerVisibility::Private)
            | (ServerVisibility::Public, ServerVisibility::GoingPrivate)
            | (ServerVisibility::GoingPrivate, ServerVisibility::Private)
    )
}

fn on_server_visibility_event(
    event: On<SetServerVisibility>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
    current: Res<State<ServerVisibility>>,
) {
    let current = current.get();
    let target = &event.transition;

    if !is_valid_visibility_transition(current, target) {
        warn!(
            "Unexpected ServerVisibility transition: {:?} -> {:?}",
            current, target
        );
    }

    next_state.set(event.transition);
}

fn is_valid_server_status_transition(from: &ServerStatus, to: &ServerStatus) -> bool {
    matches!(
        (from, to),
        (ServerStatus::Offline, ServerStatus::Starting)
            | (ServerStatus::Starting, ServerStatus::Running)
            | (ServerStatus::Starting, ServerStatus::Offline)
            | (ServerStatus::Running, ServerStatus::Stopping)
            | (ServerStatus::Stopping, ServerStatus::Offline)
    )
}

fn on_server_status_event(
    event: On<SetServerStatus>,
    mut next_state: ResMut<NextState<ServerStatus>>,
    current: Res<State<ServerStatus>>,
) {
    if !is_valid_server_status_transition(current.get(), &event.transition) {
        warn!(
            "Unexpected ServerStatus transition: {:?} -> {:?}",
            current.get(),
            event.transition
        );
    }

    next_state.set(event.transition);
}

fn on_server_startup_step(
    event: On<SetServerStartupStep>,
    current: Res<State<ServerStartupStep>>,
    mut next_state: ResMut<NextState<ServerStartupStep>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
) {
    match *event.event() {
        SetServerStartupStep::Start => {
            next_state.set(ServerStartupStep::Init);
        }
        SetServerStartupStep::Next => match current.get() {
            ServerStartupStep::Init => next_state.set(ServerStartupStep::LoadWorld),
            ServerStartupStep::LoadWorld => next_state.set(ServerStartupStep::SpawnEntities),
            ServerStartupStep::SpawnEntities => next_state.set(ServerStartupStep::Ready),
            ServerStartupStep::Ready => {}
        },
        SetServerStartupStep::Done => {
            next_server_status.set(ServerStatus::Running);
        }
        SetServerStartupStep::Failed => {
            next_server_status.set(ServerStatus::Offline);
        }
    }
}

fn on_server_shutdown_step(
    event: On<SetServerShutdownStep>,
    current: Res<State<ServerShutdownStep>>,
    mut next_state: ResMut<NextState<ServerShutdownStep>>,
    mut next_server_status: ResMut<NextState<ServerStatus>>,
) {
    match *event.event() {
        SetServerShutdownStep::Start => {
            next_state.set(ServerShutdownStep::SaveWorld);
        }
        SetServerShutdownStep::Next => match current.get() {
            ServerShutdownStep::SaveWorld => {
                next_state.set(ServerShutdownStep::DisconnectClients);
            }
            ServerShutdownStep::DisconnectClients => {
                #[cfg(feature = "hosted")]
                next_state.set(ServerShutdownStep::DespawnLocalClient);
                #[cfg(feature = "headless")]
                next_state.set(ServerShutdownStep::Cleanup);
            }
            #[cfg(feature = "hosted")]
            ServerShutdownStep::DespawnLocalClient => {
                next_state.set(ServerShutdownStep::Cleanup);
            }
            ServerShutdownStep::Cleanup => {}
        },
        SetServerShutdownStep::Done => {
            next_server_status.set(ServerStatus::Offline);
        }
    }
}

fn on_going_public_step(
    event: On<SetGoingPublicStep>,
    current: Res<State<GoingPublicStep>>,
    mut next_state: ResMut<NextState<GoingPublicStep>>,
    mut next_visibility: ResMut<NextState<ServerVisibility>>,
) {
    match *event.event() {
        SetGoingPublicStep::Start => {
            next_state.set(GoingPublicStep::Validating);
        }
        SetGoingPublicStep::Next => match current.get() {
            GoingPublicStep::Validating => next_state.set(GoingPublicStep::StartingServer),
            GoingPublicStep::StartingServer => next_state.set(GoingPublicStep::StartingDiscovery),
            GoingPublicStep::StartingDiscovery => next_state.set(GoingPublicStep::Ready),
            GoingPublicStep::Ready => {}
        },
        SetGoingPublicStep::Done => {
            next_visibility.set(ServerVisibility::Public);
        }
        SetGoingPublicStep::Failed => {
            next_visibility.set(ServerVisibility::Private);
        }
    }
}

fn on_going_private_step(
    event: On<SetGoingPrivateStep>,
    current: Res<State<GoingPrivateStep>>,
    mut next_state: ResMut<NextState<GoingPrivateStep>>,
    mut next_visibility: ResMut<NextState<ServerVisibility>>,
) {
    match *event.event() {
        SetGoingPrivateStep::Start => {
            next_state.set(GoingPrivateStep::DisconnectingClients);
        }
        SetGoingPrivateStep::Next => match current.get() {
            GoingPrivateStep::DisconnectingClients => {
                next_state.set(GoingPrivateStep::ClosingServer);
            }
            GoingPrivateStep::ClosingServer => next_state.set(GoingPrivateStep::CleanupComplete),
            GoingPrivateStep::CleanupComplete => {}
        },
        SetGoingPrivateStep::Done => {
            next_visibility.set(ServerVisibility::Private);
        }
    }
}
