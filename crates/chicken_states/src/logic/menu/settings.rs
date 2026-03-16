//! Settings menu navigation logic.
//!
//! Handles state transitions for the settings menu system:
//! - Navigation between settings categories (Audio, Video, Controls)
//! - Back navigation to main menu or settings overview
//! - Apply and Cancel actions for settings changes
//!
//! Validates all transitions to ensure proper navigation flow within the settings menu.

use {
    crate::{
        events::menu::settings::SetSettingsMenu,
        states::menu::{main::MainMenuScreen, settings::SettingsMenuScreen},
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State, warn},
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
            let current_state = match current {
                Some(ref s) => *s.get(),
                None => {
                    warn!("SettingsMenuScreen does not exist - cannot process Back event");
                    return;
                }
            };
            if !is_valid_settings_menu_screen_transition(&current_state, event.event()) {
                warn!(
                    "Invalid SettingsMenuScreen transition for Back: {:?} -> {:?}",
                    current_state,
                    event.event()
                );
                return;
            }
            match current_state {
                // Back from Overview → return to MainMenu
                SettingsMenuScreen::Overview => next_main_screen.set(MainMenuScreen::Overview),
                // Back from a category → return to Settings Overview
                _ => next_settings_screen.set(SettingsMenuScreen::Overview),
            }
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
}
