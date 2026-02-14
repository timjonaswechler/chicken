use {
    crate::{
        events::menu::settings::SettingsMenuEvent, states::menu::settings::SettingsMenuScreen,
    },
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, Res, ResMut, State},
};

pub(super) struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SettingsMenuScreen>()
            .add_observer(handle_settings_nav);
    }
}

// --- LOGIC HANDLERS ---

fn handle_settings_nav(
    trigger: On<SettingsMenuEvent>,
    mut next_screen: ResMut<NextState<SettingsMenuScreen>>,
    current_screen: Res<State<SettingsMenuScreen>>,
) {
    match trigger.event() {
        SettingsMenuEvent::Navigate(target) => {
            next_screen.set(*target);
        }
        SettingsMenuEvent::Back => {
            if *current_screen.get() != SettingsMenuScreen::Overview {
                next_screen.set(SettingsMenuScreen::Overview);
            }
        }
        SettingsMenuEvent::Apply => {
            // TODO: Apply settings logic
        }
        SettingsMenuEvent::Cancel => {
            next_screen.set(SettingsMenuScreen::Overview);
        }
    }
}
