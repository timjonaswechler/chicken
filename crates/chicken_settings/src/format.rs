use crate::error::{SettingsError, SettingsResult};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

/// TOML serialization format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// Tom's Obvious, Minimal Language (TOML)
    Toml,
}

impl Format {
    /// Returns the file extension for this format.
    pub fn extension(&self) -> &'static str {
        "toml"
    }

    /// Detects the format from a file path based on its extension.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to detect format from
    ///
    /// # Errors
    ///
    /// Returns `SettingsError::InvalidExtension` if the path has no extension
    /// or the extension is not recognized.
    ///
    /// # Example
    ///
    /// ```
    /// use chicken_settings::format::Format;
    /// use std::path::Path;
    ///
    /// let format = Format::from_path("settings.toml").unwrap();
    /// assert_eq!(format, Format::Toml);
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> SettingsResult<Self> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| SettingsError::InvalidExtension(path.to_string_lossy().to_string()))?;

        match ext.to_lowercase().as_str() {
            "toml" => Ok(Format::Toml),
            _ => Err(SettingsError::UnsupportedFormat(ext.to_string())),
        }
    }

    /// Serializes a value to bytes using TOML format.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to serialize, must implement `Serialize`
    ///
    /// # Arguments
    ///
    /// * `value` - The value to serialize
    ///
    /// # Errors
    ///
    /// Returns `SettingsError::Serialization` if serialization fails.
    pub fn serialize<T: Serialize>(&self, value: &T) -> SettingsResult<Vec<u8>> {
        toml::to_string_pretty(value)
            .map_err(|e| SettingsError::Serialization(e.to_string()))
            .map(|s| s.into_bytes())
    }

    /// Deserializes bytes from TOML format.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize, must implement `DeserializeOwned`
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to deserialize
    ///
    /// # Errors
    ///
    /// Returns `SettingsError::Deserialization` if deserialization fails.
    pub fn deserialize<T: DeserializeOwned>(&self, data: &[u8]) -> SettingsResult<T> {
        let s = String::from_utf8_lossy(data);
        toml::from_str(&s).map_err(|e| SettingsError::Deserialization(e.to_string()))
    }

    /// Returns true if this format supports delta storage.
    ///
    /// TOML always supports delta storage.
    pub fn supports_delta(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_format_from_path() {
        assert_eq!(Format::from_path("test.toml").unwrap(), Format::Toml);
    }

    #[test]
    fn test_format_extensions() {
        assert_eq!(Format::Toml.extension(), "toml");
    }

    #[test]
    fn test_serialize_deserialize_toml() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let bytes = Format::Toml.serialize(&data).unwrap();
        let decoded: TestData = Format::Toml.deserialize(&bytes).unwrap();
        assert_eq!(data, decoded);
    }
}
