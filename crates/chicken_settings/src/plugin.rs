use {
    crate::{
        error::{SettingsError, SettingsResult},
        events::{SettingsLoadFailed, SettingsLoaded, SettingsSaveFailed, SettingsSaved},
        format::Format,
        path::{PathContext, SettingsPath},
        storage::{AsyncFileStorage, StorageHandle},
        {DeltaStorage, Settings},
    },
    bevy_app::{App, Plugin, Update},
    bevy_ecs::{message::Messages, prelude::*},
    bevy_log::{debug, error, info},
    camino::Utf8PathBuf,
    chicken_settings_edit::SurgicalEditor,
    crossbeam_channel::{Receiver, Sender},
    std::{
        any::{Any, TypeId},
        collections::HashMap,
        sync::Arc,
    },
};

/// Result from an async settings operation (load or save).
pub struct SettingsCommandResult {
    /// The type ID of the settings type
    pub type_id: TypeId,
    /// The type name for logging
    pub type_name: &'static str,
    /// The path that was operated on
    pub path: Utf8PathBuf,
    /// Whether this was a save operation (true) or load operation (false)
    pub is_save: bool,
    /// The result: for load, contains the loaded settings; for save, contains ()
    pub result: Result<Box<dyn Any + Send + Sync>, SettingsError>,
}

/// Type alias for the result channel sender.
pub type SettingsResultSender = Sender<SettingsCommandResult>;
/// Type alias for the result channel receiver.
pub type SettingsResultReceiver = Receiver<SettingsCommandResult>;

/// The main plugin for settings management.
#[derive(Default)]
pub struct ChickenSettingsPlugin;

impl Plugin for ChickenSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsStorageResource>();
        app.init_resource::<SettingsRegistry>();
        app.init_resource::<SettingsLoader>();

        // Process async settings results
        app.add_systems(Update, process_settings_results);
    }
}

/// Resource that holds the storage backend.
#[derive(Resource, Clone)]
pub struct SettingsStorageResource {
    storage: StorageHandle,
}

impl SettingsStorageResource {
    /// Creates a new storage resource with default file storage.
    pub fn new() -> Self {
        Self {
            storage: StorageHandle::default_file(),
        }
    }

    /// Creates a new storage resource with a custom base path.
    pub fn with_base_path<P: AsRef<std::path::Path>>(base_path: P) -> Self {
        Self {
            storage: StorageHandle::new(AsyncFileStorage::with_base_path(base_path)),
        }
    }

    /// Returns a reference to the storage handle.
    pub fn storage(&self) -> &StorageHandle {
        &self.storage
    }
}

