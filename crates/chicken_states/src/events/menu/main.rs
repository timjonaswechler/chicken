use {crate::states::menu::main::MainMenuScreen, bevy::prelude::Event};

/// Events for navigation within the main menu.
///
/// Used to switch between different menu contexts (Singleplayer, Multiplayer, etc.)
/// or to exit the application from the apps menu.
#[derive(Event, Debug, Clone, Copy)]
pub enum SetMainMenu {
    /// Navigate to a specific main menu context (e.g., Singleplayer, Multiplayer, Wiki, Settings).
    To(MainMenuScreen),

    /// Exit the application.
    Exit,
}
