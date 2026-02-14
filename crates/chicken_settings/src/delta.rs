use crate::error::{SettingsError, SettingsResult};
use crate::format::Format;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Storage that saves only the differences (deltas) from default values.
///
/// This reduces file size and makes defaults easily changeable in code.
///
/// # Type Parameters
///
/// * `T` - The settings type being stored
#[derive(Debug)]
pub struct DeltaStorage<T: Serialize + DeserializeOwned + Default + PartialEq + Clone> {
    default: T,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Default + PartialEq + Clone> DeltaStorage<T> {
    /// Creates a new delta storage with the default value.
    pub fn new() -> Self {
        Self {
            default: T::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a new delta storage with a specific default value.
    pub fn with_default(default: T) -> Self {
        Self {
            default,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Computes the delta (differences) between current and default values.
    ///
    /// Returns a map of field paths to their values that differ from default.
    ///
    /// # Arguments
    ///
    /// * `current` - The current settings values
    pub fn compute_delta(&self, current: &T) -> SettingsResult<HashMap<String, Value>> {
        // If they're equal, return empty delta
        if self.is_equal_to_default(current) {
            return Ok(HashMap::new());
        }

        // Serialize both to JSON for comparison
        let default_json = serde_json::to_value(&self.default)
            .map_err(|e| SettingsError::Serialization(e.to_string()))?;
        let current_json = serde_json::to_value(current)
            .map_err(|e| SettingsError::Serialization(e.to_string()))?;

        // Compute differences
        let mut deltas = HashMap::new();
        self.compute_json_delta("", &default_json, &current_json, &mut deltas)?;

        Ok(deltas)
    }

    /// Applies a delta to the default value, returning the merged result.
    ///
    /// # Arguments
    ///
    /// * `delta` - Map of field paths to their values
    pub fn apply_delta(&self, delta: &HashMap<String, Value>) -> SettingsResult<T> {
        if delta.is_empty() {
            return Ok(self.default.clone());
        }

        // Start with default as JSON
        let mut result_json = serde_json::to_value(&self.default)
            .map_err(|e| SettingsError::Serialization(e.to_string()))?;

        // Apply each delta
        for (path, value) in delta {
            self.set_json_value(&mut result_json, path, value.clone())?;
        }

        // Deserialize back to T
        serde_json::from_value(result_json)
            .map_err(|e| SettingsError::Deserialization(e.to_string()))
    }

    /// Serializes settings, storing only deltas from default.
    ///
    /// # Arguments
    ///
    /// * `current` - The current settings values
    /// * `format` - The serialization format to use
    pub fn serialize(&self, current: &T, format: Format) -> SettingsResult<Vec<u8>> {
        // Compute and serialize delta
        let delta = self.compute_delta(current)?;
        format.serialize(&DeltaWrapper { delta })
    }

    /// Deserializes settings from delta format and merges with defaults.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized data
    /// * `format` - The serialization format used
    pub fn deserialize(&self, data: &[u8], format: Format) -> SettingsResult<T> {
        // Deserialize delta and apply to default
        let wrapper: DeltaWrapper = format.deserialize(data)?;
        self.apply_delta(&wrapper.delta)
    }

    /// Checks if current value equals the default.
    pub fn is_equal_to_default(&self, current: &T) -> bool {
        // Use PartialEq if available, otherwise fall back to serialization comparison
        &self.default == current
    }

    /// Gets the changed fields between current and default values.
    ///
    /// Returns a vector of (field_path, json_value) tuples for fields that differ from default.
    /// This is useful for surgical editing where only changed fields need to be updated.
    ///
    /// # Arguments
    ///
    /// * `current` - The current settings values
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the field path (e.g., "settings.volume") and its JSON value.
    pub fn get_changed_fields(&self, current: &T) -> Vec<(String, Value)> {
        match self.compute_delta(current) {
            Ok(delta) => delta.into_iter().collect(),
            Err(e) => {
                // On error, return empty vector - caller should fall back to full serialization
                eprintln!("Failed to compute delta: {}", e);
                Vec::new()
            }
        }
    }

    fn compute_json_delta(
        &self,
        path: &str,
        default: &Value,
        current: &Value,
        deltas: &mut HashMap<String, Value>,
    ) -> SettingsResult<()> {
        match (default, current) {
            // Both are objects: recurse into fields
            (Value::Object(def_obj), Value::Object(cur_obj)) => {
                // Check for added/changed fields
                for (key, cur_val) in cur_obj {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };

                    if let Some(def_val) = def_obj.get(key) {
                        // Field exists in both: check if different
                        if def_val != cur_val {
                            self.compute_json_delta(&new_path, def_val, cur_val, deltas)?;
                        }
                    } else {
                        // Field only in current: it's an addition
                        deltas.insert(new_path, cur_val.clone());
                    }
                }
            }
            // Both are arrays: element-wise comparison
            (Value::Array(def_arr), Value::Array(cur_arr)) => {
                for (i, (def_item, cur_item)) in def_arr.iter().zip(cur_arr.iter()).enumerate() {
                    let new_path = format!("{}[{}]", path, i);
                    if def_item != cur_item {
                        self.compute_json_delta(&new_path, def_item, cur_item, deltas)?;
                    }
                }

                // Handle additional elements in current
                if cur_arr.len() > def_arr.len() {
                    for i in def_arr.len()..cur_arr.len() {
                        let new_path = format!("{}[{}]", path, i);
                        deltas.insert(new_path, cur_arr[i].clone());
                    }
                }
            }
            // Values are different
            _ => {
                if default != current {
                    let final_path = if path.is_empty() {
                        "value".to_string()
                    } else {
                        path.to_string()
                    };
                    deltas.insert(final_path, current.clone());
                }
            }
        }

        Ok(())
    }

    fn set_json_value(&self, root: &mut Value, path: &str, value: Value) -> SettingsResult<()> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.len() == 1 && !parts[0].contains('[') {
            // Simple field
            if let Value::Object(map) = root {
                map.insert(parts[0].to_string(), value);
                return Ok(());
            }
        }

        // Complex path with nesting and/or array indices
        let mut current = root;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part: set the value
                if part.contains('[') {
                    // Handle array index
                    let (field, idx) = self.parse_array_access(part)?;
                    if let Value::Object(map) = current {
                        if let Some(Value::Array(arr)) = map.get_mut(&field) {
                            if let Some(slot) = arr.get_mut(idx) {
                                *slot = value;
                            } else {
                                return Err(SettingsError::DeltaError(format!(
                                    "Array index {} out of bounds",
                                    idx
                                )));
                            }
                        } else {
                            return Err(SettingsError::DeltaError(format!(
                                "Field {} is not an array",
                                field
                            )));
                        }
                    }
                } else if let Value::Object(map) = current {
                    map.insert(part.to_string(), value);
                }
                break;
            } else {
                // Navigate deeper
                if part.contains('[') {
                    let (field, idx) = self.parse_array_access(part)?;
                    if let Value::Object(map) = current {
                        if let Some(Value::Array(arr)) = map.get_mut(&field) {
                            current = arr.get_mut(idx).ok_or_else(|| {
                                SettingsError::DeltaError(format!(
                                    "Array index {} out of bounds",
                                    idx
                                ))
                            })?;
                        } else {
                            return Err(SettingsError::DeltaError(format!(
                                "Field {} is not an array",
                                field
                            )));
                        }
                    }
                } else if let Value::Object(map) = current {
                    current = map.get_mut(*part).ok_or_else(|| {
                        SettingsError::DeltaError(format!("Field {} not found", part))
                    })?;
                }
            }
        }

        Ok(())
    }

    fn parse_array_access(&self, s: &str) -> SettingsResult<(String, usize)> {
        let start = s
            .find('[')
            .ok_or_else(|| SettingsError::DeltaError("Expected array index".to_string()))?;
        let end = s
            .find(']')
            .ok_or_else(|| SettingsError::DeltaError("Unclosed array bracket".to_string()))?;

        let field = s[..start].to_string();
        let idx: usize = s[start + 1..end]
            .parse()
            .map_err(|_| SettingsError::DeltaError(format!("Invalid array index in {}", s)))?;

        Ok((field, idx))
    }
}

impl<T: Serialize + DeserializeOwned + Default + PartialEq + Clone> Default for DeltaStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper struct for serializing delta maps.
#[derive(Debug, Serialize, Deserialize)]
struct DeltaWrapper {
    #[serde(flatten)]
    delta: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
    struct TestSettings {
        volume: f32,
        muted: bool,
        quality: String,
    }

