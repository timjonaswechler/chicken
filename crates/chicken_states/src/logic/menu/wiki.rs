use {
    crate::{
        events::menu::{main::MainMenuInteraction, wiki::WikiMenuEvent},
        states::menu::{main::MainMenuContext, wiki::WikiMenuScreen},
    },
    bevy::prelude::{App, AppExtStates, Commands, NextState, On, Plugin, ResMut},
};

pub(super) struct WikiMenuPlugin;

impl Plugin for WikiMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<WikiMenuScreen>()
            .add_observer(handle_wiki_nav);
    }
}

// --- LOGIC HANDLERS ---

fn handle_wiki_nav(
    trigger: On<WikiMenuEvent>,
    mut next_screen: ResMut<NextState<WikiMenuScreen>>,
    mut commands: Commands,
) {
    match trigger.event() {
        WikiMenuEvent::Navigate(target) => {
            next_screen.set(*target);
        }
        WikiMenuEvent::Back => {
            commands.trigger(MainMenuInteraction::SwitchContext(MainMenuContext::Main));
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
            events::{app::SetAppScope, menu::main::MainMenuInteraction},
            states::{
                app::AppScope,
                menu::{main::MainMenuContext, wiki::WikiMenuScreen},
                session::SessionType,
            },
        };
        use bevy::prelude::State;
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

        /// Setzt den MainMenuContext auf Wiki, damit WikiMenuScreen aktiv ist.
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

                let main_menu = app.world().resource::<State<MainMenuContext>>();
                assert_eq!(main_menu.get(), &MainMenuContext::Main);
            }

            // Initialisiere den MainMenuContext State
            app.world_mut()
                .trigger(MainMenuInteraction::SwitchContext(MainMenuContext::Wiki));
            update_app(app, 1);

            let main_context = app.world().resource::<State<MainMenuContext>>();
            assert_eq!(main_context.get(), &MainMenuContext::Wiki);

            // Verifiziere, dass WikiMenuScreen existiert und auf Overview ist
            let wiki_screen = app.world().resource::<State<WikiMenuScreen>>();
            assert_eq!(wiki_screen.get(), &WikiMenuScreen::Overview);
        }
    }

    // =============================================================================
    // TESTS FÜR OBSERVER
    // =============================================================================

    mod observer_tests {
        use crate::states::menu::main::MainMenuContext;

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
                .trigger(WikiMenuEvent::Navigate(WikiMenuScreen::Overview));
            helpers::update_app(&mut app, 1);

            // Assert: State sollte immer noch Overview sein
            let new_screen = app.world().resource::<State<WikiMenuScreen>>();
            assert_eq!(new_screen.get(), &WikiMenuScreen::Overview);
        }

        /// Test: Back-Event verursacht keinen Panic.
        ///
        /// Das Back-Event ist aktuell ein Placeholder. Dieser Test stellt sicher,
        /// dass das Event ohne Fehler verarbeitet wird.
        #[test]
        fn test_navigate_back() {
            let mut app = helpers::test_app();
            helpers::setup_wiki_context(&mut app);

            // Act: Sende Back Event - sollte nicht panic!
            app.world_mut().trigger(WikiMenuEvent::Back);
            helpers::update_app(&mut app, 1);

            // Assert: State sollte unverändert sein
            let current_screen = app.world().resource::<State<MainMenuContext>>();
            assert_eq!(current_screen.get(), &MainMenuContext::Main);
        }
    }

    // =============================================================================
    // TESTS FÜR STATE-TRANSITIONS
    // =============================================================================
}
