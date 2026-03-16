use {
    crate::states::app::AppScope,
    bevy::prelude::{App, AppExtStates, Plugin},
};

#[cfg(any(feature = "hosted", feature = "headless"))]
use crate::states::session::{SessionState, SessionType};

#[cfg(feature = "hosted")]
use {
    crate::{events::app::SetAppScope, states::menu::main::MainMenuScreen},
    bevy::{
        app::AppExit,
        ecs::message::MessageWriter,
        input::InputPlugin,
        prelude::{
            ButtonInput, IntoScheduleConfigs, KeyCode, NextState, On, OnEnter, Res, ResMut,
            Resource, State, Time, Update, in_state, warn,
        },
    },
};

pub struct AppLogicPlugin;

impl Plugin for AppLogicPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "hosted")]
        if !app.is_plugin_added::<InputPlugin>() {
            app.add_plugins(InputPlugin);
        }
        app.init_state::<AppScope>();
        #[cfg(any(feature = "hosted", feature = "headless"))]
        app.init_state::<SessionType>()
            .add_sub_state::<SessionState>();

        #[cfg(feature = "hosted")]
        {
            // app.add_observer(on_set_session_type);

            app.add_sub_state::<MainMenuScreen>();
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

/// Validates transitions for AppScope.
#[cfg(feature = "hosted")]
pub(crate) fn is_valid_app_scope_transition(from: &AppScope, to: &SetAppScope) -> bool {
    matches!(
        (from, to),
        (AppScope::Splash, SetAppScope::Menu)
            | (AppScope::Menu, SetAppScope::Session)
            | (AppScope::Session, SetAppScope::Menu)
            | (_, SetAppScope::Exit)
    )
}

#[cfg(feature = "hosted")]
pub(crate) fn on_change_app_scope(
    event: On<SetAppScope>,
    current: Res<State<AppScope>>,
    mut state: ResMut<NextState<AppScope>>,
    mut session_type: ResMut<NextState<SessionType>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    if !is_valid_app_scope_transition(current.get(), event.event()) {
        warn!(
            "Invalid AppScope transition: {:?} -> {:?}",
            current.get(),
            event.event()
        );
        return;
    }

    match event.event() {
        SetAppScope::Menu => {
            state.set(AppScope::Menu);
            session_type.set(SessionType::None);
        }

        SetAppScope::Session => {
            state.set(AppScope::Session);
        }

        SetAppScope::Exit => {
            exit_writer.write(AppExit::Success);
        }
    }
}

#[cfg(test)]
mod tests {
    // =============================================================================
    // TESTS FÜR VALIDATOR-FUNKTIONEN
    // =============================================================================

    #[cfg(feature = "hosted")]
    mod app_scope_validator_tests {
        use crate::events::app::SetAppScope;
        use crate::logic::app::is_valid_app_scope_transition;
        use crate::states::app::AppScope;

        /// Test: Gültige AppScope-Übergänge werden als gültig erkannt.
        ///
        /// Gültige Übergänge:
        /// - Splash → Menu
        /// - Menu → Session
        /// - Session → Menu
        #[test]
        fn test_valid_app_scope_transitions() {
            // Splash → Menu
            assert!(is_valid_app_scope_transition(
                &AppScope::Splash,
                &SetAppScope::Menu
            ));

            // Menu → Session
            assert!(is_valid_app_scope_transition(
                &AppScope::Menu,
                &SetAppScope::Session
            ));

            // Session → Menu
            assert!(is_valid_app_scope_transition(
                &AppScope::Session,
                &SetAppScope::Menu
            ));

            // Any → Exit (always allowed)
            assert!(is_valid_app_scope_transition(
                &AppScope::Splash,
                &SetAppScope::Exit
            ));
            assert!(is_valid_app_scope_transition(
                &AppScope::Menu,
                &SetAppScope::Exit
            ));
            assert!(is_valid_app_scope_transition(
                &AppScope::Session,
                &SetAppScope::Exit
            ));
        }

        /// Test: Ungültige AppScope-Übergänge werden blockiert.
        ///
        /// Ungültige Übergänge:
        /// - Menu → Splash
        /// - Session → Splash
        /// - Menu → Menu (keine Änderung)
        /// - Session → Session (keine Änderung)
        /// - Splash → Splash (keine Änderung)
        #[test]
        fn test_invalid_app_scope_transitions() {
            // Menu → Splash ist ungültig (Splash is not a valid SetAppScope variant)
            // Note: With explicit variants, Menu → Splash is no longer a possible transition
            // since SetAppScope doesn't have a Splash variant

            // Splash → Session ist ungültig
            assert!(!is_valid_app_scope_transition(
                &AppScope::Splash,
                &SetAppScope::Session
            ));

            assert!(!is_valid_app_scope_transition(
                &AppScope::Menu,
                &SetAppScope::Menu
            ));
        }
    }
}