    #[test]
    fn test_compute_delta() {
        let storage = DeltaStorage::<TestSettings>::new();

        let current = TestSettings {
            volume: 0.5,
            muted: true,
            quality: "high".to_string(),
        };

        let delta = storage.compute_delta(&current).unwrap();

        // All fields should be in delta since they differ from defaults
        assert!(delta.contains_key("volume"));
        assert!(delta.contains_key("muted"));
        assert!(delta.contains_key("quality"));
    }

    #[test]
    fn test_empty_delta_when_equal_to_default() {
        let storage = DeltaStorage::<TestSettings>::new();
        let default = TestSettings::default();

        let delta = storage.compute_delta(&default).unwrap();
        assert!(delta.is_empty());
    }

    #[test]
    fn test_apply_delta() {
        let storage = DeltaStorage::<TestSettings>::new();

        let mut delta = HashMap::new();
        delta.insert("volume".to_string(), json!(0.8));
        delta.insert("muted".to_string(), json!(true));

        let result = storage.apply_delta(&delta).unwrap();

        assert_eq!(result.volume, 0.8);
        assert_eq!(result.muted, true);
        // quality should be default
        assert_eq!(result.quality, TestSettings::default().quality);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let storage = DeltaStorage::<TestSettings>::new();

        let original = TestSettings {
            volume: 0.75,
            muted: false,
            quality: "medium".to_string(),
        };

        let data = storage.serialize(&original, Format::Toml).unwrap();
        let deserialized = storage.deserialize(&data, Format::Toml).unwrap();

        assert_eq!(original, deserialized);
    }
}
