use {
    crate::{
        events::menu::settings::SetSettingsMenu,
        states::menu::{main::MainMenuScreen, settings::SettingsMenuScreen},
    },
    bevy::prelude::{warn, App, AppExtStates, NextState, On, Plugin, Res, ResMut, State},
};

pub(super) struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SettingsMenuScreen>()
            .add_observer(on_set_settings_menu);
    }
}

// --- VALIDATOR FUNCTIONS ---

/// Validates transitions from MainMenuScreen to SettingsMenuScreen states.
pub(crate) fn is_valid_main_menu_screen_settings_transition(
    from: &MainMenuScreen,
    to: &SetSettingsMenu,
) -> bool {
    match (from, to) {
        // From Overview: can go to Settings (Overview event)
        (MainMenuScreen::Overview, SetSettingsMenu::Overview) => true,
        // From Settings: can navigate within SettingsMenu
        (
            MainMenuScreen::Settings,
            SetSettingsMenu::Overview
            | SetSettingsMenu::Audio
            | SetSettingsMenu::Video
            | SetSettingsMenu::Controls
            | SetSettingsMenu::Back
            | SetSettingsMenu::Apply
            | SetSettingsMenu::Cancel,
        ) => true,
        _ => false,
    }
}

/// Validates transitions between SettingsMenuScreen states.
/// Navigation to other categories is only allowed from Overview.
pub(crate) fn is_valid_settings_menu_screen_transition(
    from: &SettingsMenuScreen,
    to: &SetSettingsMenu,
) -> bool {
    match (from, to) {
        // From Overview: can go to Audio, Video, Controls, Back, Apply, Cancel
        (SettingsMenuScreen::Overview, SetSettingsMenu::Audio) => true,
        (SettingsMenuScreen::Overview, SetSettingsMenu::Video) => true,
        (SettingsMenuScreen::Overview, SetSettingsMenu::Controls) => true,
        (SettingsMenuScreen::Overview, SetSettingsMenu::Back) => true,
        (SettingsMenuScreen::Overview, SetSettingsMenu::Apply) => true,
        (SettingsMenuScreen::Overview, SetSettingsMenu::Cancel) => true,

        // From Audio/Video/Controls: only Back, Apply, Cancel allowed
        // Back goes to Overview, not directly to other categories
        (SettingsMenuScreen::Audio, SetSettingsMenu::Back) => true,
        (SettingsMenuScreen::Audio, SetSettingsMenu::Apply) => true,
        (SettingsMenuScreen::Audio, SetSettingsMenu::Cancel) => true,

        (SettingsMenuScreen::Video, SetSettingsMenu::Back) => true,
        (SettingsMenuScreen::Video, SetSettingsMenu::Apply) => true,
        (SettingsMenuScreen::Video, SetSettingsMenu::Cancel) => true,

        (SettingsMenuScreen::Controls, SetSettingsMenu::Back) => true,
        (SettingsMenuScreen::Controls, SetSettingsMenu::Apply) => true,
        (SettingsMenuScreen::Controls, SetSettingsMenu::Cancel) => true,

        // All other transitions are invalid
        _ => false,
    }
}

// --- OBSERVER FUNCTIONS ---