impl Default for SettingsStorageResource {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory function to create an insert callback for a settings type.
pub(crate) type InsertFnFactory = Box<
    dyn Fn() -> Box<dyn FnOnce(&mut World, Box<dyn Any + Send + Sync>) + Send + Sync> + Send + Sync,
>;

/// Type-erased callback to emit a SettingsLoaded event.
/// Wrapped in Arc for Clone support.
pub(crate) type EmitLoadedFn =
    Arc<dyn Fn(&mut World, &Box<dyn Any + Send + Sync>, SettingsPath) + Send + Sync>;

/// Type-erased callback to emit a SettingsLoadFailed event.
/// Wrapped in Arc for Clone support.
pub(crate) type EmitLoadFailedFn = Arc<dyn Fn(&mut World, String, SettingsPath) + Send + Sync>;

/// Type-erased callback to emit a SettingsSaved event.
/// Wrapped in Arc for Clone support.
pub(crate) type EmitSavedFn = Arc<dyn Fn(&mut World, SettingsPath) + Send + Sync>;

/// Type-erased callback to emit a SettingsSaveFailed event.
/// Wrapped in Arc for Clone support.
pub(crate) type EmitSaveFailedFn = Arc<dyn Fn(&mut World, String, SettingsPath) + Send + Sync>;

/// Information about a registered settings type.
pub struct SettingsTypeInfo {
    /// The type name for logging
    pub type_name: &'static str,
    /// The path template (may contain placeholders)
    pub path_template: &'static str,
    /// Whether the path has dynamic placeholders
    pub has_dynamic_path: bool,
    /// The context type ID (if dynamic)
    pub context_type_id: Option<TypeId>,
    /// The format detected from extension
    pub format: Format,
    /// Factory to create an insert callback for loading resources
    pub(crate) insert_factory: Option<InsertFnFactory>,
    /// Callback to emit SettingsLoaded event
    pub(crate) emit_loaded_fn: Option<EmitLoadedFn>,
    /// Callback to emit SettingsLoadFailed event
    pub(crate) emit_load_failed_fn: Option<EmitLoadFailedFn>,
    /// Callback to emit SettingsSaved event
    pub(crate) emit_saved_fn: Option<EmitSavedFn>,
    /// Callback to emit SettingsSaveFailed event
    pub(crate) emit_save_failed_fn: Option<EmitSaveFailedFn>,
}

impl std::fmt::Debug for SettingsTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsTypeInfo")
            .field("type_name", &self.type_name)
            .field("path_template", &self.path_template)
            .field("has_dynamic_path", &self.has_dynamic_path)
            .field("context_type_id", &self.context_type_id)
            .field("format", &self.format)
            .field("insert_factory", &self.insert_factory.is_some())
            .field("emit_loaded_fn", &self.emit_loaded_fn.is_some())
            .field("emit_load_failed_fn", &self.emit_load_failed_fn.is_some())
            .field("emit_saved_fn", &self.emit_saved_fn.is_some())
            .field("emit_save_failed_fn", &self.emit_save_failed_fn.is_some())
            .finish()
    }
}

/// Registry of all registered settings types.
#[derive(Resource, Default)]
pub struct SettingsRegistry {
    registered_types: HashMap<TypeId, SettingsTypeInfo>,
    /// Map from context type ID to settings types that use it
    context_to_settings: HashMap<TypeId, Vec<TypeId>>,
}

impl SettingsRegistry {
    /// Register a static settings type.
    pub fn register_static<T: Settings + Clone>(&mut self) {
        let type_id = TypeId::of::<T>();
        let info = SettingsTypeInfo {
            type_name: T::type_name(),
            path_template: T::path_template(),
            has_dynamic_path: T::has_dynamic_path(),
            context_type_id: None,
            format: T::format(),
            insert_factory: Some(Self::create_insert_factory::<T>()),
            emit_loaded_fn: Some(Self::create_emit_loaded_fn::<T>()),
            emit_load_failed_fn: Some(Self::create_emit_load_failed_fn::<T>()),
            emit_saved_fn: Some(Self::create_emit_saved_fn::<T>()),
            emit_save_failed_fn: Some(Self::create_emit_save_failed_fn::<T>()),
        };
        self.registered_types.insert(type_id, info);
    }

    fn create_insert_factory<T: Settings>() -> InsertFnFactory {
        Box::new(|| {
            Box::new(|world: &mut World, data: Box<dyn Any + Send + Sync>| {
                if let Ok(settings) = data.downcast::<T>() {
                    world.insert_resource(*settings);
                }
            })
        })
    }

    fn create_emit_loaded_fn<T: Settings + Clone>() -> EmitLoadedFn {
        Arc::new(
            |world: &mut World, data: &Box<dyn Any + Send + Sync>, path: SettingsPath| {
                if let Some(settings) = data.downcast_ref::<T>() {
                    let event = SettingsLoaded::new(settings.clone(), path);
                    if let Some(mut messages) =
                        world.get_resource_mut::<Messages<SettingsLoaded<T>>>()
                    {
                        messages.write(event);
                    }
                }
            },
        )
    }

