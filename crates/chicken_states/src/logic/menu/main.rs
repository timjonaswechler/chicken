use {
    crate::{events::menu::main::MainMenuInteraction, states::menu::main::MainMenuContext},
    bevy::prelude::{App, AppExit, AppExtStates, MessageWriter, NextState, On, Plugin, ResMut},
};

pub(super) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        // State initialization is done in states module or logic root?
        // SubStates must be added. MainMenuContext is a SubState of AppScope::Menu.
        app.add_sub_state::<MainMenuContext>()
            .add_observer(handle_main_menu_interaction);
    }
}

fn handle_main_menu_interaction(
    event: On<MainMenuInteraction>,
    mut menu_context: ResMut<NextState<MainMenuContext>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    match *event {
        MainMenuInteraction::SwitchContext(context) => {
            menu_context.set(context);
        }
        MainMenuInteraction::Exit => {
            exit_writer.write(AppExit::Success);
        }
    }
}
