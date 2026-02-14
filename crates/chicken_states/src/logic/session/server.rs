use {
    crate::{
        events::session::SetServerVisibility,
        states::session::{PhysicsSimulation, ServerVisibility},
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, ResMut},
};

pub struct ServerSessionPlugin;

impl Plugin for ServerSessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ServerVisibility>()
            .add_observer(on_server_visibility_event)
            .add_computed_state::<PhysicsSimulation>(); // Server computes physics simulation too
    }
}

fn on_server_visibility_event(
    event: On<SetServerVisibility>,
    mut next_state: ResMut<NextState<ServerVisibility>>,
) {
    // TODO: going public only when in Game Menu or PendingPublic State before
    next_state.set(event.transition);
}
