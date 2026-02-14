use crate::error::{SettingsError, SettingsResult};
use crate::format::Format;
use crate::path::SettingsPath;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

/// Trait for asynchronous settings storage backends.
///
/// Implement this trait to provide custom storage backends (e.g., cloud storage,
/// encrypted storage, etc.).
#[async_trait]
pub trait SettingsStorage: Send + Sync + 'static {
    /// Loads raw bytes from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to load from
    ///
    /// # Errors
    ///
    /// Returns `SettingsError::NotFound` if the file doesn't exist,
    /// or `SettingsError::Io` for other I/O errors.
    async fn load(&self, path: &SettingsPath) -> SettingsResult<Vec<u8>>;

    /// Saves raw bytes to the specified path.
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save to
    /// * `data` - The bytes to save
    ///
    /// # Errors
    ///
    /// Returns `SettingsError::Io` if the operation fails.
    async fn save(&self, path: &SettingsPath, data: &[u8]) -> SettingsResult<()>;

    /// Checks if a file exists at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    async fn exists(&self, path: &SettingsPath) -> bool;

    /// Deletes the file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to delete
    ///
    /// # Errors
    ///
    /// Returns `SettingsError::Io` if the operation fails.
    async fn delete(&self, path: &SettingsPath) -> SettingsResult<()>;
}

/// Standard async file system storage.
///
/// This is the default storage backend that reads/writes files
/// relative to the executable directory.
#[derive(Debug, Clone)]
pub struct AsyncFileStorage {
    base_path: std::path::PathBuf,
}

impl AsyncFileStorage {
    /// Creates a new file storage with the executable's directory as base.
    pub fn new() -> Self {
        let base_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        Self { base_path }
    }

    /// Creates a new file storage with a custom base path.
    pub fn with_base_path<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Returns the base path for this storage.
    pub fn base_path(&self) -> &std::path::Path {
        &self.base_path
    }

    /// Resolves a settings path to a full filesystem path.
    fn resolve_path(&self, path: &SettingsPath) -> std::path::PathBuf {
        self.base_path.join(path.as_str())
    }
}

impl Default for AsyncFileStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SettingsStorage for AsyncFileStorage {
    async fn load(&self, path: &SettingsPath) -> SettingsResult<Vec<u8>> {
        let full_path = self.resolve_path(path);

        if !self.exists(path).await {
            return Err(SettingsError::NotFound(full_path));
        }

        fs::read(&full_path).await.map_err(SettingsError::from)
    }

    async fn save(&self, path: &SettingsPath, data: &[u8]) -> SettingsResult<()> {
        let full_path = self.resolve_path(path);

        // Ensure parent directories exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&full_path, data)
            .await
            .map_err(SettingsError::from)
    }

    async fn exists(&self, path: &SettingsPath) -> bool {
        let full_path = self.resolve_path(path);
        fs::metadata(&full_path).await.is_ok()
    }

    async fn delete(&self, path: &SettingsPath) -> SettingsResult<()> {
        let full_path = self.resolve_path(path);
        fs::remove_file(&full_path)
            .await
            .map_err(SettingsError::from)
    }
}

/// Storage handle that wraps the storage backend and provides convenience methods.
#[derive(Clone)]
pub struct StorageHandle {
    storage: std::sync::Arc<dyn SettingsStorage>,
}

impl StorageHandle {
    /// Creates a new storage handle from a storage backend.
    pub fn new<S: SettingsStorage>(storage: S) -> Self {
        Self {
            storage: std::sync::Arc::new(storage),
        }
    }

    /// Creates a default file storage handle.
    pub fn default_file() -> Self {
        Self::new(AsyncFileStorage::new())
    }

    /// Loads and deserializes settings from the specified path.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize
    ///
    /// # Arguments
    ///
    /// * `path` - The path to load from
    pub async fn load<T: serde::de::DeserializeOwned>(
        &self,
        path: &SettingsPath,
    ) -> SettingsResult<T> {
        let data = self.storage.load(path).await?;
        let format = Format::from_path(path)?;
        format.deserialize(&data)
    }

    /// Serializes and saves settings to the specified path.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to serialize
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save to
    /// * `value` - The value to serialize
    pub async fn save<T: serde::Serialize>(
        &self,
        path: &SettingsPath,
        value: &T,
    ) -> SettingsResult<()> {
        let format = Format::from_path(path)?;
        let data = format.serialize(value)?;
        self.storage.save(path, &data).await
    }

    /// Returns a reference to the underlying storage backend.
    pub fn storage(&self) -> &dyn SettingsStorage {
        &*self.storage
    }

    /// Loads raw bytes from the specified path without deserialization.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to load from
    pub async fn load_raw(&self, path: &SettingsPath) -> SettingsResult<Vec<u8>> {
        self.storage.load(path).await
    }

    /// Saves raw bytes to the specified path without serialization.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save to
    /// * `data` - The bytes to save
    pub async fn save_raw(&self, path: &SettingsPath, data: &[u8]) -> SettingsResult<()> {
        self.storage.save(path, data).await
    }

    /// Checks if a file exists at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    pub async fn exists(&self, path: &SettingsPath) -> bool {
        self.storage.exists(path).await
    }
}

impl Default for StorageHandle {
    fn default() -> Self {
        Self::default_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_storage_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage = AsyncFileStorage::with_base_path(temp_dir.path());

        let path = SettingsPath::new("test/settings.toml").unwrap();
        let data = b"test = true";

        // Save
        storage.save(&path, data).await.unwrap();

        // Load
        let loaded = storage.load(&path).await.unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn test_file_storage_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = AsyncFileStorage::with_base_path(temp_dir.path());

        let path = SettingsPath::new("test/file.toml").unwrap();

        assert!(!storage.exists(&path).await);

        storage.save(&path, b"test").await.unwrap();

        assert!(storage.exists(&path).await);
    }

    #[tokio::test]
    async fn test_file_storage_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let storage = AsyncFileStorage::with_base_path(temp_dir.path());

        let path = SettingsPath::new("nonexistent.toml").unwrap();

        let result = storage.load(&path).await;
        assert!(matches!(result, Err(SettingsError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_storage_handle_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let handle = StorageHandle::new(AsyncFileStorage::with_base_path(temp_dir.path()));

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let path = SettingsPath::new("data.toml").unwrap();

        handle.save(&path, &data).await.unwrap();
        let loaded: TestData = handle.load(&path).await.unwrap();

        assert_eq!(data, loaded);
    }
}
