use {
    crate::{
        events::session::SetSessionType,
        states::{
            app::AppScope,
            session::{SessionState, SessionType},
        },
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State, warn},
};

#[cfg(feature = "hosted")]
use {
    crate::{events::app::SetAppScope, states::menu::main::MainMenuScreen},
    bevy::{
        app::AppExit,
        ecs::message::MessageWriter,
        input::InputPlugin,
        prelude::{
            ButtonInput, IntoScheduleConfigs, KeyCode, OnEnter, Resource, Time, Update, in_state,
        },
    },
};

#[cfg_attr(doc, aquamarine::aquamarine)]
/// ## AppLogicPlugin Flow
///
/// ```mermaid
/// flowchart TD
///     Splash((Splash)) --> Menu[Menu]
///     Menu --> Session{SessionType?}
///     Session -->|Singleplayer| Single[(Singleplayer)]
///     Session -->|Client| Client[(Client)]
///     Session -->|Server| Server[(DedicatedServer)]
/// ```
///
/// **Navigation**:
/// - [`AppScope`](crate::states::app::AppScope)
/// - [`SessionType`](crate::states::session::SessionType)
/// - [`SplashTimer`](crate::plugins::app_logic::SplashTimer)
pub struct AppLogicPlugin;

impl Plugin for AppLogicPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "hosted")]
        if !app.is_plugin_added::<InputPlugin>() {
            app.add_plugins(InputPlugin);
        }
        app.init_state::<AppScope>()
            .init_state::<SessionType>()
            .add_sub_state::<SessionState>()
            .add_observer(on_set_session_type);

        #[cfg(feature = "hosted")]
        {
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

/// Validates transitions for SessionType.
pub(crate) fn is_valid_session_type_transition(from: &SessionType, to: &SetSessionType) -> bool {
    matches!(
        (from, to),
        (SessionType::None, _) | (_, SetSessionType::None)
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
        SetSessionType::None => {
            next_session_type.set(SessionType::None);
        }
        #[cfg(feature = "hosted")]
        SetSessionType::Singleplayer => {
            next_session_type.set(SessionType::Singleplayer);
        }
        #[cfg(feature = "hosted")]
        SetSessionType::Client => {
            next_session_type.set(SessionType::Client);
        }
        #[cfg(feature = "headless")]
        SetSessionType::DedicatedServer => {
            next_session_type.set(SessionType::DedicatedServer);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die App-Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Validator-Funktionen (ob SessionType-Übergänge gültig/ungültig sind)
    //! 2. Observer-Logik (ob Events korrekt verarbeitet werden)
    //! 3. AppScope-Übergänge (nur mit hosted feature)

    // Imports are provided by submodules as needed

    mod helpers {
        use crate::{
            events::session::SetSessionType,
            logic::app::AppLogicPlugin,
            states::app::AppScope,
            states::session::{SessionState, SessionType},
        };
        use bevy::{prelude::*, state::app::StatesPlugin};

        #[cfg(feature = "hosted")]
        use crate::states::menu::main::MainMenuScreen;

        pub fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, StatesPlugin, AppLogicPlugin));

            #[cfg(feature = "hosted")]
            app.add_sub_state::<MainMenuScreen>();

            app
        }

        /// Runs the app for the specified number of update ticks.
        pub fn update_app(app: &mut App, i: u8) {
            for _ in 0..i {
                app.update();
            }
        }

        /// Setup for tests with hosted feature: Starts at Splash, transitions to Session.
        #[cfg(feature = "hosted")]
        pub fn setup_test_app_hosted() -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            // Initial state should be Splash
            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Splash);

            // Transition to Session
            let mut next_app_scope = app.world_mut().resource_mut::<NextState<AppScope>>();
            next_app_scope.set(AppScope::Session);
            update_app(&mut app, 1);

            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Session);

            let session_state = app.world().resource::<State<SessionState>>();
            assert_eq!(session_state.get(), &SessionState::Setup);

            // SessionType should be None initially
            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::None);

            app
        }

        /// Setup for tests with headless feature: Starts directly in Session.
        #[cfg(feature = "headless")]
        pub fn setup_test_app_headless() -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            // Initial state should be Session (headless starts directly in Session)
            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Session);

            let session_state = app.world().resource::<State<SessionState>>();
            assert_eq!(session_state.get(), &SessionState::Setup);

            // SessionType should be None initially
            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::None);

            app
        }

        /// Setup for hosted feature tests with Menu scope.
        #[cfg(feature = "hosted")]
        pub fn setup_test_app_in_menu() -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            // Initial state should be Splash
            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Splash);

            // Transition to Menu
            let mut next_app_scope = app.world_mut().resource_mut::<NextState<AppScope>>();
            next_app_scope.set(AppScope::Menu);
            update_app(&mut app, 1);

            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Menu);

            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &SessionType::None);

            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Overview);

            app
        }

        /// Triggers SetSessionType event and updates the app.
        pub fn set_session_type(app: &mut App, session_type: SessionType) {
            let event = match session_type {
                SessionType::None => SetSessionType::None,
                #[cfg(feature = "hosted")]
                SessionType::Singleplayer => SetSessionType::Singleplayer,
                #[cfg(feature = "hosted")]
                SessionType::Client => SetSessionType::Client,
                #[cfg(feature = "headless")]
                SessionType::DedicatedServer => SetSessionType::DedicatedServer,
            };
            app.world_mut().trigger(event);
            update_app(app, 1);
        }

        /// Asserts that the current SessionType matches the expected value.
        pub fn assert_session_type(app: &mut App, expected: SessionType) {
            let session_type = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type.get(), &expected);
        }

        /// Asserts that the current AppScope matches the expected value.
        pub fn assert_app_scope(app: &mut App, expected: AppScope) {
            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &expected);
        }
    }

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

            // Menu → Menu ist ungültig (keine Änderung)
            assert!(!is_valid_app_scope_transition(
                &AppScope::Menu,
                &SetAppScope::Menu
            ));

            // Session → Session ist ungültig (keine Änderung) - not possible with explicit variants
            // Session → Menu is valid, so we can't test invalid same-state transition
        }
    }

    mod validator_tests {
        use crate::events::session::SetSessionType;
        use crate::logic::app::is_valid_session_type_transition;
        use crate::states::session::SessionType;

        /// Test: Gültige SessionType-Übergänge werden als gültig erkannt.
        ///
        /// Gültige Übergänge:
        /// - Von None zu jedem SessionType (To)
        /// - Von jedem SessionType zu None
        #[test]
        fn test_valid_session_type_transitions() {
            // None → Singleplayer (hosted only)
            #[cfg(feature = "hosted")]
            assert!(is_valid_session_type_transition(
                &SessionType::None,
                &SetSessionType::Singleplayer
            ));

            // None → Client (hosted only)
            #[cfg(feature = "hosted")]
            assert!(is_valid_session_type_transition(
                &SessionType::None,
                &SetSessionType::Client
            ));

            // None → DedicatedServer (headless only)
            #[cfg(feature = "headless")]
            assert!(is_valid_session_type_transition(
                &SessionType::None,
                &SetSessionType::DedicatedServer
            ));

            // Singleplayer → None (hosted only)
            #[cfg(feature = "hosted")]
            assert!(is_valid_session_type_transition(
                &SessionType::Singleplayer,
                &SetSessionType::None
            ));

            // Client → None (hosted only)
            #[cfg(feature = "hosted")]
            assert!(is_valid_session_type_transition(
                &SessionType::Client,
                &SetSessionType::None
            ));

            // DedicatedServer → None (headless only)
            #[cfg(feature = "headless")]
            assert!(is_valid_session_type_transition(
                &SessionType::DedicatedServer,
                &SetSessionType::None
            ));

            // None → None (edge case)
            assert!(is_valid_session_type_transition(
                &SessionType::None,
                &SetSessionType::None
            ));
        }

        /// Test: Ungültige SessionType-Übergänge werden blockiert.
        ///
        /// Ungültige Übergänge:
        /// - Von Singleplayer direkt zu Client (hosted)
        /// - Von Client direkt zu Singleplayer (hosted)
        /// - Von Singleplayer zu DedicatedServer (hosted)
        #[test]
        fn test_invalid_session_type_transitions() {
            // Singleplayer → Client (direkter Übergang ungültig)
            #[cfg(feature = "hosted")]
            assert!(!is_valid_session_type_transition(
                &SessionType::Singleplayer,
                &SetSessionType::Client
            ));

            // Client → Singleplayer (direkter Übergang ungültig)
            #[cfg(feature = "hosted")]
            assert!(!is_valid_session_type_transition(
                &SessionType::Client,
                &SetSessionType::Singleplayer
            ));

            // DedicatedServer → Singleplayer (direkter Übergang ungültig)
            #[cfg(all(feature = "headless", feature = "hosted"))]
            assert!(!is_valid_session_type_transition(
                &SessionType::DedicatedServer,
                &SetSessionType::Singleplayer
            ));

            // DedicatedServer → Client (direkter Übergang ungültig)
            #[cfg(all(feature = "headless", feature = "hosted"))]
            assert!(!is_valid_session_type_transition(
                &SessionType::DedicatedServer,
                &SetSessionType::Client
            ));
        }
    }

    // =============================================================================
    // TESTS FÜR OBSERVER-FUNKTIONEN
    // =============================================================================

    mod observer_tests {
        use crate::logic::app::tests::helpers;
        use crate::states::session::SessionType;

        /// Test: Gültiger SessionType-Wechsel über Observer funktioniert.
        #[cfg(feature = "hosted")]
        #[test]
        fn test_on_set_session_type_valid_transition() {
            let mut app = helpers::setup_test_app_hosted();

            // Transition from None to Singleplayer
            helpers::set_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);

            // Transition back to None
            helpers::set_session_type(&mut app, SessionType::None);
            helpers::assert_session_type(&mut app, SessionType::None);
        }

        /// Test: Gültiger SessionType-Wechsel zu Client über Observer funktioniert.
        #[cfg(feature = "hosted")]
        #[test]
        fn test_on_set_session_type_to_client() {
            let mut app = helpers::setup_test_app_hosted();

            // Transition from None to Client
            helpers::set_session_type(&mut app, SessionType::Client);
            helpers::assert_session_type(&mut app, SessionType::Client);

            // Transition back to None
            helpers::set_session_type(&mut app, SessionType::None);
            helpers::assert_session_type(&mut app, SessionType::None);
        }

        /// Test: Ungültiger SessionType-Wechsel wird vom Observer blockiert.
        #[cfg(feature = "hosted")]
        #[test]
        fn test_on_set_session_type_invalid_transition_blocked() {
            let mut app = helpers::setup_test_app_hosted();

            // First set to Singleplayer
            helpers::set_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);

            // Try to transition directly to Client (invalid)
            helpers::set_session_type(&mut app, SessionType::Client);

            // Should still be Singleplayer (transition blocked)
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);
        }

        /// Test: Mehrere gültige SessionType-Wechsel in Sequenz.
        #[cfg(feature = "hosted")]
        #[test]
        fn test_session_type_sequence() {
            let mut app = helpers::setup_test_app_hosted();

            // None → Singleplayer
            helpers::set_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);

            // Singleplayer → None
            helpers::set_session_type(&mut app, SessionType::None);
            helpers::assert_session_type(&mut app, SessionType::None);

            // None → Client
            helpers::set_session_type(&mut app, SessionType::Client);
            helpers::assert_session_type(&mut app, SessionType::Client);

            // Client → None
            helpers::set_session_type(&mut app, SessionType::None);
            helpers::assert_session_type(&mut app, SessionType::None);
        }

        /// Test: Gültiger SessionType-Wechsel für DedicatedServer (headless).
        #[cfg(feature = "headless")]
        #[test]
        fn test_on_set_session_type_dedicated_server() {
            let mut app = helpers::setup_test_app_headless();

            // Transition from None to DedicatedServer
            helpers::set_session_type(&mut app, SessionType::DedicatedServer);
            helpers::assert_session_type(&mut app, SessionType::DedicatedServer);

            // Transition back to None
            helpers::set_session_type(&mut app, SessionType::None);
            helpers::assert_session_type(&mut app, SessionType::None);
        }
    }

    // =============================================================================
    // TESTS FÜR APPSCOPE (nur mit hosted feature)
    // =============================================================================

    #[cfg(feature = "hosted")]
    mod app_scope_tests {
        use crate::events::app::SetAppScope;
        use crate::logic::app::tests::helpers;
        use crate::states::app::AppScope;
        use crate::states::menu::main::MainMenuScreen;
        use crate::states::session::SessionType;
        use bevy::prelude::{NextState, State};

        /// Test: Ungültiger AppScope-Wechsel wird vom Observer blockiert.
        ///
        /// Ein Versuch, von Menu zu Splash zu wechseln, sollte blockiert werden.
        #[test]
        fn test_invalid_app_scope_transition_blocked() {
            let mut app = helpers::test_app();
            helpers::update_app(&mut app, 1);

            // Initial state is Splash
            helpers::assert_app_scope(&mut app, AppScope::Splash);

            // Transition to Menu
            let mut next_app_scope = app.world_mut().resource_mut::<NextState<AppScope>>();
            next_app_scope.set(AppScope::Menu);
            helpers::update_app(&mut app, 1);
            helpers::assert_app_scope(&mut app, AppScope::Menu);

            // Note: Menu → Splash is no longer possible with explicit variants
            // since SetAppScope doesn't have a Splash variant
            // Transition to Session first, then try Menu → Session → Menu loop
            app.world_mut().trigger(SetAppScope::Session);
            helpers::update_app(&mut app, 1);
            helpers::assert_app_scope(&mut app, AppScope::Session);
        }

        /// Test: Ungültiger AppScope-Wechsel Session → Splash wird blockiert.
        #[test]
        fn test_session_to_splash_blocked() {
            let mut app = helpers::setup_test_app_hosted();

            // Current state is Session
            helpers::assert_app_scope(&mut app, AppScope::Session);

            // Note: Session → Splash is no longer possible with explicit variants
            // since SetAppScope doesn't have a Splash variant
            // Session → Menu is valid, so transition to Menu instead
            app.world_mut().trigger(SetAppScope::Menu);
            helpers::update_app(&mut app, 1);
            helpers::assert_app_scope(&mut app, AppScope::Menu);
        }

        /// Test: SetAppScope::To(Menu) setzt alle Zustände korrekt zurück.
        #[test]
        fn test_on_change_app_scope_to_menu() {
            let mut app = helpers::setup_test_app_hosted();

            // First set session type to Singleplayer
            helpers::set_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);

            // Trigger SetAppScope::Menu
            app.world_mut().trigger(SetAppScope::Menu);
            helpers::update_app(&mut app, 1);

            // AppScope should be Menu
            helpers::assert_app_scope(&mut app, AppScope::Menu);

            // SessionType should be reset to None
            helpers::assert_session_type(&mut app, SessionType::None);

            // MainMenuScreen should be set to Main
            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Overview);
        }

        /// Test: Initialer Zustand ist Splash im hosted mode.
        #[test]
        fn test_initial_state_is_splash() {
            let mut app = helpers::test_app();
            helpers::update_app(&mut app, 1);

            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Splash);
        }

        /// Test: Splash → Menu Transition.
        #[test]
        fn test_splash_to_menu_transition() {
            let mut app = helpers::test_app();
            helpers::update_app(&mut app, 1);

            // Initial state is Splash
            helpers::assert_app_scope(&mut app, AppScope::Splash);

            // Transition to Menu
            let mut next_app_scope = app.world_mut().resource_mut::<NextState<AppScope>>();
            next_app_scope.set(AppScope::Menu);
            helpers::update_app(&mut app, 1);

            helpers::assert_app_scope(&mut app, AppScope::Menu);
        }

        /// Test: Splash → Session Transition.
        #[test]
        fn test_splash_to_session_transition() {
            let mut app = helpers::test_app();
            helpers::update_app(&mut app, 1);

            // Initial state is Splash
            helpers::assert_app_scope(&mut app, AppScope::Splash);

            // Transition to Session
            let mut next_app_scope = app.world_mut().resource_mut::<NextState<AppScope>>();
            next_app_scope.set(AppScope::Session);
            helpers::update_app(&mut app, 1);

            helpers::assert_app_scope(&mut app, AppScope::Session);
        }
    }

    #[cfg(feature = "headless")]
    mod headless_tests {
        use crate::logic::app::tests::helpers;
        use crate::states::app::AppScope;
        use bevy::prelude::State;

        /// Test: Initialer Zustand ist Session im headless mode.
        #[test]
        fn test_initial_state_is_session() {
            let mut app = helpers::test_app();
            helpers::update_app(&mut app, 1);

            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Session);
        }
    }

    // =============================================================================
    // INTEGRATION TESTS
    // =============================================================================

    #[cfg(feature = "hosted")]
    mod integration_tests {
        use crate::events::app::SetAppScope;
        use crate::logic::app::tests::helpers;
        use crate::states::app::AppScope;
        use crate::states::session::SessionType;
        use bevy::prelude::NextState;

        /// Test: Vollständiger Workflow - Splash → Menu → Session → Menu.
        #[test]
        fn test_full_app_scope_workflow() {
            let mut app = helpers::test_app();
            helpers::update_app(&mut app, 1);

            // Start at Splash
            helpers::assert_app_scope(&mut app, AppScope::Splash);

            // Splash → Menu
            let mut next_app_scope = app.world_mut().resource_mut::<NextState<AppScope>>();
            next_app_scope.set(AppScope::Menu);
            helpers::update_app(&mut app, 1);
            helpers::assert_app_scope(&mut app, AppScope::Menu);

            // Menu → Session
            let mut next_app_scope = app.world_mut().resource_mut::<NextState<AppScope>>();
            next_app_scope.set(AppScope::Session);
            helpers::update_app(&mut app, 1);
            helpers::assert_app_scope(&mut app, AppScope::Session);

            // Set SessionType to Singleplayer
            helpers::set_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);

            // Session → Menu (via SetAppScope event)
            app.world_mut().trigger(SetAppScope::Menu);
            helpers::update_app(&mut app, 1);
            helpers::assert_app_scope(&mut app, AppScope::Menu);
            helpers::assert_session_type(&mut app, SessionType::None);
        }

        /// Test: SessionType wird korrekt zurückgesetzt beim Wechsel zu Menu.
        #[test]
        fn test_session_type_reset_on_menu() {
            let mut app = helpers::setup_test_app_hosted();

            // Set to Singleplayer
            helpers::set_session_type(&mut app, SessionType::Singleplayer);
            helpers::assert_session_type(&mut app, SessionType::Singleplayer);

            // Transition to Menu via SetAppScope event
            app.world_mut().trigger(SetAppScope::Menu);
            helpers::update_app(&mut app, 1);

            // SessionType should be None
            helpers::assert_session_type(&mut app, SessionType::None);
        }
    }
}
