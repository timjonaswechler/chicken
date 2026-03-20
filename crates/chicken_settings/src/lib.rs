#![warn(missing_docs, clippy::unwrap_used)]

//! Async settings management for Bevy 0.18.0 with TOML support and file watching.
//!
//! This crate provides a type-safe, asynchronous settings system that uses
//! TOML for serialization with automatic format detection based on file extension.
//!
//! # Features
//!
//! - **Static and dynamic paths**: Settings can have fixed paths or paths with
//!   placeholders that are resolved at runtime using context.
//! - **Context as Resource**: Dynamic paths use Bevy resources as context.
//! - **Hybrid API**: Use events for fire-and-forget operations, or resource
//!   methods when you need results.
//! - **Format auto-detection**: Supported formats are detected from file extensions.
//! - **Delta storage**: Only changed values are saved, reducing file size.
//! - **Event-driven**: Settings changes are communicated through events.
//!
//! # Examples
//!
//! ## Static Settings
//!
//! ```ignore
//! use chicken_settings::prelude::*;
//! use bevy::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone)]
//! #[settings("settings/audio.toml")]
//! struct AudioSettings {
//!     volume: f32,
//!     muted: bool,
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(SettingsPlugin)
//!         .add_settings::<AudioSettings>()
//!         .run();
//! }
//! ```
//!
//! ## Dynamic Settings with Context
//!
//! ```ignore
//! use chicken_settings::prelude::*;
//! use bevy::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(SettingsContext, Resource, Clone)]
//! struct SaveContext {
//!     slot_id: u32,
//! }
//!
//! #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone)]
//! #[settings("saves/{slot_id}/player.toml", context = SaveContext)]
//! struct PlayerSettings {
//!     name: String,
//!     health: f32,
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(SettingsPlugin)
//!         .insert_resource(SaveContext { slot_id: 0 })
//!         .add_settings_with_context::<PlayerSettings, SaveContext>()
//!         .run();
//! }
//! ```

/// Error types for the settings system.
pub mod error;

/// Serialization format support with auto-detection from file extensions.
pub mod format;

/// Storage backends for reading and writing settings files.
pub mod storage;

/// Delta storage for saving only changed values.
pub mod delta;

/// Events for requesting and responding to settings operations.
pub mod events;

/// Path resolution and context handling for dynamic paths.
pub mod path;

/// The main plugin and app extension for settings management.
pub mod plugin;

pub use delta::DeltaStorage;
pub use error::SettingsError;
pub use events::*;
pub use format::Format;
pub use path::{PathContext, SettingsPath};
pub use plugin::{
    ChickenSettingsPlugin, SettingsAppExt, SettingsLoader, SettingsRegistry,
    SettingsStorageResource,
};
pub use storage::{AsyncFileStorage, SettingsStorage, StorageHandle};
pub use chicken_settings_edit::{EditError, SurgicalEditor, TomlEditor, update_toml_value};

use bevy_ecs::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::any::Any;

/// Marker trait for settings that don't use dynamic paths.
///
/// This trait is automatically implemented for settings with static paths.
pub trait StaticSettings: Settings {}

/// Core trait for settings types.
///
/// This trait is automatically implemented by the `#[derive(Settings)]` macro.
/// Manual implementation is possible but not recommended.
///
/// # Associated Types
///
/// * `Context` - The context type for path resolution. Use `()` for static settings.
///
/// # Required Methods
///
/// * `type_name()` - Returns the unique type name for this settings type.
/// * `path_template()` - Returns the path template (may contain placeholders).
/// * `has_dynamic_path()` - Returns true if the path has placeholders.
/// * `format()` - Returns the serialization format based on file extension.
/// * `into_box()` - Converts to a boxed Any for type-erased storage.
/// * `clone_box()` - Clones as a boxed Any.
pub trait Settings:
    Resource + Serialize + DeserializeOwned + Default + Clone + Send + Sync + 'static
{
    /// The context type for path resolution.
    ///
    /// For static settings, use `()`. For dynamic settings, this is your context type.
    type Context: PathContext;

    /// Returns the unique type name for this settings type.
    fn type_name() -> &'static str;

    /// Returns the path template for this settings type.
    ///
    /// This may contain placeholders like `{slot_id}` for dynamic paths.
    /// Placeholders are resolved using the [`PathContext`] trait.
    fn path_template() -> &'static str;

    /// Returns true if this settings type has placeholders in its path template.
    fn has_dynamic_path() -> bool;

    /// Returns the format based on the file extension.
    ///
    /// See [`Format::from_path`](crate::Format::from_path) for supported formats.
    fn format() -> Format;

    /// Converts this settings instance to a boxed Any for type-erased storage.
    fn into_box(self) -> Box<dyn Any + Send + Sync>;

    /// Creates a clone of this settings instance as a boxed Any.
    fn clone_box(&self) -> Box<dyn Any + Send + Sync>;
}

/// Extension trait for settings with dynamic paths.
///
/// Settings types with placeholders in their path should implement this trait
/// to enable runtime path resolution.
///
/// This is automatically implemented by the `#[derive(Settings)]` macro.
pub trait DynamicSettings: Settings {
    /// Resolves the path template using the provided context.
    ///
    /// # Arguments
    ///
    /// * `context` - The context to use for resolving placeholders
    fn resolve_path(context: &Self::Context) -> SettingsPath;
}

/// Trait for types that can be used as settings context.
///
/// Implement this trait and derive `Resource` for your context types.
/// The context is stored as a Bevy resource and read implicitly during
/// settings operations.
///
/// # Example
///
/// ```ignore
/// use chicken_settings::prelude::*;
/// use bevy::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(SettingsContext, Resource, Clone)]
/// struct SaveContext {
///     slot_id: u32,
/// }
///
/// fn change_slot(
///     mut context: ResMut<SaveContext>,
///     loader: Res<SettingsLoader>,
/// ) {
///     context.slot_id = 5;
///     // After changing context, reload settings explicitly
///     loader.load::<PlayerSettings>();
/// }
/// ```
pub trait SettingsContext: PathContext + Resource {}

impl<T: PathContext + Resource> SettingsContext for T {}

/// Re-export derive macros when the `derive` feature is enabled.
pub use chicken_settings_derive::{Settings, SettingsContext};

/// Prelude module for convenient imports.
pub mod prelude {
    // Import derive macros when the feature is enabled
    pub use super::DynamicSettings;

    // Import traits - but exclude the ones that are derive macros when feature is enabled
    pub use super::{
        AsyncFileStorage, DeltaStorage, Format, LoadAllSettings, LoadSettings, PathContext,
        ReloadSettings, SaveAllSettings, SaveSettings, SettingsAppExt, SettingsError,
        SettingsLoadFailed, SettingsLoaded, SettingsLoader, SettingsPath, SettingsRegistry,
        SettingsSaveFailed, SettingsSaved, SettingsStorage, SettingsStorageResource,
        StaticSettings, StorageHandle,
    };
}
