use {
    crate::{events::menu::wiki::WikiMenuEvent, states::menu::wiki::WikiMenuScreen},
    bevy::prelude::{App, AppExtStates, NextState, On, Plugin, ResMut},
};

pub(super) struct WikiMenuPlugin;

impl Plugin for WikiMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<WikiMenuScreen>()
            .add_observer(handle_wiki_nav);
    }
}

// --- LOGIC HANDLERS ---

fn handle_wiki_nav(trigger: On<WikiMenuEvent>, mut next_screen: ResMut<NextState<WikiMenuScreen>>) {
    match trigger.event() {
        WikiMenuEvent::Navigate(target) => {
            next_screen.set(*target);
        }
        WikiMenuEvent::Back => {
            // Placeholder for back navigation logic
        }
    }
}
