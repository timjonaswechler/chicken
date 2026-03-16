//! Menu navigation events for UI interaction.
//!
//! Defines events for controlling menu navigation flows:
//! - `multiplayer`: Events for hosting and joining multiplayer games
//! - `singleplayer`: Events for new game creation and save loading
//! - `settings`: Events for navigating settings categories and applying changes
//! - `wiki`: Events for browsing in-game documentation
//!
//! These events are triggered by UI elements and processed by observers
//! in the corresponding logic modules to update menu states.

pub mod multiplayer;
pub mod settings;
pub mod singleplayer;
pub mod wiki;
