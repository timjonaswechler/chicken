use bevy::prelude::{Reflect, States};

/// The high-level context of the application.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AppScope {
    /// Initial splash screen / Intro (Client only).
    /// Loads essential assets and shows logo.
    #[cfg(feature = "client")]
    Splash,

    /// The main menu (Client only).
    #[cfg(feature = "client")]
    Menu,

    /// The actual game session (Client & Server).
    /// Contains the world, physics, and networking.
    Session,
}

impl Default for AppScope {
    fn default() -> Self {
        // Client starts at Splash screen
        #[cfg(feature = "client")]
        return AppScope::Splash;

        // Dedicated Server starts directly in Session
        #[cfg(all(feature = "server", not(feature = "client")))]
        return AppScope::Session;

        // Fallback safety
        #[cfg(all(not(feature = "client"), not(feature = "server")))]
        panic!("No valid features enabled");
    }
}
