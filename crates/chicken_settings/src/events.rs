//! Events for the settings system.
//!
//! This module provides event types for fire-and-forget settings operations.
//! For operations where you need results, use the [`SettingsLoader`](crate::plugin::SettingsLoader) resource methods.

use crate::path::{PathContext, SettingsPath};
use crate::Settings;
use bevy_ecs::message::Message;
use std::marker::PhantomData;

/// Event to request saving settings to disk.
///
/// # Examples
///
/// ```ignore
/// // Fire-and-forget save
/// events.send(SaveSettings::<AudioSettings>::new());
/// ```
#[derive(Debug, Clone)]
pub struct SaveSettings<T: Settings, C = ()> {
    context: Option<C>,
    _phantom: PhantomData<T>,
}

impl<T: Settings> SaveSettings<T, ()> {
    /// Create a new save event for static settings.
    pub fn new() -> Self {
        Self {
            context: None,
            _phantom: PhantomData,
        }
    }
}

impl<T: Settings, C: PathContext> SaveSettings<T, C> {
    /// Create a new save event with context for dynamic settings.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The settings type to save
    /// * `C` - The context type for path resolution
    pub fn with_context(context: C) -> Self {
        Self {
            context: Some(context),
            _phantom: PhantomData,
        }
    }

    /// Get the context, if any.
    pub fn context(&self) -> Option<&C> {
        self.context.as_ref()
    }

    /// Returns true if this save request has a context.
    pub fn has_context(&self) -> bool {
        self.context.is_some()
    }
}

impl<T: Settings, C> Default for SaveSettings<T, C> {
    fn default() -> Self {
        Self {
            context: None,
            _phantom: PhantomData,
        }
    }
}

/// Event to request loading settings from disk.
///
/// # Examples
///
/// ```ignore
/// // Fire-and-forget load
/// events.send(LoadSettings::<AudioSettings>::new());
/// ```
#[derive(Debug, Clone)]
pub struct LoadSettings<T: Settings, C = ()> {
    context: Option<C>,
    _phantom: PhantomData<T>,
}

impl<T: Settings> LoadSettings<T, ()> {
    /// Create a new load event for static settings.
    pub fn new() -> Self {
        Self {
            context: None,
            _phantom: PhantomData,
        }
    }
}

impl<T: Settings, C: PathContext> LoadSettings<T, C> {
    /// Create a new load event with context for dynamic settings.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The settings type to load
    /// * `C` - The context type for path resolution
    pub fn with_context(context: C) -> Self {
        Self {
            context: Some(context),
            _phantom: PhantomData,
        }
    }

    /// Get the context, if any.
    pub fn context(&self) -> Option<&C> {
        self.context.as_ref()
    }

    /// Returns true if this load request has a context.
    pub fn has_context(&self) -> bool {
        self.context.is_some()
    }
}

impl<T: Settings, C> Default for LoadSettings<T, C> {
    fn default() -> Self {
        Self {
            context: None,
            _phantom: PhantomData,
        }
    }
}

/// Event emitted when settings have been successfully loaded.
///
/// Listen for this event to react to settings changes.
#[derive(Debug, Clone, Message)]
pub struct SettingsLoaded<T: Settings> {
    /// The loaded settings value
    pub settings: T,
    /// The path the settings were loaded from
    pub path: SettingsPath,
}

impl<T: Settings> SettingsLoaded<T> {
    /// Create a new SettingsLoaded event.
    pub fn new(settings: T, path: SettingsPath) -> Self {
        Self { settings, path }
    }
}

/// Event emitted when settings have been successfully saved.
#[derive(Debug, Clone, Message)]
pub struct SettingsSaved<T: Settings> {
    /// The path the settings were saved to
    pub path: SettingsPath,
    _phantom: PhantomData<T>,
}

impl<T: Settings> SettingsSaved<T> {
    /// Create a new SettingsSaved event.
    pub fn new(path: SettingsPath) -> Self {
        Self {
            path,
            _phantom: PhantomData,
        }
    }
}

/// Event emitted when settings failed to load.
#[derive(Debug, Clone, Message)]
pub struct SettingsLoadFailed<T: Settings> {
    /// The error message
    pub error_message: String,
    /// The path that was attempted
    pub path: SettingsPath,
    _phantom: PhantomData<T>,
}

impl<T: Settings> SettingsLoadFailed<T> {
    /// Create a new SettingsLoadFailed event.
    pub fn new(error_message: impl Into<String>, path: SettingsPath) -> Self {
        Self {
            error_message: error_message.into(),
            path,
            _phantom: PhantomData,
        }
    }
}

/// Event emitted when settings failed to save.
#[derive(Debug, Clone, Message)]
pub struct SettingsSaveFailed<T: Settings> {
    /// The error message
    pub error_message: String,
    /// The path that was attempted
    pub path: SettingsPath,
    _phantom: PhantomData<T>,
}

impl<T: Settings> SettingsSaveFailed<T> {
    /// Create a new SettingsSaveFailed event.
    pub fn new(error_message: impl Into<String>, path: SettingsPath) -> Self {
        Self {
            error_message: error_message.into(),
            path,
            _phantom: PhantomData,
        }
    }
}

/// Event to force reload settings from disk.
///
/// Unlike [`LoadSettings`], this event will reload settings even if
/// they haven't changed on disk, and will not emit a [`SettingsLoaded`]
/// event if the loaded settings are identical to the current ones.
#[derive(Debug, Clone)]
pub struct ReloadSettings<T: Settings, C = ()> {
    context: Option<C>,
    force: bool,
    _phantom: PhantomData<T>,
}

impl<T: Settings> ReloadSettings<T, ()> {
    /// Create a new reload event for static settings.
    pub fn new() -> Self {
        Self {
            context: None,
            force: false,
            _phantom: PhantomData,
        }
    }

    /// Create a forced reload event that always emits [`SettingsLoaded`].
    pub fn forced() -> Self {
        Self {
            context: None,
            force: true,
            _phantom: PhantomData,
        }
    }
}

impl<T: Settings, C: PathContext> ReloadSettings<T, C> {
    /// Create a new reload event with context for dynamic settings.
    pub fn with_context(context: C) -> Self {
        Self {
            context: Some(context),
            force: false,
            _phantom: PhantomData,
        }
    }

    /// Create a forced reload event with context.
    pub fn forced_with_context(context: C) -> Self {
        Self {
            context: Some(context),
            force: true,
            _phantom: PhantomData,
        }
    }

    /// Get the context, if any.
    pub fn context(&self) -> Option<&C> {
        self.context.as_ref()
    }

    /// Returns true if this is a forced reload.
    pub fn is_forced(&self) -> bool {
        self.force
    }
}

impl<T: Settings, C> Default for ReloadSettings<T, C> {
    fn default() -> Self {
        Self {
            context: None,
            force: false,
            _phantom: PhantomData,
        }
    }
}

/// Event to request saving all registered settings.
///
/// This is useful for "save all" functionality, e.g., when the game exits.
#[derive(Debug, Clone, Default)]
pub struct SaveAllSettings;

/// Event to request loading all registered settings.
///
/// This is useful for "load all" functionality, e.g., when the game starts.
#[derive(Debug, Clone, Default)]
pub struct LoadAllSettings;
