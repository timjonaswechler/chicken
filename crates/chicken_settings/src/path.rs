use crate::error::{SettingsError, SettingsResult};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::HashMap;

/// A validated settings path.
///
/// This type ensures that paths are valid UTF-8 and provides utilities
/// for path manipulation and placeholder resolution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SettingsPath {
    inner: Utf8PathBuf,
}

impl SettingsPath {
    /// Creates a new settings path from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not valid UTF-8.
    pub fn new<S: AsRef<str>>(path: S) -> SettingsResult<Self> {
        let path_str = path.as_ref();
        let utf8_path = Utf8PathBuf::from(path_str);
        Ok(Self { inner: utf8_path })
    }

    /// Creates a new settings path from a path buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not valid UTF-8.
    pub fn from_path_buf<P: AsRef<std::path::Path>>(path: P) -> SettingsResult<Self> {
        let path = path.as_ref();
        let utf8_path = Utf8PathBuf::try_from(path.to_path_buf())
            .map_err(|_| SettingsError::PathResolution("Path is not valid UTF-8".to_string()))?;
        Ok(Self { inner: utf8_path })
    }

    /// Returns the underlying UTF-8 path.
    pub fn as_utf8_path(&self) -> &Utf8Path {
        &self.inner
    }

    /// Returns the path as a string slice.
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Returns the file extension, if any.
    pub fn extension(&self) -> Option<&str> {
        self.inner.extension()
    }

    /// Returns the parent directory, if any.
    pub fn parent(&self) -> Option<SettingsPath> {
        self.inner.parent().map(|p| Self {
            inner: p.to_path_buf(),
        })
    }

    /// Joins another path component to this path.
    pub fn join<P: AsRef<str>>(&self, path: P) -> Self {
        Self {
            inner: self.inner.join(path.as_ref()),
        }
    }

    /// Resolves placeholders in the path using the provided values.
    ///
    /// Placeholders are denoted by curly braces: `{placeholder_name}`
    ///
    /// # Arguments
    ///
    /// * `values` - A map of placeholder names to their values
    ///
    /// # Errors
    ///
    /// Returns an error if a placeholder cannot be resolved.
    ///
    /// # Example
    ///
    /// ```
    /// use chicken_settings::path::SettingsPath;
    /// use std::collections::HashMap;
    ///
    /// let template = SettingsPath::new("saves/{slot_id}/game.json").unwrap();
    /// let mut values = HashMap::new();
    /// values.insert("slot_id".to_string(), "3".to_string());
    ///
    /// let resolved = template.resolve_placeholders(&values).unwrap();
    /// assert_eq!(resolved.as_str(), "saves/3/game.json");
    /// ```
    pub fn resolve_placeholders(&self, values: &HashMap<String, String>) -> SettingsResult<Self> {
        let mut result = self.as_str().to_string();

        // Find all placeholders in the format {name}
        let mut start = 0;
        while let Some(placeholder_start) = result[start..].find('{') {
            let placeholder_start = start + placeholder_start;

            if let Some(placeholder_end) = result[placeholder_start..].find('}') {
                let placeholder_end = placeholder_start + placeholder_end;
                let placeholder_name = &result[placeholder_start + 1..placeholder_end];

                // Look up the placeholder value
                if let Some(value) = values.get(placeholder_name) {
                    result.replace_range(placeholder_start..=placeholder_end, value);
                    // Adjust start position after replacement
                    start = placeholder_start + value.len();
                } else {
                    return Err(SettingsError::PathResolution(format!(
                        "Missing placeholder value for '{}'",
                        placeholder_name
                    )));
                }
            } else {
                return Err(SettingsError::PathResolution(
                    "Unclosed placeholder brace".to_string(),
                ));
            }
        }

        Self::new(result)
    }

    /// Checks if this path contains unresolved placeholders.
    pub fn has_placeholders(&self) -> bool {
        self.as_str().contains('{') && self.as_str().contains('}')
    }

    /// Extracts placeholder names from the path.
    pub fn extract_placeholders(&self) -> Vec<String> {
        let mut placeholders = Vec::new();
        let mut start = 0;

        while let Some(placeholder_start) = self.as_str()[start..].find('{') {
            let placeholder_start = start + placeholder_start;

            if let Some(placeholder_end) = self.as_str()[placeholder_start..].find('}') {
                let placeholder_end = placeholder_start + placeholder_end;
                let placeholder_name = &self.as_str()[placeholder_start + 1..placeholder_end];
                placeholders.push(placeholder_name.to_string());
                start = placeholder_end + 1;
            } else {
                break;
            }
        }

        placeholders
    }
}

