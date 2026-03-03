use {
    crate::{events::menu::settings::SetSettingsMenu, states::menu::settings::SettingsMenuScreen},
    bevy::prelude::{warn, App, AppExtStates, NextState, On, Plugin, Res, ResMut, State},
};

pub(super) struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SettingsMenuScreen>()
            .add_observer(handle_settings_nav);
    }
}

// --- LOGIC HANDLERS ---

/// Validates transitions for SettingsMenuScreen.
pub(crate) fn is_valid_settings_screen_transition(
    _from: &SettingsMenuScreen,
    _to: &SetSettingsMenu,
) -> bool {
    // All transitions between SettingsMenuScreen variants are valid.
    // Apply, Back, and Cancel are always valid from any state.
    true
}

fn handle_settings_nav(
    trigger: On<SetSettingsMenu>,
    current: Option<Res<State<SettingsMenuScreen>>>,
    mut next_screen: ResMut<NextState<SettingsMenuScreen>>,
) {
    // Get current state
    let current = match current {
        Some(c) => *c.get(),
        None => {
            warn!("SettingsMenuScreen does not exist - MainMenuScreen must be Settings first");
            return;
        }
    };

    // Validate transition
    if !is_valid_settings_screen_transition(&current, trigger.event()) {
        warn!(
            "Invalid SettingsMenuScreen transition: {:?} -> {:?}",
            current,
            trigger.event()
        );
        return;
    }

    match trigger.event() {
        SetSettingsMenu::To(target) => {
            next_screen.set(*target);
        }
        SetSettingsMenu::Back => {
            if current != SettingsMenuScreen::Overview {
                next_screen.set(SettingsMenuScreen::Overview);
            }
        }
        SetSettingsMenu::Apply => {
            // TODO: Apply settings logic
        }
        SetSettingsMenu::Cancel => {
            next_screen.set(SettingsMenuScreen::Overview);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die Settings Menu Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Observer-Logik (ob SetSettingsMenus korrekt verarbeitet werden)
    //! 2. State-Übergänge (ob SettingsMenuScreen korrekt gewechselt wird)
    //! 3. Navigation zwischen Settings-Screens (Overview, Audio, Video, Controls)

    use crate::events::menu::settings::SetSettingsMenu;

    // =============================================================================
    // TESTS FÜR VALIDATOR-FUNKTIONEN
    // =============================================================================

    mod validator_tests {
        use crate::events::menu::settings::SetSettingsMenu;
        use crate::logic::menu::settings::is_valid_settings_screen_transition;
        use crate::states::menu::settings::SettingsMenuScreen;

        /// Test: Gültige SettingsMenuScreen-Übergänge werden als gültig erkannt.
        ///
        /// Alle Übergänge zwischen SettingsMenuScreen-Varianten sind gültig.
        /// Apply, Back und Cancel sind von jedem State gültig.
        #[test]
        fn test_valid_settings_screen_transitions() {
            // Overview → To(Audio) ist gültig
            assert!(is_valid_settings_screen_transition(
                &SettingsMenuScreen::Overview,
                &SetSettingsMenu::To(SettingsMenuScreen::Audio)
            ));

            // Audio → To(Video) ist gültig
            assert!(is_valid_settings_screen_transition(
                &SettingsMenuScreen::Audio,
                &SetSettingsMenu::To(SettingsMenuScreen::Video)
            ));

            // Video → Back ist gültig
            assert!(is_valid_settings_screen_transition(
                &SettingsMenuScreen::Video,
                &SetSettingsMenu::Back
            ));

            // Controls → Cancel ist gültig
            assert!(is_valid_settings_screen_transition(
                &SettingsMenuScreen::Controls,
                &SetSettingsMenu::Cancel
            ));

            // Overview → Apply ist gültig
            assert!(is_valid_settings_screen_transition(
                &SettingsMenuScreen::Overview,
                &SetSettingsMenu::Apply
            ));
        }

        /// Test: Alle Übergänge zwischen allen SettingsMenuScreen-Varianten sind gültig.
        ///
        /// Dieser Test prüft alle Kombinationen.
        #[test]
        fn test_all_screen_transitions_valid() {
            let screens = [
                SettingsMenuScreen::Overview,
                SettingsMenuScreen::Audio,
                SettingsMenuScreen::Video,
                SettingsMenuScreen::Controls,
            ];

            for from in &screens {
                for to in &screens {
                    assert!(
                        is_valid_settings_screen_transition(from, &SetSettingsMenu::To(*to)),
                        "Transition from {:?} to {:?} should be valid",
                        from,
                        to
                    );
                }
            }
        }

        /// Test: Apply, Back und Cancel sind von jedem Screen gültig.
        #[test]
        fn test_all_actions_valid_from_all_screens() {
            let screens = [
                SettingsMenuScreen::Overview,
                SettingsMenuScreen::Audio,
                SettingsMenuScreen::Video,
                SettingsMenuScreen::Controls,
            ];

            for screen in &screens {
                assert!(
                    is_valid_settings_screen_transition(screen, &SetSettingsMenu::Apply),
                    "Apply from {:?} should be valid",
                    screen
                );
                assert!(
                    is_valid_settings_screen_transition(screen, &SetSettingsMenu::Back),
                    "Back from {:?} should be valid",
                    screen
                );
                assert!(
                    is_valid_settings_screen_transition(screen, &SetSettingsMenu::Cancel),
                    "Cancel from {:?} should be valid",
                    screen
                );
            }
        }
    }

    mod helpers {
        use crate::{
            events::{app::SetAppScope, menu::main::SetMainMenu},
            logic::{app::AppLogicPlugin, menu::MenuPlugin},
            states::{
                app::AppScope,
                menu::{main::MainMenuScreen, settings::SettingsMenuScreen},
                session::SessionType,
            },
        };
        use bevy::{prelude::*, state::app::StatesPlugin};

        /// Erstellt eine Test-App mit SettingsMenuPlugin und AppLogicPlugin.
        ///
        /// AppLogicPlugin ist notwendig, um die Messaging-Infrastruktur zu initialisieren,
        /// die für State-Transitions benötigt wird.
        pub fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, StatesPlugin, AppLogicPlugin, MenuPlugin));
            app
        }

        /// Führt den App-Update für die angegebene Anzahl von Ticks aus.
        pub fn update_app(app: &mut App, i: u8) {
            for _ in 0..i {
                app.update();
            }
        }

        /// Setzt die App in den Settings-Context.
        ///
        /// Diese Funktion nutzt insert_state, um direkt den MainMenuScreen::Settings zu setzen,
        /// ohne die Event-Transition-Machinerie zu verwenden (vermeidet Messaging-Probleme).
        pub fn setup_settings_app() -> App {
            let mut app = test_app();
            update_app(&mut app, 1);

            let session_type_state = app.world().resource::<State<SessionType>>();
            assert_eq!(session_type_state.get(), &SessionType::None);

            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Splash);

            app.world_mut().trigger(SetAppScope::To(AppScope::Menu));
            update_app(&mut app, 1);

            // Verifiziere, dass wir im Menu sind
            let app_scope = app.world().resource::<State<AppScope>>();
            assert_eq!(app_scope.get(), &AppScope::Menu);

            // Verifiziere, dass MainMenuScreen existiert und auf Main steht
            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Overview);

            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Settings));
            update_app(&mut app, 1);

            // Verifiziere, dass wir im Settings sind
            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Settings);

            // Verifiziere, dass SettingsMenuScreen existiert und auf Overview steht
            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Overview);

            app
        }
    }

    // =============================================================================
    // TESTS FÜR SETTINGS MENU NAVIGATION OBSERVER
    // =============================================================================

    mod observer_tests {
        use super::*;
        use crate::states::menu::settings::SettingsMenuScreen;
        use bevy::prelude::*;

        /// Test: Navigate-Event ändert den SettingsMenuScreen.
        ///
        /// Ein Navigate-Event mit Audio sollte den Screen zu
        /// SettingsMenuScreen::Audio ändern.
        #[test]
        fn test_navigate_changes_screen() {
            let mut app = helpers::setup_settings_app();

            // Trigger Navigate::Audio
            app.world_mut()
                .trigger(SetSettingsMenu::To(SettingsMenuScreen::Audio));
            helpers::update_app(&mut app, 1);

            // Verifiziere State-Änderung
            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Audio);
        }

        /// Test: Back-Event von einem Nicht-Overview-Screen kehrt zu Overview zurück.
        ///
        /// Wenn der aktuelle Screen nicht Overview ist, sollte Back zu Overview wechseln.
        #[test]
        fn test_back_from_non_overview_returns_overview() {
            let mut app = helpers::setup_settings_app();

            // Zuerst zu Video navigieren
            app.world_mut()
                .trigger(SetSettingsMenu::To(SettingsMenuScreen::Video));
            helpers::update_app(&mut app, 1);

            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Video);

            // Trigger Back
            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);

            // Verifiziere, dass wir zurück zu Overview sind
            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Overview);
        }

        /// Test: Back-Event von Overview bleibt auf Overview.
        ///
        /// Wenn der aktuelle Screen bereits Overview ist, sollte Back keine Änderung bewirken.
        #[test]
        fn test_back_from_overview_stays_overview() {
            let mut app = helpers::setup_settings_app();

            // Verifiziere, dass wir auf Overview sind
            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Overview);

            // Trigger Back
            app.world_mut().trigger(SetSettingsMenu::Back);
            helpers::update_app(&mut app, 1);

            // Verifiziere, dass wir immer noch auf Overview sind
            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Overview);
        }

        /// Test: Cancel-Event kehrt immer zu Overview zurück.
        ///
        /// Cancel sollte unabhängig vom aktuellen Screen zu Overview wechseln.
        #[test]
        fn test_cancel_returns_overview() {
            let mut app = helpers::setup_settings_app();

            // Zuerst zu Controls navigieren
            app.world_mut()
                .trigger(SetSettingsMenu::To(SettingsMenuScreen::Controls));
            helpers::update_app(&mut app, 1);

            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Controls);

            // Trigger Cancel
            app.world_mut().trigger(SetSettingsMenu::Cancel);
            helpers::update_app(&mut app, 1);

            // Verifiziere, dass wir zurück zu Overview sind
            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Overview);
        }

        /// Test: Apply-Event (aktuell Placeholder) verändert den State nicht.
        ///
        /// Der Apply-Event hat aktuell noch keine Implementierung (TODO).
        /// Dieser Test verifiziert das aktuelle Verhalten und sollte aktualisiert werden,
        /// wenn die Apply-Logik implementiert wird.
        #[test]
        fn test_apply_placeholder() {
            let mut app = helpers::setup_settings_app();

            // Zuerst zu Audio navigieren
            app.world_mut()
                .trigger(SetSettingsMenu::To(SettingsMenuScreen::Audio));
            helpers::update_app(&mut app, 1);

            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Audio);

            // Trigger Apply (aktuell Placeholder)
            app.world_mut().trigger(SetSettingsMenu::Apply);
            helpers::update_app(&mut app, 1);

            // Verifiziere, dass der State unverändert ist (aktueller Placeholder-Verhalten)
            // TODO: Diese Assertion sollte aktualisiert werden, wenn Apply-Logik implementiert wird
            let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
            assert_eq!(settings_screen.get(), &SettingsMenuScreen::Audio);
        }
    }

    // =============================================================================
    // TESTS FÜR STATE TRANSITIONS
    // =============================================================================

    mod state_transition_tests {
        use super::*;
        use crate::states::menu::settings::SettingsMenuScreen;
        use bevy::prelude::*;

        /// Test: Navigation von Overview zu allen Settings-Screens.
        ///
        /// Testet, dass von Overview zu Audio, Video und Controls navigiert werden kann.
        #[test]
        fn test_overview_to_all_screens() {
            let targets = [
                SettingsMenuScreen::Audio,
                SettingsMenuScreen::Video,
                SettingsMenuScreen::Controls,
            ];

            for target in targets {
                let mut app = helpers::setup_settings_app();

                // Verifiziere Start-Status
                let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(settings_screen.get(), &SettingsMenuScreen::Overview);

                // Navigiere zum Target
                app.world_mut().trigger(SetSettingsMenu::To(target));
                helpers::update_app(&mut app, 1);

                // Verifiziere State-Änderung
                let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(
                    settings_screen.get(),
                    &target,
                    "Failed to navigate from Overview to {:?}",
                    target
                );
            }
        }

        /// Test: Navigation zwischen allen Settings-Screens.
        ///
        /// Testet direkte Übergänge zwischen allen Screen-Kombinationen.
        #[test]
        fn test_navigation_between_all_screens() {
            let screens = [
                SettingsMenuScreen::Overview,
                SettingsMenuScreen::Audio,
                SettingsMenuScreen::Video,
                SettingsMenuScreen::Controls,
            ];

            for from in screens {
                for to in screens {
                    let mut app = helpers::setup_settings_app();

                    // Zuerst zu 'from' navigieren
                    if from != SettingsMenuScreen::Overview {
                        app.world_mut().trigger(SetSettingsMenu::To(from));
                        helpers::update_app(&mut app, 1);
                    }

                    // Dann zu 'to' navigieren
                    app.world_mut().trigger(SetSettingsMenu::To(to));
                    helpers::update_app(&mut app, 1);

                    // Verifiziere finalen State
                    let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
                    assert_eq!(
                        settings_screen.get(),
                        &to,
                        "Failed to navigate from {:?} to {:?}",
                        from,
                        to
                    );
                }
            }
        }

        /// Test: Back-Navigation von allen Non-Overview-Screens.
        ///
        /// Testet, dass Back von Audio, Video und Controls zu Overview führt.
        #[test]
        fn test_back_from_all_non_overview_screens() {
            let non_overview_screens = [
                SettingsMenuScreen::Audio,
                SettingsMenuScreen::Video,
                SettingsMenuScreen::Controls,
            ];

            for screen in non_overview_screens {
                let mut app = helpers::setup_settings_app();

                // Navigiere zum Screen
                app.world_mut().trigger(SetSettingsMenu::To(screen));
                helpers::update_app(&mut app, 1);

                let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(settings_screen.get(), &screen);

                // Trigger Back
                app.world_mut().trigger(SetSettingsMenu::Back);
                helpers::update_app(&mut app, 1);

                // Verifiziere Rückkehr zu Overview
                let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(
                    settings_screen.get(),
                    &SettingsMenuScreen::Overview,
                    "Back from {:?} should return to Overview",
                    screen
                );
            }
        }

        /// Test: Cancel von allen Settings-Screens kehrt zu Overview zurück.
        ///
        /// Testet, dass Cancel von jedem Screen zu Overview führt.
        #[test]
        fn test_cancel_from_all_screens() {
            let screens = [
                SettingsMenuScreen::Overview,
                SettingsMenuScreen::Audio,
                SettingsMenuScreen::Video,
                SettingsMenuScreen::Controls,
            ];

            for screen in screens {
                let mut app = helpers::setup_settings_app();

                // Navigiere zum Screen (falls nicht Overview)
                if screen != SettingsMenuScreen::Overview {
                    app.world_mut().trigger(SetSettingsMenu::To(screen));
                    helpers::update_app(&mut app, 1);
                }

                let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(settings_screen.get(), &screen);

                // Trigger Cancel
                app.world_mut().trigger(SetSettingsMenu::Cancel);
                helpers::update_app(&mut app, 1);

                // Verifiziere Rückkehr zu Overview
                let settings_screen = app.world().resource::<State<SettingsMenuScreen>>();
                assert_eq!(
                    settings_screen.get(),
                    &SettingsMenuScreen::Overview,
                    "Cancel from {:?} should return to Overview",
                    screen
                );
            }
        }
    }
}