    fn create_emit_load_failed_fn<T: Settings>() -> EmitLoadFailedFn {
        Arc::new(
            |world: &mut World, error_message: String, path: SettingsPath| {
                let event = SettingsLoadFailed::<T>::new(error_message, path);
                if let Some(mut messages) =
                    world.get_resource_mut::<Messages<SettingsLoadFailed<T>>>()
                {
                    messages.write(event);
                }
            },
        )
    }

    fn create_emit_saved_fn<T: Settings>() -> EmitSavedFn {
        Arc::new(|world: &mut World, path: SettingsPath| {
            let event = SettingsSaved::<T>::new(path);
            if let Some(mut messages) = world.get_resource_mut::<Messages<SettingsSaved<T>>>() {
                messages.write(event);
            }
        })
    }

    fn create_emit_save_failed_fn<T: Settings>() -> EmitSaveFailedFn {
        Arc::new(
            |world: &mut World, error_message: String, path: SettingsPath| {
                let event = SettingsSaveFailed::<T>::new(error_message, path);
                if let Some(mut messages) =
                    world.get_resource_mut::<Messages<SettingsSaveFailed<T>>>()
                {
                    messages.write(event);
                }
            },
        )
    }

    /// Register a dynamic settings type with context.
    ///
    /// The settings will use the context to resolve dynamic paths.
    /// Note: Auto-reload has been removed. The game must call `load<T>()`
    /// explicitly when the context changes.
    pub fn register_dynamic<T, C>(&mut self)
    where
        T: Settings<Context = C> + Clone,
        C: PathContext + Resource,
    {
        let type_id = TypeId::of::<T>();
        let context_id = TypeId::of::<C>();

        let info = SettingsTypeInfo {
            type_name: T::type_name(),
            path_template: T::path_template(),
            has_dynamic_path: true,
            context_type_id: Some(context_id),
            format: T::format(),
            insert_factory: Some(Self::create_insert_factory::<T>()),
            emit_loaded_fn: Some(Self::create_emit_loaded_fn::<T>()),
            emit_load_failed_fn: Some(Self::create_emit_load_failed_fn::<T>()),
            emit_saved_fn: Some(Self::create_emit_saved_fn::<T>()),
            emit_save_failed_fn: Some(Self::create_emit_save_failed_fn::<T>()),
        };

        self.registered_types.insert(type_id, info);
        self.context_to_settings
            .entry(context_id)
            .or_default()
            .push(type_id);

        info!(
            "Registered dynamic settings '{}' with context",
            T::type_name()
        );
    }

    /// Get info for a registered type.
    pub fn get_info(&self, type_id: TypeId) -> Option<&SettingsTypeInfo> {
        self.registered_types.get(&type_id)
    }

