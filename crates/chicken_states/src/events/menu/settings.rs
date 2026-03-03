use {crate::states::menu::settings::SettingsMenuScreen, bevy::prelude::Event};

/// Events for navigation and actions within the settings menu.
///
/// Used to navigate between settings categories and manage configuration changes,
/// including applying or discarding modifications.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSettingsMenu {
    /// Navigate to a specific settings screen (Overview, Audio, Video, Controls).
    To(SettingsMenuScreen),
    /// Return to the previous menu level.
    Back,
    /// Apply the current settings changes.
    Apply,
    /// Discard changes and return to previous state.
    Cancel,
}
