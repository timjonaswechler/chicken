use {
    crate::{
        events::menu::{main::SetMainMenu, wiki::WikiMenuEvent},
        states::menu::{main::MainMenuScreen, wiki::WikiMenuScreen},
    },
    bevy::prelude::{App, AppExtStates, Commands, NextState, On, Plugin, Res, ResMut, State, warn},
};

pub(super) struct WikiMenuPlugin;

impl Plugin for WikiMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<WikiMenuScreen>()
            .add_observer(handle_wiki_nav);
    }
}

// --- LOGIC HANDLERS ---

/// Validates transitions for WikiMenuScreen.
pub(crate) fn is_valid_wiki_screen_transition(_from: &WikiMenuScreen, _to: &WikiMenuEvent) -> bool {
    // WikiMenuScreen currently only has Overview variant,
    // so all transitions are valid. When more variants are added,
    // validation logic should be implemented here.
    true
}

fn handle_wiki_nav(
    trigger: On<WikiMenuEvent>,
    current: Option<Res<State<WikiMenuScreen>>>,
    mut next_screen: ResMut<NextState<WikiMenuScreen>>,
    mut commands: Commands,
) {
    // Get current state, default to Overview if not set
    let current = match current {
        Some(c) => *c.get(),
        None => {
            warn!("WikiMenuScreen does not exist - MainMenuScreen must be Wiki first");
            return;
        }
    };

    // Validate transition
    if !is_valid_wiki_screen_transition(&current, trigger.event()) {
        warn!(
            "Invalid WikiMenuScreen transition: {:?} -> {:?}",
            current,
            trigger.event()
        );
        return;
    }

    match trigger.event() {
        WikiMenuEvent::To(target) => {
            next_screen.set(*target);
        }
        WikiMenuEvent::Back => {
            commands.trigger(SetMainMenu::To(MainMenuScreen::Overview));
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die Wiki-Menu Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Observer-Logik (ob Events korrekt verarbeitet werden)
    //! 2. SubState-Übergänge (ob Navigation zwischen Screens funktioniert)

    use crate::events::menu::wiki::WikiMenuEvent;
    use crate::states::menu::wiki::WikiMenuScreen;

    mod helpers {
        use crate::{
            ChickenStatePlugin,
            events::{app::SetAppScope, menu::main::SetMainMenu},
            states::{
                app::AppScope,
                menu::{main::MainMenuScreen, wiki::WikiMenuScreen},
                session::SessionType,
            },
        };
        use bevy::prelude::{NextState, State};
        use bevy::{prelude::*, state::app::StatesPlugin};

        /// Erstellt eine Test-App mit WikiMenuPlugin.
        pub fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, StatesPlugin, ChickenStatePlugin));
            app
        }

        /// Führt die App für eine bestimmte Anzahl von Update-Ticks aus.
        pub fn update_app(app: &mut App, ticks: u8) {
            for _ in 0..ticks {
                app.update();
            }
        }

        /// Setzt den MainMenuScreen auf Wiki, damit WikiMenuScreen aktiv ist.
        pub fn setup_wiki_context(app: &mut App) {
            #[cfg(feature = "hosted")]
            {
                let session_type_state = app.world().resource::<State<SessionType>>();
                assert_eq!(session_type_state.get(), &SessionType::None);

                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Splash);

                app.world_mut().trigger(SetAppScope::To(AppScope::Menu));
                update_app(app, 1);

                let menu_context = app.world().resource::<State<AppScope>>();
                assert_eq!(menu_context, &AppScope::Menu);

                let main_menu = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(main_menu.get(), &MainMenuScreen::Overview);
            }

            // Initialisiere den MainMenuScreen State
            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Wiki));
            update_app(app, 1);

            let main_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(main_context.get(), &MainMenuScreen::Wiki);

            // Verifiziere, dass WikiMenuScreen existiert und auf Overview ist
            let wiki_screen = app.world().resource::<State<WikiMenuScreen>>();
            assert_eq!(wiki_screen.get(), &WikiMenuScreen::Overview);
        }
    }

    // =============================================================================
    // TESTS FÜR VALIDATOR-FUNKTIONEN
    // =============================================================================

    mod validator_tests {
        use crate::events::menu::wiki::WikiMenuEvent;
        use crate::logic::menu::wiki::is_valid_wiki_screen_transition;
        use crate::states::menu::wiki::WikiMenuScreen;

        /// Test: Gültige WikiMenuScreen-Übergänge werden als gültig erkannt.
        ///
        /// Da WikiMenuScreen aktuell nur Overview hat, sind alle Übergänge gültig.
        #[test]
        fn test_valid_wiki_screen_transitions() {
            // Overview → To(Overview) ist gültig
            assert!(is_valid_wiki_screen_transition(
                &WikiMenuScreen::Overview,
                &WikiMenuEvent::To(WikiMenuScreen::Overview)
            ));

            // Overview → Back ist gültig
            assert!(is_valid_wiki_screen_transition(
                &WikiMenuScreen::Overview,
                &WikiMenuEvent::Back
            ));
        }

        /// Test: Alle Übergänge sind aktuell gültig (da nur Overview existiert).
        ///
        /// Dieser Test dokumentiert das aktuelle Verhalten.
        /// Wenn weitere Varianten hinzugefügt werden, sollten hier
        /// ungültige Übergänge getestet werden.
        #[test]
        fn test_no_invalid_transitions_yet() {
            // Aktuell gibt es keine ungültigen Übergänge,
            // da WikiMenuScreen nur Overview hat.
            // Dieser Test wird failen, sobald Validierung implementiert wird.
            assert!(is_valid_wiki_screen_transition(
                &WikiMenuScreen::Overview,
                &WikiMenuEvent::To(WikiMenuScreen::Overview)
            ));
        }
    }

    // =============================================================================
    // TESTS FÜR OBSERVER
    // =============================================================================

    mod observer_tests {
        use crate::{
            events::app::SetAppScope,
            states::{app::AppScope, menu::main::MainMenuScreen},
        };

        use super::*;
        use bevy::prelude::State;

        /// Test: Navigate-Event ändert den WikiMenuScreen.
        ///
        /// Ein Navigate-Event mit einem Ziel-Screen sollte den State auf diesen
        /// Screen ändern.
        #[test]
        fn test_navigate_changes_screen() {
            let mut app = helpers::test_app();
            helpers::setup_wiki_context(&mut app);

            // Aktueller State sollte Overview sein
            let current_screen = app.world().resource::<State<WikiMenuScreen>>();
            assert_eq!(current_screen.get(), &WikiMenuScreen::Overview);

            // Act: Sende Navigate Event zu Overview (zurück zu sich selbst)
            app.world_mut()
                .trigger(WikiMenuEvent::To(WikiMenuScreen::Overview));
            helpers::update_app(&mut app, 1);

            // Assert: State sollte immer noch Overview sein
            let new_screen = app.world().resource::<State<WikiMenuScreen>>();
            assert_eq!(new_screen.get(), &WikiMenuScreen::Overview);
        }

        /// Test: Back-Event wechselt zurück zum Hauptmenü.
        ///
        /// Ein Back-Event sollte den MainMenuScreen zu Main ändern.
        #[test]
        fn test_navigate_back() {
            let mut app = helpers::test_app();
            helpers::setup_wiki_context(&mut app);

            // Act: Sende Back Event
            app.world_mut().trigger(WikiMenuEvent::Back);
            helpers::update_app(&mut app, 1);

            // Assert: MainMenuScreen sollte zu Main gewechselt sein
            let current_screen = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(current_screen.get(), &MainMenuScreen::Overview);
        }

        /// Test: To-Event zu unterschiedlichem Screen ändert den State.
        ///
        /// Da WikiMenuScreen aktuell nur Overview hat, testen wir den
        /// Übergang zu Overview (selber State).
        #[test]
        fn test_navigate_to_overview() {
            let mut app = helpers::test_app();
            helpers::setup_wiki_context(&mut app);

            // Act: Sende To(Overview) Event
            app.world_mut()
                .trigger(WikiMenuEvent::To(WikiMenuScreen::Overview));
            helpers::update_app(&mut app, 1);

            // Assert: State sollte Overview sein
            let current_screen = app.world().resource::<State<WikiMenuScreen>>();
            assert_eq!(current_screen.get(), &WikiMenuScreen::Overview);
        }

        /// Test: Events werden ignoriert wenn WikiMenuScreen nicht existiert.
        ///
        /// Wenn MainMenuScreen nicht Wiki ist, sollte WikiMenuScreen
        /// nicht existieren und Events sollten ignoriert werden.
        #[test]
        fn test_event_without_wiki_context_ignored() {
            let mut app = helpers::test_app();
            helpers::update_app(&mut app, 1);

            // Setup: Gehe zu Menu, aber nicht zu Wiki
            #[cfg(feature = "hosted")]
            {
                app.world_mut().trigger(SetAppScope::To(AppScope::Menu));
                helpers::update_app(&mut app, 1);

                // Verifiziere, dass wir im Menu sind (nicht Wiki)
                let app_scope = app.world().resource::<State<AppScope>>();
                assert_eq!(app_scope.get(), &AppScope::Menu);

                let main_menu = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(main_menu.get(), &MainMenuScreen::Overview);
            }

            // Act: Versuche Wiki-Event zu senden (sollte ignoriert werden)
            app.world_mut()
                .trigger(WikiMenuEvent::To(WikiMenuScreen::Overview));
            helpers::update_app(&mut app, 1);

            // Assert: MainMenuScreen sollte unverändert sein
            let main_menu = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(main_menu.get(), &MainMenuScreen::Overview);
        }
    }

    // =============================================================================
    // TESTS FÜR STATE-TRANSITIONS
    // =============================================================================
}