    /// Get all settings types that use a given context type.
    pub fn get_settings_for_context(&self, context_id: TypeId) -> &[TypeId] {
        self.context_to_settings
            .get(&context_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check if a type is registered.
    pub fn is_registered(&self, type_id: TypeId) -> bool {
        self.registered_types.contains_key(&type_id)
    }
}

/// Extension trait for App to add settings functionality.
pub trait SettingsAppExt {
    /// Unified method for registering static settings (no context).
    ///
    /// For static settings:
    /// ```ignore
    /// app.add_settings::<AudioSettings>();
    /// ```
    fn add_settings<T>(&mut self) -> &mut Self
    where
        T: Settings<Context = ()> + PartialEq + Send + Sync + 'static;

    /// Unified method for registering dynamic settings with context.
    ///
    /// For dynamic settings (with context):
    /// ```ignore
    /// app.add_settings_with_context::<PlayerSettings, SaveContext>();
    /// ```
    fn add_settings_with_context<T, C>(&mut self) -> &mut Self
    where
        T: Settings<Context = C> + PartialEq + Send + Sync + 'static,
        C: PathContext + Resource + Default;
}

impl SettingsAppExt for App {
    fn add_settings<T>(&mut self) -> &mut Self
    where
        T: Settings<Context = ()> + PartialEq + Send + Sync + 'static,
    {
        // Register static settings in the registry
        if let Some(mut registry) = self.world_mut().get_resource_mut::<SettingsRegistry>() {
            registry.register_static::<T>();
        }

        // Insert the resource
        if self.world().get_resource::<T>().is_none() {
            self.world_mut().insert_resource(T::default());
        }

        self
    }

    fn add_settings_with_context<T, C>(&mut self) -> &mut Self
    where
        T: Settings<Context = C> + PartialEq + Send + Sync + 'static,
        C: PathContext + Resource + Default,
    {
        let context_name = std::any::type_name::<C>();
        let settings_name = T::type_name();

        debug!(
            "Adding settings '{}' with context '{}'",
            settings_name, context_name
        );

        // Register dynamic settings in the registry
        if let Some(mut registry) = self.world_mut().get_resource_mut::<SettingsRegistry>() {
            registry.register_dynamic::<T, C>();
        } else {
            error!("SettingsRegistry resource not found - plugin may not be initialized");
        }

        // Insert the resource
        if self.world().get_resource::<T>().is_none() {
            debug!("Inserting default resource for '{}'", settings_name);
            self.world_mut().insert_resource(T::default());
        } else {
            debug!(
                "Resource for '{}' already exists, not inserting default",
                settings_name
            );
        }

        self
    }
}

/// Resource that provides convenient access to settings operations.
#[derive(Resource)]
pub struct SettingsLoader {
    storage: StorageHandle,
    /// Shared Tokio runtime for async operations
    runtime: Option<tokio::runtime::Runtime>,
    /// Channel sender for async results
    result_tx: SettingsResultSender,
    /// Channel receiver for async results
    result_rx: SettingsResultReceiver,
}

impl Default for SettingsLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsLoader {
    /// Create a new settings loader with default file storage.
    pub fn new() -> Self {
        let runtime = tokio::runtime::Runtime::new().ok();
        if runtime.is_none() {
            error!("Failed to create Tokio runtime for SettingsLoader");
        }
        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        Self {
            storage: StorageHandle::default_file(),
            runtime,
            result_tx,
            result_rx,
        }
    }

    /// Create a new settings loader with a custom base path.
    pub fn with_base_path<P: AsRef<std::path::Path>>(base_path: P) -> Self {
        let runtime = tokio::runtime::Runtime::new().ok();
        if runtime.is_none() {
            error!("Failed to create Tokio runtime for SettingsLoader");
        }
        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        Self {
            storage: StorageHandle::new(AsyncFileStorage::with_base_path(base_path)),
            runtime,
            result_tx,
            result_rx,
        }
    }

    /// Get a clone of the result sender for use in async tasks.
    fn result_sender(&self) -> SettingsResultSender {
        self.result_tx.clone()
    }

    /// Load settings asynchronously.
    ///
    /// Returns immediately and runs the load operation in the background.
    /// The loaded settings will be inserted into the world automatically.
    pub fn load<T>(&self)
    where
        T: Settings + PartialEq + Send + Sync + 'static,
    {
        let path_template = T::path_template();
        let _format = T::format();

        let path = match SettingsPath::new(path_template) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to create path for {}: {}", T::type_name(), e);
                return;
            }
        };

        let storage = self.storage.clone();
        let result_tx = self.result_sender();
        let type_id = TypeId::of::<T>();
        let type_name = T::type_name();
        let path_buf: Utf8PathBuf = path.clone().into();

        // Spawn the task using the shared runtime
        if let Some(rt) = self.runtime.as_ref() {
            rt.spawn(async move {
                match storage.load::<T>(&path).await {
                    Ok(settings) => {
                        debug!("Successfully loaded settings for {}", type_name);
                        // Send result back through channel
                        let result = SettingsCommandResult {
                            type_id,
                            type_name,
                            path: path_buf,
                            is_save: false,
                            result: Ok(Box::new(settings)),
                        };
                        if result_tx.send(result).is_err() {
                            error!("Failed to send load result for {}", type_name);
                        }
                    }
                    Err(SettingsError::NotFound(_)) => {
                        debug!("Settings file not found for {}, using defaults", type_name);
                        // Send default settings back through channel
                        let settings = T::default();
                        let result = SettingsCommandResult {
                            type_id,
                            type_name,
                            path: path_buf,
                            is_save: false,
                            result: Ok(Box::new(settings)),
                        };
                        if result_tx.send(result).is_err() {
                            error!("Failed to send load result for {}", type_name);
                        }
                    }
                    Err(e) => {
                        error!("Failed to load settings for {}: {}", type_name, e);
                        // Send error back through channel
                        let result = SettingsCommandResult {
                            type_id,
                            type_name,
                            path: path_buf,
                            is_save: false,
                            result: Err(e),
                        };
                        if result_tx.send(result).is_err() {
                            error!("Failed to send load error for {}", type_name);
                        }
                    }
                }
            });
        } else {
            error!(
                "Cannot load {}: Tokio runtime not available",
                T::type_name()
            );
        }
    }