impl AsRef<Utf8Path> for SettingsPath {
    fn as_ref(&self) -> &Utf8Path {
        &self.inner
    }
}

impl AsRef<std::path::Path> for SettingsPath {
    fn as_ref(&self) -> &std::path::Path {
        self.inner.as_std_path()
    }
}

impl From<SettingsPath> for Utf8PathBuf {
    fn from(path: SettingsPath) -> Self {
        path.inner
    }
}

impl From<SettingsPath> for std::path::PathBuf {
    fn from(path: SettingsPath) -> Self {
        path.inner.into_std_path_buf()
    }
}

/// Trait for types that provide context for path resolution.
///
/// Implement this trait for your context types to enable dynamic path resolution
/// for settings with placeholders in their paths.
///
/// For static settings (no placeholders in path), the unit type `()` implements
/// this trait and can be used as the context.
///
/// # Example
///
/// ```
/// use chicken_settings::path::{PathContext, SettingsPath};
/// use std::collections::HashMap;
///
/// #[derive(Clone)]
/// struct SaveContext {
///     slot_id: u32,
/// }
///
/// impl PathContext for SaveContext {
///     fn resolve_placeholder(&self, name: &str) -> Option<String> {
///         match name {
///             "slot_id" => Some(self.slot_id.to_string()),
///             _ => None,
///         }
///     }
///     
///     fn to_map(&self) -> HashMap<String, String> {
///         let mut map = HashMap::new();
///         map.insert("slot_id".to_string(), self.slot_id.to_string());
///         map
///     }
/// }
/// ```
pub trait PathContext: Send + Sync + Clone + 'static {
    /// Resolves a single placeholder name to its value.
    fn resolve_placeholder(&self, name: &str) -> Option<String>;

    /// Converts this context to a HashMap of all placeholder values.
    fn to_map(&self) -> HashMap<String, String>;

    /// Resolves all placeholders in a path template.
    fn resolve_path(&self, template: &str) -> SettingsResult<SettingsPath> {
        let path = SettingsPath::new(template)?;
        let placeholders = path.extract_placeholders();

        if placeholders.is_empty() {
            return Ok(path);
        }

        let mut values = HashMap::new();
        for name in placeholders {
            if let Some(value) = self.resolve_placeholder(&name) {
                values.insert(name, value);
            } else {
                return Err(SettingsError::PathResolution(format!(
                    "Cannot resolve placeholder '{}'",
                    name
                )));
            }
        }

        path.resolve_placeholders(&values)
    }
}

/// Implementation of PathContext for the unit type.
///
/// This allows static settings (those without placeholders) to use `()` as their context type.
impl PathContext for () {
    fn resolve_placeholder(&self, _name: &str) -> Option<String> {
        // Unit type has no placeholders to resolve
        None
    }

    fn to_map(&self) -> HashMap<String, String> {
        // Unit type has no values
        HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_path_creation() {
        let path = SettingsPath::new("settings/audio.json").unwrap();
        assert_eq!(path.as_str(), "settings/audio.json");
        assert_eq!(path.extension(), Some("json"));
    }

    #[test]
    fn test_resolve_placeholders() {
        let path = SettingsPath::new("saves/{slot_id}/game.json").unwrap();

        let mut values = HashMap::new();
        values.insert("slot_id".to_string(), "42".to_string());

        let resolved = path.resolve_placeholders(&values).unwrap();
        assert_eq!(resolved.as_str(), "saves/42/game.json");
    }

    #[test]
    fn test_extract_placeholders() {
        let path = SettingsPath::new("saves/{slot_id}/{player_name}/game.json").unwrap();
        let placeholders = path.extract_placeholders();

        assert_eq!(placeholders, vec!["slot_id", "player_name"]);
    }

    #[test]
    fn test_has_placeholders() {
        let with = SettingsPath::new("saves/{id}/game.json").unwrap();
        let without = SettingsPath::new("settings/global.json").unwrap();

        assert!(with.has_placeholders());
        assert!(!without.has_placeholders());
    }
}