/// Handles SetSettingsMenu events.
/// Validates parent and sub-state transitions before applying state changes.
fn on_set_settings_menu(
    event: On<SetSettingsMenu>,
    current_parent: Res<State<MainMenuScreen>>,
    current: Option<Res<State<SettingsMenuScreen>>>,
    mut next_main_screen: ResMut<NextState<MainMenuScreen>>,
    mut next_settings_screen: ResMut<NextState<SettingsMenuScreen>>,
) {
    // Validate parent state transition
    if !is_valid_main_menu_screen_settings_transition(current_parent.get(), event.event()) {
        warn!(
            "Invalid MainMenuScreen transition for SetSettingsMenu: {:?} with parent {:?}",
            event.event(),
            current_parent.get()
        );
        return;
    }

    match *event.event() {
        // Back: Return to MainMenuScreen::Overview (only valid from SettingsMenuScreen::Overview)
        SetSettingsMenu::Back => {
            // Validate that we're in Overview substate
            if let Some(ref current_state) = current {
                if !is_valid_settings_menu_screen_transition(current_state.get(), event.event()) {
                    warn!(
                        "Invalid SettingsMenuScreen transition for Back: {:?} -> {:?}",
                        current_state.get(),
                        event.event()
                    );
                    return;
                }
            } else {
                warn!("SettingsMenuScreen does not exist - cannot process Back event");
                return;
            }
            next_main_screen.set(MainMenuScreen::Overview);
        }
        // Overview: Switch parent to Settings (substate is initialized automatically with default Overview)
        SetSettingsMenu::Overview => {
            next_main_screen.set(MainMenuScreen::Settings);
        }
        // Audio/Video/Controls: Set the substate (parent must already be Settings)
        SetSettingsMenu::Audio | SetSettingsMenu::Video | SetSettingsMenu::Controls => {
            // Validate substate transition if we have a current state
            if let Some(ref current_state) = current {
                if !is_valid_settings_menu_screen_transition(current_state.get(), event.event()) {
                    warn!(
                        "Invalid SettingsMenuScreen transition: {:?} -> {:?}",
                        current_state.get(),
                        event.event()
                    );
                    return;
                }
            }

            match *event.event() {
                SetSettingsMenu::Audio => next_settings_screen.set(SettingsMenuScreen::Audio),
                SetSettingsMenu::Video => next_settings_screen.set(SettingsMenuScreen::Video),
                SetSettingsMenu::Controls => next_settings_screen.set(SettingsMenuScreen::Controls),
                _ => {}
            }
        }
        // Apply/Cancel: Handle actions with substate validation
        SetSettingsMenu::Apply | SetSettingsMenu::Cancel => {
            // Validate substate transition if we have a current state
            if let Some(ref current_state) = current {
                if !is_valid_settings_menu_screen_transition(current_state.get(), event.event()) {
                    warn!(
                        "Invalid SettingsMenuScreen transition: {:?} -> {:?}",
                        current_state.get(),
                        event.event()
                    );
                    return;
                }
            } else {
                warn!("SettingsMenuScreen does not exist - cannot process Apply/Cancel event");
                return;
            }

            match *event.event() {
                SetSettingsMenu::Apply => {
                    // TODO: Apply settings logic
                    // For now, just stay in current state
                }
                SetSettingsMenu::Cancel => {
                    // Cancel always goes back to Overview (not MainMenu!)
                    next_settings_screen.set(SettingsMenuScreen::Overview);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die Settings Menu Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Validator-Funktionen (ob Übergänge gültig/ungültig sind)
    //! 2. Observer-Logik (ob Events korrekt verarbeitet werden)
    //! 3. Integration-Tests (komplette Workflows)

    use crate::events::menu::settings::SetSettingsMenu;
    use crate::states::menu::{main::MainMenuScreen, settings::SettingsMenuScreen};

    mod validator_tests {
        use super::*;
        use crate::logic::menu::settings::{
            is_valid_main_menu_screen_settings_transition, is_valid_settings_menu_screen_transition,
        };

        /// Test: Gültige MainMenuScreen::Settings-Übergänge werden akzeptiert.
        #[test]
        fn test_valid_main_menu_screen_settings_transitions() {
            // From Settings: all events are valid
            assert!(is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Settings,
                &SetSettingsMenu::Overview
            ));
            assert!(is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Settings,
                &SetSettingsMenu::Audio
            ));
            assert!(is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Settings,
                &SetSettingsMenu::Video
            ));
            assert!(is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Settings,
                &SetSettingsMenu::Controls
            ));
            assert!(is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Settings,
                &SetSettingsMenu::Back
            ));
            assert!(is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Settings,
                &SetSettingsMenu::Apply
            ));
            assert!(is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Settings,
                &SetSettingsMenu::Cancel
            ));
        }

        /// Test: Valid transition from MainMenuScreen::Overview to Settings.
        #[test]
        fn test_valid_main_menu_overview_to_settings_transition() {
            // Overview -> Overview (switches parent to Settings, substate initialized automatically)
            assert!(is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Overview,
                &SetSettingsMenu::Overview
            ));
        }

        /// Test: Ungültige MainMenuScreen-Übergänge werden blockiert.
        #[test]
        fn test_invalid_main_menu_screen_settings_transitions() {
            // Overview -> Audio/Video/Controls/Back/Apply/Cancel is invalid (must go to Overview first to switch parent)
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Overview,
                &SetSettingsMenu::Audio
            ));
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Overview,
                &SetSettingsMenu::Video
            ));
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Overview,
                &SetSettingsMenu::Controls
            ));
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Overview,
                &SetSettingsMenu::Back
            ));
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Overview,
                &SetSettingsMenu::Apply
            ));
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Overview,
                &SetSettingsMenu::Cancel
            ));

            // Singleplayer -> Any SetSettingsMenu event is invalid
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Singleplayer,
                &SetSettingsMenu::Overview
            ));
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Singleplayer,
                &SetSettingsMenu::Audio
            ));

            // Multiplayer -> Any SetSettingsMenu event is invalid
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Multiplayer,
                &SetSettingsMenu::Overview
            ));
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Multiplayer,
                &SetSettingsMenu::Back
            ));

            // Wiki -> Any SetSettingsMenu event is invalid
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Wiki,
                &SetSettingsMenu::Overview
            ));
            assert!(!is_valid_main_menu_screen_settings_transition(
                &MainMenuScreen::Wiki,
                &SetSettingsMenu::Apply
            ));
        }

        /// Test: Gültige SettingsMenuScreen-Übergänge von Overview.
        #[test]
        fn test_valid_settings_transitions_from_overview() {
            // From Overview: can go to Audio, Video, Controls, Back, Apply, Cancel
            assert!(is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Overview,
                &SetSettingsMenu::Audio
            ));
            assert!(is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Overview,
                &SetSettingsMenu::Video
            ));
            assert!(is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Overview,
                &SetSettingsMenu::Controls
            ));
            assert!(is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Overview,
                &SetSettingsMenu::Back
            ));
            assert!(is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Overview,
                &SetSettingsMenu::Apply
            ));
            assert!(is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Overview,
                &SetSettingsMenu::Cancel
            ));
        }

        /// Test: Ungültige direkte Übergänge zwischen Kategorien.
        #[test]
        fn test_invalid_direct_category_transitions() {
            // From Audio: cannot go directly to Video or Controls
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Audio,
                &SetSettingsMenu::Video
            ));
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Audio,
                &SetSettingsMenu::Controls
            ));
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Audio,
                &SetSettingsMenu::Overview
            ));

            // From Video: cannot go directly to Audio or Controls
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Video,
                &SetSettingsMenu::Audio
            ));
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Video,
                &SetSettingsMenu::Controls
            ));
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Video,
                &SetSettingsMenu::Overview
            ));

            // From Controls: cannot go directly to Audio or Video
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Controls,
                &SetSettingsMenu::Audio
            ));
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Controls,
                &SetSettingsMenu::Video
            ));
            assert!(!is_valid_settings_menu_screen_transition(
                &SettingsMenuScreen::Controls,
                &SetSettingsMenu::Overview
            ));
        }

        /// Test: Gültige Übergänge von Audio/Video/Controls.
        #[test]
        fn test_valid_transitions_from_categories() {
            // From Audio/Video/Controls: only Back, Apply, Cancel allowed
            let categories = [
                SettingsMenuScreen::Audio,
                SettingsMenuScreen::Video,
                SettingsMenuScreen::Controls,
            ];

            for category in &categories {
                assert!(
                    is_valid_settings_menu_screen_transition(category, &SetSettingsMenu::Back),
                    "Back should be valid from {:?}",
                    category
                );
                assert!(
                    is_valid_settings_menu_screen_transition(category, &SetSettingsMenu::Apply),
                    "Apply should be valid from {:?}",
                    category
                );
                assert!(
                    is_valid_settings_menu_screen_transition(category, &SetSettingsMenu::Cancel),
                    "Cancel should be valid from {:?}",
                    category
                );
            }
        }
    }

    mod observer_tests {
        use super::*;

        pub mod helpers {
            use crate::{
                events::{app::SetAppScope, menu::settings::SetSettingsMenu},
                states::{
                    app::AppScope,
                    menu::{main::MainMenuScreen, settings::SettingsMenuScreen},
                },
                ChickenStatePlugin,
            };
            use bevy::{prelude::*, state::app::StatesPlugin};

            /// Creates a test app with all required plugins.
            pub fn test_app() -> App {
                let mut app = App::new();
                app.add_plugins((MinimalPlugins, StatesPlugin, ChickenStatePlugin));
                app
            }

            /// Runs the app for a specified number of update ticks.
            pub fn update_app(app: &mut App, ticks: u8) {
                for _ in 0..ticks {
                    app.update();
                }
            }

            /// Setup helper: Sets MainMenuScreen to Settings and SettingsMenuScreen to Overview.
            pub fn setup_test_app_in_settings_overview() -> App {
                let mut app = test_app();
                update_app(&mut app, 1);

                // Verify initial state
                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Splash);

                // Transition to Menu
                app.world_mut().trigger(SetAppScope::Menu);
                update_app(&mut app, 1);

                // Verify we're in MainMenu::Overview
                let menu_state = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(menu_state.get(), &MainMenuScreen::Overview);

                // Navigate to Settings
                app.world_mut().trigger(SetSettingsMenu::Overview);
                update_app(&mut app, 1);

                // Verify we're in Settings
                let menu_state = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(menu_state.get(), &MainMenuScreen::Settings);

                // Verify we're in SettingsMenuScreen::Overview
                let settings_state = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(settings_state.get(), &SettingsMenuScreen::Overview);

                app
            }

            /// Setup helper: Sets SettingsMenuScreen to Audio.
            pub fn setup_test_app_in_audio() -> App {
                let mut app = setup_test_app_in_settings_overview();

                // Navigate to Audio
                app.world_mut().trigger(SetSettingsMenu::Audio);
                update_app(&mut app, 1);

                let settings_state = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(settings_state.get(), &SettingsMenuScreen::Audio);

                app
            }

            /// Setup helper: Sets SettingsMenuScreen to Video.
            pub fn setup_test_app_in_video() -> App {
                let mut app = setup_test_app_in_settings_overview();

                // Navigate to Video
                app.world_mut().trigger(SetSettingsMenu::Video);
                update_app(&mut app, 1);

                let settings_state = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(settings_state.get(), &SettingsMenuScreen::Video);

                app
            }

            /// Setup helper: Sets SettingsMenuScreen to Controls.
            pub fn setup_test_app_in_controls() -> App {
                let mut app = setup_test_app_in_settings_overview();

                // Navigate to Controls
                app.world_mut().trigger(SetSettingsMenu::Controls);
                update_app(&mut app, 1);

                let settings_state = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(settings_state.get(), &SettingsMenuScreen::Controls);

                app
            }

            /// Asserts that SettingsMenuScreen state matches expected value.
            pub fn assert_settings_screen(app: &mut App, expected: SettingsMenuScreen) {
                let settings_state = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(settings_state.get(), &expected);
            }

            /// Asserts that MainMenuScreen state matches expected value.
            pub fn assert_main_menu_screen(app: &mut App, expected: MainMenuScreen) {
                let menu_state = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(menu_state.get(), &expected);
            }
        }

        /// Test: Overview -> Audio transition works.
        #[test]
        fn test_observer_overview_to_audio() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            app.world_mut().trigger(SetSettingsMenu::Audio);
            helpers::update_app(&mut app, 1);

            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
        }

        /// Test: Overview -> Video transition works.
        #[test]
        fn test_observer_overview_to_video() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            app.world_mut().trigger(SetSettingsMenu::Video);
            helpers::update_app(&mut app, 1);

            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Video);
        }

        /// Test: Overview -> Controls transition works.
        #[test]
        fn test_observer_overview_to_controls() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            app.world_mut().trigger(SetSettingsMenu::Controls);
            helpers::update_app(&mut app, 1);

            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Controls);
        }

        /// Test: Audio -> Back -> Main Menu transition works.
        #[test]
        fn test_observer_audio_back_to_main_menu() {
            let mut app = helpers::setup_test_app_in_audio();

            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Video -> Back -> Main Menu transition works.
        #[test]
        fn test_observer_video_back_to_main_menu() {
            let mut app = helpers::setup_test_app_in_video();

            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Controls -> Back -> Main Menu transition works.
        #[test]
        fn test_observer_controls_back_to_main_menu() {
            let mut app = helpers::setup_test_app_in_controls();

            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Overview -> Back -> MainMenu transition works.
        #[test]
        fn test_observer_overview_back_to_main_menu() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);

            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Invalid direct category navigation is blocked (Audio -> Video).
        #[test]
        fn test_observer_invalid_category_switch_blocked() {
            let mut app = helpers::setup_test_app_in_audio();

            // Try to go directly from Audio to Video (should be ignored)
            app.world_mut().trigger(SetSettingsMenu::Video);
            helpers::update_app(&mut app, 1);

            // Should still be in Audio
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
        }

        /// Test: Apply event works from Audio state.
        #[test]
        fn test_observer_apply_from_audio() {
            let mut app = helpers::setup_test_app_in_audio();

            // Apply should stay in current state (currently a no-op)
            app.world_mut().trigger(SetSettingsMenu::Apply);
            helpers::update_app(&mut app, 1);

            // Should still be in Audio (Apply is a no-op)
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
        }

        /// Test: Cancel from Video returns to Overview.
        #[test]
        fn test_observer_cancel_returns_to_overview() {
            let mut app = helpers::setup_test_app_in_video();

            app.world_mut().trigger(SetSettingsMenu::Cancel);
            helpers::update_app(&mut app, 1);

            // Cancel always returns to Overview
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);
        }

        /// Test: Apply event works from Video state.
        #[test]
        fn test_observer_apply_from_video() {
            let mut app = helpers::setup_test_app_in_video();

            app.world_mut().trigger(SetSettingsMenu::Apply);
            helpers::update_app(&mut app, 1);

            // Should still be in Video
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Video);
        }

        /// Test: Apply event works from Controls state.
        #[test]
        fn test_observer_apply_from_controls() {
            let mut app = helpers::setup_test_app_in_controls();

            app.world_mut().trigger(SetSettingsMenu::Apply);
            helpers::update_app(&mut app, 1);

            // Should still be in Controls
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Controls);
        }

        /// Test: Cancel from Controls returns to Overview.
        #[test]
        fn test_observer_cancel_from_controls() {
            let mut app = helpers::setup_test_app_in_controls();

            app.world_mut().trigger(SetSettingsMenu::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);
        }

        /// Test: Cancel from Audio returns to Overview.
        #[test]
        fn test_observer_cancel_from_audio() {
            let mut app = helpers::setup_test_app_in_audio();

            app.world_mut().trigger(SetSettingsMenu::Cancel);
            helpers::update_app(&mut app, 1);

            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);
        }

        /// Test: Apply from Overview works (stays in Overview).
        #[test]
        fn test_observer_apply_from_overview() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            app.world_mut().trigger(SetSettingsMenu::Apply);
            helpers::update_app(&mut app, 1);

            // Should stay in Overview
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);
        }

        /// Test: Observer ignores invalid events with warning.
        #[test]
        fn test_observer_ignores_invalid_events() {
            let mut app = helpers::setup_test_app_in_audio();

            // Try to go directly to Overview from Audio (should be ignored)
            app.world_mut().trigger(SetSettingsMenu::Overview);
            helpers::update_app(&mut app, 1);

            // Should still be in Audio
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
        }
    }

    mod integration_tests {
        use super::*;

        mod helpers {
            pub use super::super::observer_tests::helpers::*;
        }

        /// Test: Overview -> Audio -> Back -> Overview.
        #[test]
        fn test_overview_to_audio_and_back() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            // Navigate to Audio
            app.world_mut().trigger(SetSettingsMenu::Audio);
            helpers::update_app(&mut app, 1);
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);

            // Go back to Overview
            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Overview -> Video -> Cancel -> Overview.
        #[test]
        fn test_overview_to_video_and_cancel() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            // Navigate to Video
            app.world_mut().trigger(SetSettingsMenu::Video);
            helpers::update_app(&mut app, 1);
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Video);

            // Cancel should return to Overview
            app.world_mut().trigger(SetSettingsMenu::Cancel);
            helpers::update_app(&mut app, 1);
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);
        }

        /// Test: Back from Overview goes to MainMenu.
        #[test]
        fn test_back_from_overview_goes_to_main_menu() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            // Back from Overview should go to MainMenu
            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Apply works from all states (Audio, Video, Controls).
        #[test]
        fn test_apply_from_all_states() {
            // Test Apply from Audio
            {
                let mut app = helpers::setup_test_app_in_audio();
                app.world_mut().trigger(SetSettingsMenu::Apply);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
            }

            // Test Apply from Video
            {
                let mut app = helpers::setup_test_app_in_video();
                app.world_mut().trigger(SetSettingsMenu::Apply);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Video);
            }

            // Test Apply from Controls
            {
                let mut app = helpers::setup_test_app_in_controls();
                app.world_mut().trigger(SetSettingsMenu::Apply);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Controls);
            }

            // Test Apply from Overview
            {
                let mut app = helpers::setup_test_app_in_settings_overview();
                app.world_mut().trigger(SetSettingsMenu::Apply);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);
            }
        }

        /// Test: Complete settings workflow - Overview -> Audio -> Apply -> Back -> Overview -> Video -> Back -> MainMenu.
        #[test]
        fn test_complete_settings_workflow() {
            let mut app = helpers::setup_test_app_in_settings_overview();

            // 1. Navigate to Audio
            app.world_mut().trigger(SetSettingsMenu::Audio);
            helpers::update_app(&mut app, 1);
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);

            // 2. Apply (stays in Audio)
            app.world_mut().trigger(SetSettingsMenu::Apply);
            helpers::update_app(&mut app, 1);
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);

            // 3. Back to Overview (must go back first before navigating to another category)
            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);

            // 4. Navigate to Settings again
            app.world_mut().trigger(SetSettingsMenu::Overview);
            helpers::update_app(&mut app, 1);
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Overview);

            // 5. Navigate to Video
            app.world_mut().trigger(SetSettingsMenu::Video);
            helpers::update_app(&mut app, 1);
            helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Video);

            // 6. Back to MainMenu
            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);
            helpers::assert_main_menu_screen(&mut app, MainMenuScreen::Overview);
        }

        /// Test: Invalid direct navigation between categories is blocked.
        #[test]
        fn test_invalid_direct_navigation_blocked() {
            // Test: Audio -> Video (should be blocked)
            {
                let mut app = helpers::setup_test_app_in_audio();
                app.world_mut().trigger(SetSettingsMenu::Video);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
            }

            // Test: Audio -> Controls (should be blocked)
            {
                let mut app = helpers::setup_test_app_in_audio();
                app.world_mut().trigger(SetSettingsMenu::Controls);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Audio);
            }

            // Test: Video -> Controls (should be blocked)
            {
                let mut app = helpers::setup_test_app_in_video();
                app.world_mut().trigger(SetSettingsMenu::Controls);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Video);
            }

            // Test: Video -> Audio (should be blocked)
            {
                let mut app = helpers::setup_test_app_in_video();
                app.world_mut().trigger(SetSettingsMenu::Audio);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Video);
            }

            // Test: Controls -> Audio (should be blocked)
            {
                let mut app = helpers::setup_test_app_in_controls();
                app.world_mut().trigger(SetSettingsMenu::Audio);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Controls);
            }

            // Test: Controls -> Video (should be blocked)
            {
                let mut app = helpers::setup_test_app_in_controls();
                app.world_mut().trigger(SetSettingsMenu::Video);
                helpers::update_app(&mut app, 1);
                helpers::assert_settings_screen(&mut app, SettingsMenuScreen::Controls);
            }
        }
    }
}