    /// Load settings synchronously (blocking).
    ///
    /// Waits for the load operation to complete before returning.
    /// Returns the loaded settings, or default if not found.
    pub fn load_now<T>(&self) -> SettingsResult<T>
    where
        T: Settings,
    {
        let block = async {
            let path = SettingsPath::new(T::path_template())?;

            match self.storage.load::<T>(&path).await {
                Ok(settings) => {
                    info!("Loaded settings for {} (synchronous)", T::type_name());
                    Ok(settings)
                }
                Err(SettingsError::NotFound(_)) => {
                    info!(
                        "Settings file not found for {}, using defaults",
                        T::type_name()
                    );
                    Ok(T::default())
                }
                Err(e) => Err(e),
            }
        };

        if let Some(rt) = self.runtime.as_ref() {
            rt.block_on(block)
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(block)
        }
    }

    /// Save settings asynchronously.
    ///
    /// Returns immediately and runs the save operation in the background.
    /// Uses surgical TOML editing for existing files to preserve comments and formatting.
    pub fn save<T>(&self, settings: &T)
    where
        T: Settings + PartialEq + Send + Sync + Clone + 'static,
    {
        let path_template = T::path_template();
        let path = match SettingsPath::new(path_template) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to create path for {}: {}", T::type_name(), e);
                return;
            }
        };

        let storage = self.storage.clone();
        let settings = settings.clone();
        let result_tx = self.result_sender();
        let type_id = TypeId::of::<T>();
        let type_name = T::type_name();
        let path_buf: Utf8PathBuf = path.clone().into();

        // Spawn the task using the shared runtime
        if let Some(rt) = self.runtime.as_ref() {
            rt.spawn(async move {
                let save_result: Result<(), SettingsError> = async {
                    // Check if file exists
                    let file_exists = storage.exists(&path).await;

                    if file_exists {
                        // SURGICAL EDIT: Read existing, update changed fields only
                        match storage.load_raw(&path).await {
                            Ok(bytes) => {
                                let mut content = String::from_utf8_lossy(&bytes).to_string();

                                // Get changed fields from delta storage
                                let delta = DeltaStorage::<T>::new();
                                let changed = delta.get_changed_fields(&settings);

                                if changed.is_empty() {
                                    debug!("No changes to save for {}", type_name);
                                    Ok(())
                                } else {
                                    // For each changed field, surgically update
                                    let editor = chicken_settings_edit::TomlEditor::new();
                                    for (key_path, value) in changed {
                                        let path_vec: Vec<&str> = key_path.split('.').collect();
                                        if let Err(e) =
                                            editor.update_value(&mut content, &path_vec, &value)
                                        {
                                            error!("Failed to update {}: {}", key_path, e);
                                        }
                                    }

                                    // Write back
                                    storage
                                        .save_raw(&path, content.as_bytes())
                                        .await
                                        .map_err(|e| {
                                            error!("Failed to save: {}", e);
                                            e
                                        })
                                }
                            }
                            Err(e) => {
                                // Fallback to full rewrite on error
                                error!(
                                    "Failed to load for surgical edit, using full rewrite: {}",
                                    e
                                );
                                let format = T::format();
                                let data = {
                                    let delta_storage = DeltaStorage::<T>::new();
                                    delta_storage.serialize(&settings, format).map_err(|e| {
                                        error!(
                                            "Failed to serialize settings for {}: {}",
                                            type_name, e
                                        );
                                        SettingsError::Serialization(e.to_string())
                                    })?
                                };

                                storage.storage().save(&path, &data).await
                            }
                        }
                    } else {
                        // NEW FILE: Full serialization (no existing content to preserve)
                        let format = T::format();
                        let data = {
                            let delta_storage = DeltaStorage::<T>::new();
                            delta_storage.serialize(&settings, format).map_err(|e| {
                                error!("Failed to serialize settings for {}: {}", type_name, e);
                                SettingsError::Serialization(e.to_string())
                            })?
                        };

                        storage.storage().save(&path, &data).await
                    }
                }
                .await;

                // Send result back through channel
                let result = SettingsCommandResult {
                    type_id,
                    type_name,
                    path: path_buf,
                    is_save: true,
                    result: save_result.map(|_| Box::new(()) as Box<dyn Any + Send + Sync>),
                };
                if result_tx.send(result).is_err() {
                    error!("Failed to send save result for {}", type_name);
                }
            });
        } else {
            error!(
                "Cannot save {}: Tokio runtime not available",
                T::type_name()
            );
        }
    }

    /// Save settings synchronously (blocking).
    ///
    /// Waits for the save operation to complete before returning.
    /// Uses surgical TOML editing for existing files to preserve comments and formatting.
    pub fn save_now<T>(&self, settings: &T) -> SettingsResult<()>
    where
        T: Settings + PartialEq + Clone,
    {
        let path = SettingsPath::new(T::path_template())?;

        let rt = tokio::runtime::Runtime::new()?;
        let result = rt.block_on(async {
            // Check if file exists
            let file_exists = self.storage.exists(&path).await;

            let save_result = if file_exists {
                // SURGICAL EDIT: Read existing, update changed fields only
                match self.storage.load_raw(&path).await {
                    Ok(bytes) => {
                        let mut content = String::from_utf8_lossy(&bytes).to_string();

                        // Get changed fields from delta storage
                        let delta = DeltaStorage::<T>::new();
                        let changed = delta.get_changed_fields(&settings);

                        if changed.is_empty() {
                            debug!("No changes to save for {}", T::type_name());
                            Ok(())
                        } else {
                            // For each changed field, surgically update
                            let editor = chicken_settings_edit::TomlEditor::new();
                            for (key_path, value) in changed {
                                let path_vec: Vec<&str> = key_path.split('.').collect();
                                if let Err(e) = editor.update_value(&mut content, &path_vec, &value)
                                {
                                    error!("Failed to update {}: {}", key_path, e);
                                }
                            }

                            // Write back
                            self.storage
                                .save_raw(&path, content.as_bytes())
                                .await
                                .map_err(|e| {
                                    error!("Failed to save: {}", e);
                                    e
                                })
                        }
                    }
                    Err(e) => {
                        // Fallback to full rewrite on error
                        error!(
                            "Failed to load for surgical edit, using full rewrite: {}",
                            e
                        );
                        let format = T::format();
                        let data = {
                            let delta_storage = DeltaStorage::<T>::new();
                            delta_storage.serialize(settings, format)?
                        };

                        self.storage.storage().save(&path, &data).await
                    }
                }
            } else {
                // NEW FILE: Full serialization (no existing content to preserve)
                let format = T::format();
                let data = {
                    let delta_storage = DeltaStorage::<T>::new();
                    delta_storage.serialize(settings, format)?
                };

                self.storage.storage().save(&path, &data).await
            };

            save_result
        });

        if result.is_ok() {
            info!("Saved settings for {} (synchronous)", T::type_name());
        }

        result
    }
}

