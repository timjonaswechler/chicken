use crate::states::app::AppScope;
use crate::states::session::SessionState;
use bevy::prelude::{App, AppExtStates, Plugin};

#[cfg(feature = "client")]
use bevy::prelude::{
    in_state, ButtonInput, IntoScheduleConfigs, KeyCode, NextState, OnEnter, Res, ResMut, Resource,
    Time, Update,
};

pub struct AppLogicPlugin;

impl Plugin for AppLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppScope>().add_sub_state::<SessionState>();
        #[cfg(feature = "client")]
        {
            // Initialize the splash timer resource
            app.insert_resource(SplashTimer::default());

            // Reset timer when entering Splash
            app.add_systems(OnEnter(AppScope::Splash), reset_splash_timer);

            // Run the splash screen handler while we're in the Splash state
            app.add_systems(
                Update,
                (handle_splash_screen).run_if(in_state(AppScope::Splash)),
            );
        }
    }
}

#[cfg(feature = "client")]
#[derive(Default, Resource)]
struct SplashTimer {
    pub elapsed: f32,
}

#[cfg(feature = "client")]
fn reset_splash_timer(mut timer: ResMut<SplashTimer>) {
    timer.elapsed = 0.0;
}

#[cfg(feature = "client")]
fn handle_splash_screen(
    mut next_state: ResMut<NextState<AppScope>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut splash_timer: ResMut<SplashTimer>,
) {
    // Allow pressing Escape to skip the splash immediately.
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppScope::Menu);
        return;
    }

    // Accumulate elapsed time and transition after 5 seconds.
    splash_timer.elapsed += time.delta().as_secs_f32();
    if splash_timer.elapsed >= 5.0 {
        next_state.set(AppScope::Menu);
    }
}
