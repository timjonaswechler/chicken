use {crate::states::menu::main::MainMenuContext, bevy::prelude::Event};

/// Events for navigation within the main menu.
///
/// Used to switch between different menu contexts (Singleplayer, Multiplayer, etc.)
/// or to exit the application from the main menu.
#[derive(Event, Debug, Clone, Copy)]
pub enum MainMenuInteraction {
    /// Navigate to a specific main menu context (e.g., Singleplayer, Multiplayer, Wiki, Settings).
    SwitchContext(MainMenuContext),
    /// Exit the application.
    Exit,
}
