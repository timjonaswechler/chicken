use {
    crate::{events::menu::main::SetMainMenu, states::menu::main::MainMenuScreen},
    bevy::prelude::{
        App, AppExit, AppExtStates, MessageWriter, NextState, On, Plugin, Res, ResMut, State, warn,
    },
};

pub(super) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        // State initialization is done in states module or logic root?
        // SubStates must be added. MainMenuScreen is a SubState of AppScope::Menu.
        app.add_sub_state::<MainMenuScreen>()
            .add_observer(handle_main_menu_interaction);
    }
}

// --- LOGIC HANDLERS ---

/// Validates transitions for MainMenuScreen.
pub(crate) fn is_valid_main_menu_screen_transition(
    _from: &MainMenuScreen,
    _to: &SetMainMenu,
) -> bool {
    // All transitions between MainMenuScreen variants are valid.
    // Exit is always valid from any state.
    true
}

fn handle_main_menu_interaction(
    event: On<SetMainMenu>,
    current: Option<Res<State<MainMenuScreen>>>,
    mut menu_context: ResMut<NextState<MainMenuScreen>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    // Get current state
    let current = match current {
        Some(c) => *c.get(),
        None => {
            warn!("MainMenuScreen does not exist - AppScope must be Menu first");
            return;
        }
    };

    // Validate transition
    if !is_valid_main_menu_screen_transition(&current, event.event()) {
        warn!(
            "Invalid MainMenuScreen transition: {:?} -> {:?}",
            current,
            event.event()
        );
        return;
    }

    match *event.event() {
        SetMainMenu::To(context) => {
            menu_context.set(context);
        }
        SetMainMenu::Exit => {
            exit_writer.write(AppExit::Success);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests für die Main Menu Logik.
    //!
    //! Diese Tests prüfen:
    //! 1. Observer-Logik (ob Events korrekt verarbeitet werden)
    //! 2. State-Übergänge (ob MainMenuScreen korrekt gewechselt wird)
    //! 3. AppExit-Events (ob Exit korrekt ausgelöst wird)

    mod helpers {
        use crate::{
            ChickenStatePlugin,
            events::app::SetAppScope,
            states::{app::AppScope, menu::main::MainMenuScreen, session::SessionType},
        };
        use bevy::{prelude::*, state::app::StatesPlugin};

        /// Erstellt eine Test-App mit MainMenuPlugin und AppLogicPlugin.
        pub fn test_app() -> App {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, StatesPlugin, ChickenStatePlugin));
            app
        }

        /// Führt den App-Update für die angegebene Anzahl von Ticks aus.
        pub fn update_app(app: &mut App, i: u8) {
            for _ in 0..i {
                app.update();
            }
        }

        /// Setzt die App in den Menu-Scope und initialisiert MainMenuScreen.
        pub fn setup_menu_app() -> App {
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

            app
        }
    }

    // =============================================================================
    // TESTS FÜR VALIDATOR-FUNKTIONEN
    // =============================================================================

    mod validator_tests {
        use crate::events::menu::main::SetMainMenu;
        use crate::logic::menu::main::is_valid_main_menu_screen_transition;
        use crate::states::menu::main::MainMenuScreen;

        /// Test: Gültige MainMenuScreen-Übergänge werden als gültig erkannt.
        ///
        /// Alle Übergänge zwischen MainMenuScreen-Varianten sind gültig.
        /// Exit ist von jedem State gültig.
        #[test]
        fn test_valid_main_menu_screen_transitions() {
            // Main → To(Singleplayer) ist gültig
            assert!(is_valid_main_menu_screen_transition(
                &MainMenuScreen::Overview,
                &SetMainMenu::To(MainMenuScreen::Singleplayer)
            ));

            // Singleplayer → To(Main) ist gültig
            assert!(is_valid_main_menu_screen_transition(
                &MainMenuScreen::Singleplayer,
                &SetMainMenu::To(MainMenuScreen::Overview)
            ));

            // Multiplayer → To(Settings) ist gültig
            assert!(is_valid_main_menu_screen_transition(
                &MainMenuScreen::Multiplayer,
                &SetMainMenu::To(MainMenuScreen::Settings)
            ));

            // Exit von jedem State ist gültig
            assert!(is_valid_main_menu_screen_transition(
                &MainMenuScreen::Overview,
                &SetMainMenu::Exit
            ));
            assert!(is_valid_main_menu_screen_transition(
                &MainMenuScreen::Singleplayer,
                &SetMainMenu::Exit
            ));
            assert!(is_valid_main_menu_screen_transition(
                &MainMenuScreen::Settings,
                &SetMainMenu::Exit
            ));
        }

        /// Test: Alle Übergänge zwischen MainMenuScreen-Varianten sind gültig.
        ///
        /// Dieser Test prüft alle Kombinationen.
        #[test]
        fn test_all_screen_transitions_valid() {
            let screens = [
                MainMenuScreen::Overview,
                MainMenuScreen::Singleplayer,
                MainMenuScreen::Multiplayer,
                MainMenuScreen::Wiki,
                MainMenuScreen::Settings,
            ];

            for from in &screens {
                for to in &screens {
                    assert!(
                        is_valid_main_menu_screen_transition(from, &SetMainMenu::To(*to)),
                        "Transition from {:?} to {:?} should be valid",
                        from,
                        to
                    );
                }
            }
        }
    }

    // =============================================================================
    // TESTS FÜR MAIN MENU INTERACTION OBSERVER
    // =============================================================================

    mod observer_tests {
        use crate::{events::menu::main::SetMainMenu, states::menu::main::MainMenuScreen};
        use bevy::prelude::*;

        use super::helpers;

        /// Test: SwitchContext-Event ändert den MainMenuScreen.
        ///
        /// Ein SwitchContext-Event mit Singleplayer sollte den State zu
        /// MainMenuScreen::Singleplayer ändern.
        #[test]
        fn test_switch_context_changes_menu() {
            let mut app = helpers::setup_menu_app();

            // Trigger SwitchContext::Singleplayer
            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Singleplayer));
            helpers::update_app(&mut app, 1);

            // Verifiziere State-Änderung
            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Singleplayer);
        }

        /// Test: Mehrere SwitchContext-Events wechseln den State korrekt.
        ///
        /// Testet, dass mehrere aufeinanderfolgende SwitchContext-Events
        /// den MainMenuScreen korrekt ändern.
        #[test]
        fn test_multiple_switch_context_changes() {
            let mut app = helpers::setup_menu_app();

            // Wechsel zu Singleplayer
            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Singleplayer));
            helpers::update_app(&mut app, 1);

            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Singleplayer);

            // Wechsel zu Multiplayer
            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Multiplayer));
            helpers::update_app(&mut app, 1);

            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Multiplayer);

            // Wechsel zu Settings
            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Settings));
            helpers::update_app(&mut app, 1);

            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Settings);

            // Zurück zu Main
            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Overview));
            helpers::update_app(&mut app, 1);

            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Overview);
        }

        /// Test: SwitchContext-Event funktioniert für alle Context-Varianten.
        ///
        /// Testet, dass jede MainMenuScreen-Variante korrekt gesetzt werden kann.
        #[test]
        fn test_switch_context_all_variants() {
            let variants = [
                MainMenuScreen::Overview,
                MainMenuScreen::Singleplayer,
                MainMenuScreen::Multiplayer,
                MainMenuScreen::Wiki,
                MainMenuScreen::Settings,
            ];

            for variant in variants {
                let mut app = helpers::setup_menu_app();

                app.world_mut().trigger(SetMainMenu::To(variant));
                helpers::update_app(&mut app, 1);

                let menu_context = app.world().resource::<State<MainMenuScreen>>();
                assert_eq!(
                    menu_context.get(),
                    &variant,
                    "Failed to switch to {:?}",
                    variant
                );
            }
        }

        /// Test: Exit-Event löst AppExit::Success aus.
        ///
        /// Ein Exit-Event sollte ein AppExit::Success in den MessageWriter schreiben.
        #[test]
        fn test_exit_writes_app_exit() {
            let mut app = helpers::setup_menu_app();

            // Trigger Exit
            app.world_mut().trigger(SetMainMenu::Exit);
            helpers::update_app(&mut app, 1);

            // Verifiziere, dass AppExit::Success ausgelöst wurde
            let exit_messages = app.world().resource::<Messages<AppExit>>();
            let mut reader = exit_messages.get_cursor();
            let messages: Vec<&AppExit> = reader.read(exit_messages).collect();

            assert_eq!(messages.len(), 1, "Expected exactly one AppExit message");
            assert_eq!(messages[0], &AppExit::Success);
        }

        /// Test: Exit-Event erzeugt keinen State-Change.
        ///
        /// Das Exit-Event sollte den MainMenuScreen nicht ändern.
        #[test]
        fn test_exit_does_not_change_context() {
            let mut app = helpers::setup_menu_app();

            // Zuerst zu Singleplayer wechseln
            app.world_mut()
                .trigger(SetMainMenu::To(MainMenuScreen::Singleplayer));
            helpers::update_app(&mut app, 1);

            // Dann Exit triggern
            app.world_mut().trigger(SetMainMenu::Exit);
            helpers::update_app(&mut app, 1);

            // Verifiziere, dass Context unverändert ist
            let menu_context = app.world().resource::<State<MainMenuScreen>>();
            assert_eq!(menu_context.get(), &MainMenuScreen::Singleplayer);

            // Und AppExit wurde trotzdem ausgelöst
            let exit_messages = app.world().resource::<Messages<AppExit>>();
            let mut reader = exit_messages.get_cursor();
            let messages: Vec<&AppExit> = reader.read(exit_messages).collect();
            assert_eq!(messages.len(), 1);
        }
    }
}
