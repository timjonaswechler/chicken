use {
    crate::states::{app::AppScope, session::SessionState},
    bevy::prelude::{App, AppExtStates, Plugin},
};

#[cfg(feature = "hosted")]
use {
    crate::{
        events::app::ChangeAppScope,
        states::{menu::main::MainMenuContext, session::SessionType},
    },
    bevy::prelude::{
        ButtonInput, IntoScheduleConfigs, KeyCode, NextState, On, OnEnter, Res, ResMut, Resource,
        Time, Update, in_state,
    },
};

pub struct AppLogicPlugin;

impl Plugin for AppLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppScope>().add_sub_state::<SessionState>();
        #[cfg(feature = "hosted")]
        {
            // Initialize the splash timer resource
            app.insert_resource(SplashTimer::default())
                // Reset timer when entering Splash
                .add_systems(OnEnter(AppScope::Splash), reset_splash_timer)
                // Run the splash screen handler while we're in the Splash state
                .add_systems(
                    Update,
                    (handle_splash_screen).run_if(in_state(AppScope::Splash)),
                )
                .add_observer(on_change_app_scope);
        }
    }
}

#[cfg(feature = "hosted")]
#[derive(Default, Resource)]
struct SplashTimer {
    pub elapsed: f32,
}

#[cfg(feature = "hosted")]
fn reset_splash_timer(mut timer: ResMut<SplashTimer>) {
    timer.elapsed = 0.0;
}

#[cfg(feature = "hosted")]
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

#[cfg(feature = "hosted")]
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
