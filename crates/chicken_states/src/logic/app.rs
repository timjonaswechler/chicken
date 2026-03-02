use {
    crate::{
        events::session::SetSessionType,
        states::app::AppScope,
        states::session::{SessionState, SessionType},
    },
    bevy::{
        input::InputPlugin,
        prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State, warn},
    },
};

#[cfg(feature = "hosted")]
use {
    crate::{events::app::SetAppScope, states::menu::main::MainMenuContext},
    bevy::prelude::{
        ButtonInput, IntoScheduleConfigs, KeyCode, OnEnter, Resource, Time, Update, in_state,
    },
};

pub struct AppLogicPlugin;

impl Plugin for AppLogicPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<InputPlugin>() {
            app.add_plugins(InputPlugin);
        }
        app.init_state::<AppScope>()
            .init_state::<SessionType>()
            .add_sub_state::<SessionState>()
            .add_observer(on_set_session_type);

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
pub(crate) fn on_change_app_scope(
    event: On<SetAppScope>,
    mut state: ResMut<NextState<AppScope>>,
    mut session_type: ResMut<NextState<SessionType>>,
    mut menu_state: ResMut<NextState<MainMenuContext>>,
) {
    if matches!(event.event(), SetAppScope::To(AppScope::Menu)) {
        state.set(AppScope::Menu);
        session_type.set(SessionType::None);
        menu_state.set(MainMenuContext::Main);
    }
}

/// Validates transitions for SessionType.
pub(crate) fn is_valid_session_type_transition(from: &SessionType, to: &SetSessionType) -> bool {
    matches!(
        (from, to),
        (SessionType::None, SetSessionType::To(_)) | (_, SetSessionType::To(SessionType::None))
    )
}

pub fn on_set_session_type(
    event: On<SetSessionType>,
    current: Res<State<SessionType>>,
    mut next_session_type: ResMut<NextState<SessionType>>,
) {
    if !is_valid_session_type_transition(current.get(), event.event()) {
        warn!(
            "Invalid ServerStatus transition for ServerStartupStep: {:?} with parent status {:?}",
            event.event(),
            current.get()
        );
        return;
    }

    match *event.event() {
        SetSessionType::To(session_type) => {
            next_session_type.set(session_type);
        }
    }
}
