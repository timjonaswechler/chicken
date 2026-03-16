use bevy::prelude::{Reflect, States};

/// The high-level context of the application.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AppScope {
    /// Initial splash screen / Intro (Client only).
    /// Loads essential assets and shows logo.
    #[cfg(feature = "hosted")]
    Splash,

    /// The main menu (Client only).
    #[cfg(feature = "hosted")]
    Menu,

    /// The actual game session (Client & Server).
    /// Contains the world, physics, and networking.
    Session,
}

impl Default for AppScope {
    fn default() -> Self {
        // Client starts at Splash screen
        #[cfg(feature = "hosted")]
        return AppScope::Splash;

        // Dedicated Server starts directly in Session
        #[cfg(feature = "headless")]
        return AppScope::Session;

        // Unreachable: compile_error! in lib.rs fires first when no feature is enabled
        #[cfg(not(any(feature = "hosted", feature = "headless")))]
        return AppScope::Session;
    }
}
