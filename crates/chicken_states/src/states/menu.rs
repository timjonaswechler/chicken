//! Menu state definitions for main menu navigation.
//!
//! Defines the state hierarchy for the game's main menu system:
//! - [`main`]: Top-level menu categories (Singleplayer, Multiplayer, Wiki, Settings)
//! - [`singleplayer`]: New game creation and save loading flows
//! - [`multiplayer`]: Hosting and joining multiplayer sessions
//! - [`settings`]: Audio, video, and control configuration screens
//! - [`wiki`]: In-game documentation browser
//!
//! All menu states are `SubStates` of `AppScope::Menu` and are only available
//! in hosted builds with UI support.

pub mod main;
pub mod multiplayer;
pub mod settings;
pub mod singleplayer;
pub mod wiki;