/// System that processes async settings operation results and updates the world.
///
/// This system polls the result channel and:
/// - For successful loads: inserts the resource into the world and emits SettingsLoaded event
/// - For failed loads: emits SettingsLoadFailed event
/// - For successful saves: emits SettingsSaved event
/// - For failed saves: emits SettingsSaveFailed event
fn process_settings_results(world: &mut World) {
    // Collect results first to avoid borrowing issues
    let results: Vec<SettingsCommandResult> = {
        let loader = world
            .get_resource::<SettingsLoader>()
            .expect("SettingsLoader resource not found");
        loader.result_rx.try_iter().collect()
    };

    for result in results {
        // Convert path to SettingsPath for events
        let path = match SettingsPath::from_path_buf(&result.path) {
            Ok(p) => p,
            Err(_) => {
                error!("Invalid path for settings result: {}", result.path);
                continue;
            }
        };

        if result.is_save {
            // Handle save results
            match result.result {
                Ok(_) => {
                    debug!("Settings saved successfully: {}", result.type_name);
                    // Emit SettingsSaved event - get callback in a separate scope
                    {
                        let emit_fn = world
                            .get_resource::<SettingsRegistry>()
                            .and_then(|registry| registry.get_info(result.type_id))
                            .and_then(|info| info.emit_saved_fn.as_ref())
                            .map(|f| f.clone());
                        if let Some(emit_fn) = emit_fn {
                            emit_fn(world, path);
                        }
                    }
                }
                Err(ref e) => {
                    let error_message = e.to_string();
                    error!("Settings save failed for {}: {}", result.type_name, e);
                    // Emit SettingsSaveFailed event - get callback in a separate scope
                    {
                        let emit_fn = world
                            .get_resource::<SettingsRegistry>()
                            .and_then(|registry| registry.get_info(result.type_id))
                            .and_then(|info| info.emit_save_failed_fn.as_ref())
                            .map(|f| f.clone());
                        if let Some(emit_fn) = emit_fn {
                            emit_fn(world, error_message, path);
                        }
                    }
                }
            }
        } else {
            // Handle load results
            match result.result {
                Ok(data) => {
                    debug!("Settings loaded successfully: {}", result.type_name);
                    // Get callbacks in a separate scope to release borrow before using world
                    let (insert_fn, emit_loaded_fn) = {
                        let registry_info = world
                            .get_resource::<SettingsRegistry>()
                            .and_then(|registry| registry.get_info(result.type_id));

                        let insert_fn = registry_info
                            .and_then(|info| info.insert_factory.as_ref())
                            .map(|factory| factory());
                        let emit_loaded_fn = registry_info
                            .and_then(|info| info.emit_loaded_fn.as_ref())
                            .map(|f| f.clone());

                        (insert_fn, emit_loaded_fn)
                    };

                    // Emit SettingsLoaded event FIRST (uses reference, doesn't move data)
                    if let Some(emit_fn) = emit_loaded_fn {
                        emit_fn(world, &data, path);
                    }

                    // Then insert resource into world using the type-erased callback
                    if let Some(insert_fn) = insert_fn {
                        insert_fn(world, data);
                    } else {
                        error!(
                            "No insert callback for type {} - was it registered?",
                            result.type_name
                        );
                    }
                }
                Err(ref e) => {
                    let error_message = e.to_string();
                    error!("Settings load failed for {}: {}", result.type_name, e);
                    // Emit SettingsLoadFailed event - get callback in a separate scope
                    {
                        let emit_fn = world
                            .get_resource::<SettingsRegistry>()
                            .and_then(|registry| registry.get_info(result.type_id))
                            .and_then(|info| info.emit_load_failed_fn.as_ref())
                            .map(|f| f.clone());
                        if let Some(emit_fn) = emit_fn {
                            emit_fn(world, error_message, path);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Resource)]
    struct TestSettings {
        value: i32,
    }

    impl Settings for TestSettings {
        type Context = ();

        fn type_name() -> &'static str {
            "TestSettings"
        }

        fn path_template() -> &'static str {
            "test/settings.toml"
        }

        fn has_dynamic_path() -> bool {
            false
        }

        fn format() -> Format {
            Format::Toml
        }

        fn into_box(self) -> Box<dyn std::any::Any + Send + Sync> {
            Box::new(self)
        }

        fn clone_box(&self) -> Box<dyn std::any::Any + Send + Sync> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_settings_registry() {
        let mut registry = SettingsRegistry::default();
        registry.register_static::<TestSettings>();

        assert!(registry.is_registered(TypeId::of::<TestSettings>()));

        let info = registry.get_info(TypeId::of::<TestSettings>()).unwrap();
        assert_eq!(info.type_name, "TestSettings");
        assert!(!info.has_dynamic_path);
    }

    #[test]
    fn test_settings_loader_creation() {
        let _loader = SettingsLoader::new();
    }

    #[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Resource)]
    struct TestSurgicalSettings {
        volume: i32,
        enabled: bool,
        name: String,
    }

    impl Settings for TestSurgicalSettings {
        type Context = ();

        fn type_name() -> &'static str {
            "TestSurgicalSettings"
        }

        fn path_template() -> &'static str {
            "test/surgical_settings.toml"
        }

        fn has_dynamic_path() -> bool {
            false
        }

        fn format() -> Format {
            Format::Toml
        }

        fn into_box(self) -> Box<dyn std::any::Any + Send + Sync> {
            Box::new(self)
        }

        fn clone_box(&self) -> Box<dyn std::any::Any + Send + Sync> {
            Box::new(self.clone())
        }
    }

    #[test]
    fn test_surgical_save_preserves_comments() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let loader = SettingsLoader::with_base_path(temp_dir.path());

        // Create initial file with comments
        let file_path = temp_dir.path().join("test/surgical_settings.toml");
        std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(
            file,
            r#"# This is a header comment
volume = 50  # Volume setting
enabled = false  # Enabled flag
name = "default"  # Name setting
"#
        )
        .unwrap();
        drop(file);

        // Modify only one field and save
        let settings = TestSurgicalSettings {
            volume: 75,
            enabled: false,
            name: "default".to_string(),
        };

        loader.save_now(&settings).unwrap();

        // Read back and verify surgical edit worked
        let content = std::fs::read_to_string(&file_path).unwrap();

        // Verify the key changes:
        // 1. Volume should be updated from 50 to 75
        assert!(
            content.contains("volume = 75"),
            "Volume should be updated from 50 to 75"
        );
        // 2. Inline comment should be preserved (toml_edit preserves inline comments)
        assert!(
            content.contains("# Volume setting"),
            "Inline comment should be preserved"
        );
        // 3. Unchanged field should remain
        assert!(
            content.contains("enabled = false"),
            "Unchanged field should remain"
        );
        assert!(
            content.contains("name = \"default\""),
            "Name should remain unchanged"
        );

        // Note: Header comments are not preserved by toml_edit when values change
        // This is a known limitation of the library
    }

    #[test]
    fn test_new_file_uses_full_serialization() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let loader = SettingsLoader::with_base_path(temp_dir.path());

        // Save to non-existent file
        let settings = TestSurgicalSettings {
            volume: 100,
            enabled: true,
            name: "test".to_string(),
        };

        loader.save_now(&settings).unwrap();

        // Verify file was created with full content
        let file_path = temp_dir.path().join("test/surgical_settings.toml");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("volume = 100"));
        assert!(content.contains("enabled = true"));
        assert!(content.contains("name = \"test\""));
    }
}
