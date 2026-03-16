//! Settings menu navigation events.
//!
//! Events for navigating the settings configuration interface:
//! - Navigate between settings categories: Overview, Audio, Video, Controls
//! - Return to parent menu (Back)
//! - Apply or discard configuration changes
//!
//! Settings events are processed by the `logic::menu::settings` observers
//! which handle state transitions and coordinate with the settings system.

use bevy::prelude::Event;

/// Events for navigation and actions within the settings menu.
///
/// Used to navigate between settings categories and manage configuration changes,
/// including applying or discarding modifications.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSettingsMenu {
    /// Navigate to the overview settings screen.
    Overview,
    /// Navigate to the audio settings screen.
    Audio,
    /// Navigate to the video settings screen.
    Video,
    /// Navigate to the controls settings screen.
    Controls,
    /// Return to the previous menu level.
    Back,
    /// Apply the current settings changes.
    Apply,
    /// Discard changes and return to previous state.
    Cancel,
}
