use serde::Serialize;
use std::ops::Range;
use thiserror::Error;
use toml_edit::{DocumentMut, Item, Table, Value};

#[derive(Error, Debug)]
pub enum EditError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("TOML error: {0}")]
    Toml(String),
}

pub type Result<T> = std::result::Result<T, EditError>;

/// Trait for surgical TOML editing
pub trait SurgicalEditor {
    /// Update a value at the given key path. Creates intermediate tables if needed.
    /// Preserves comments and formatting of unchanged parts.
    fn update_value<T: Serialize>(
        &self,
        content: &mut String,
        key_path: &[&str],
        new_value: &T,
    ) -> Result<()>;

    /// Insert a new value. Fails if key already exists.
    fn insert_value<T: Serialize>(
        &self,
        content: &mut String,
        key_path: &[&str],
        new_value: &T,
    ) -> Result<()>;

    /// Remove a value at the given key path.
    fn remove_value(&self, content: &mut String, key_path: &[&str]) -> Result<()>;

    /// Get the byte range of a value at key path (useful for previews)
    fn get_value_range(&self, content: &str, key_path: &[&str]) -> Result<Range<usize>>;
}

/// TOML-specific surgical editor using toml_edit
pub struct TomlEditor;

impl TomlEditor {
    pub fn new() -> Self {
        Self
    }

    /// Convert a serializable value to a toml_edit::Value
    fn serialize_to_value<T: Serialize>(&self, value: &T) -> Result<Value> {
        let json = serde_json::to_string(value)?;
        let json_value: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| EditError::Serialization(e))?;

        Self::json_value_to_toml_value(json_value)
    }

    /// Convert serde_json::Value to toml_edit::Value
    fn json_value_to_toml_value(value: serde_json::Value) -> Result<Value> {
        match value {
            serde_json::Value::Null => {
                Err(EditError::Toml("Null values not supported".to_string()))
            }
            serde_json::Value::Bool(b) => Ok(Value::from(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::from(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::from(f))
                } else {
                    Err(EditError::Toml("Invalid number".to_string()))
                }
            }
            serde_json::Value::String(s) => Ok(Value::from(s)),
            serde_json::Value::Array(arr) => {
                let mut toml_arr = toml_edit::Array::new();
                for item in arr {
                    let toml_val = Self::json_value_to_toml_value(item)?;
                    toml_arr.push(toml_val);
                }
                Ok(Value::Array(toml_arr))
            }
            serde_json::Value::Object(map) => {
                let mut inline_table = toml_edit::InlineTable::new();
                for (k, v) in map {
                    let toml_val = Self::json_value_to_toml_value(v)?;
                    inline_table.insert(&k, toml_val);
                }
                Ok(Value::InlineTable(inline_table))
            }
        }
    }

    /// Check if document contains key at path
    fn path_exists(&self, doc: &DocumentMut, key_path: &[&str]) -> bool {
        if key_path.is_empty() {
            return false;
        }

        let mut current = doc.as_item();

        for (i, key) in key_path.iter().enumerate() {
            if i == key_path.len() - 1 {
                // Check if final key exists
                return match current {
                    Item::Table(table) => table.contains_key(key),
                    _ => false,
                };
            } else {
                // Navigate deeper
                match current {
                    Item::Table(table) => match table.get(key) {
                        Some(item) => current = item,
                        None => return false,
                    },
                    _ => return false,
                }
            }
        }

        false
    }

    /// Preserve decoration from source item to target item
    fn preserve_decor(source: &Item, target: &mut Item) {
        // For values, we need to preserve the decoration within the Formatted wrapper
        match (source, target) {
            (Item::Value(src_val), Item::Value(tgt_val)) => {
                // Match on value types to access decorators
                match (src_val, tgt_val) {
                    (Value::Integer(src), Value::Integer(tgt)) => {
                        tgt.decor_mut().clone_from(src.decor());
                    }
                    (Value::Float(src), Value::Float(tgt)) => {
                        tgt.decor_mut().clone_from(src.decor());
                    }
                    (Value::String(src), Value::String(tgt)) => {
                        tgt.decor_mut().clone_from(src.decor());
                    }
                    (Value::Boolean(src), Value::Boolean(tgt)) => {
                        tgt.decor_mut().clone_from(src.decor());
                    }
                    (Value::Datetime(src), Value::Datetime(tgt)) => {
                        tgt.decor_mut().clone_from(src.decor());
                    }
                    (Value::Array(src), Value::Array(tgt)) => {
                        tgt.decor_mut().clone_from(src.decor());
                    }
                    (Value::InlineTable(src), Value::InlineTable(tgt)) => {
                        tgt.decor_mut().clone_from(src.decor());
                    }
                    _ => {} // Different types, can't preserve
                }
            }
            (Item::Table(src), Item::Table(tgt)) => {
                tgt.decor_mut().clone_from(src.decor());
            }
            _ => {}
        }
    }
}

impl Default for TomlEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl SurgicalEditor for TomlEditor {
    fn update_value<T: Serialize>(
        &self,
        content: &mut String,
        key_path: &[&str],
        new_value: &T,
    ) -> Result<()> {
        let mut doc: DocumentMut = content
            .parse()
            .map_err(|e| EditError::Parse(format!("{:?}", e)))?;

        if key_path.is_empty() {
            return Err(EditError::KeyNotFound("Empty key path".to_string()));
        }

        // Navigate and update
        let mut current = doc.as_item_mut();

        for (i, key) in key_path.iter().enumerate() {
            let is_last = i == key_path.len() - 1;

            if is_last {
                // Final key - set the value
                let new_val = self.serialize_to_value(new_value)?;

                if let Ok(index) = key.parse::<usize>() {
                    // Array index
                    match current {
                        Item::Value(Value::Array(arr)) => {
                            if index >= arr.len() {
                                return Err(EditError::KeyNotFound(format!(
                                    "Array index {} out of bounds",
                                    index
                                )));
                            }
                            arr.replace(index, new_val);
                        }
                        _ => {
                            return Err(EditError::KeyNotFound(format!(
                                "'{}' is not an array",
                                key
                            )))
                        }
                    }
                } else {
                    // Table key - preserve comments by getting existing item and replacing just the value
                    match current {
                        Item::Table(table) => {
                            // Check if key exists to preserve formatting
                            if let Some(existing) = table.get(key).cloned() {
                                // Create new item from value
                                let mut new_item = Item::Value(new_val);

                                // Preserve decoration from existing item
                                Self::preserve_decor(&existing, &mut new_item);

                                // Insert the new item with preserved decoration
                                table.insert(key, new_item);
                            } else {
                                // New key
                                table.insert(key, Item::Value(new_val));
                            }
                        }
                        _ => {
                            // Replace entire item
                            *current = Item::Value(new_val);
                        }
                    }
                }
            } else {
                // Intermediate key - navigate or create
                match current {
                    Item::Table(table) => {
                        if !table.contains_key(key) {
                            table.insert(key, Item::Table(Table::new()));
                        }
                        current = table
                            .get_mut(key)
                            .ok_or_else(|| EditError::KeyNotFound(key.to_string()))?;
                    }
                    _ => {
                        return Err(EditError::KeyNotFound(format!(
                            "Cannot navigate into non-table at '{}'",
                            key
                        )));
                    }
                }
            }
        }

        *content = doc.to_string();
        Ok(())
    }

    fn insert_value<T: Serialize>(
        &self,
        content: &mut String,
        key_path: &[&str],
        new_value: &T,
    ) -> Result<()> {
        let doc: DocumentMut = content
            .parse()
            .map_err(|e| EditError::Parse(format!("{:?}", e)))?;

        // Check if key already exists
        if self.path_exists(&doc, key_path) {
            let key = key_path.join(".");
            return Err(EditError::Toml(format!("Key '{}' already exists", key)));
        }

        drop(doc);
        // Now update (which will create)
        self.update_value(content, key_path, new_value)
    }

    fn remove_value(&self, content: &mut String, key_path: &[&str]) -> Result<()> {
        let mut doc: DocumentMut = content
            .parse()
            .map_err(|e| EditError::Parse(format!("{:?}", e)))?;

        if key_path.is_empty() {
            return Err(EditError::KeyNotFound("Empty key path".to_string()));
        }

        let last_key = key_path[key_path.len() - 1];

        if key_path.len() == 1 {
            // Top-level key
            if !doc.contains_key(last_key) {
                return Err(EditError::KeyNotFound(last_key.to_string()));
            }
            doc.remove(last_key);
        } else {
            // Navigate to parent
            let mut current = doc.as_item_mut();

            for (i, key) in key_path.iter().enumerate() {
                if i == key_path.len() - 1 {
                    break;
                }

                match current {
                    Item::Table(table) => {
                        current = table
                            .get_mut(key)
                            .ok_or_else(|| EditError::KeyNotFound(key.to_string()))?;
                    }
                    _ => {
                        return Err(EditError::KeyNotFound(format!(
                            "Cannot navigate into non-table at '{}'",
                            key
                        )));
                    }
                }
            }

            // Remove the key from parent
            match current {
                Item::Table(table) => {
                    if !table.contains_key(last_key) {
                        return Err(EditError::KeyNotFound(last_key.to_string()));
                    }
                    table.remove(last_key);
                }
                Item::Value(Value::Array(arr)) => {
                    if let Ok(index) = last_key.parse::<usize>() {
                        if index >= arr.len() {
                            return Err(EditError::KeyNotFound(format!(
                                "Array index {} out of bounds",
                                index
                            )));
                        }
                        arr.remove(index);
                    } else {
                        return Err(EditError::KeyNotFound(format!(
                            "'{}' is not a valid array index",
                            last_key
                        )));
                    }
                }
                _ => {
                    return Err(EditError::KeyNotFound(format!(
                        "Cannot remove from non-table/non-array parent"
                    )));
                }
            }
        }

        *content = doc.to_string();
        Ok(())
    }

    fn get_value_range(&self, content: &str, key_path: &[&str]) -> Result<Range<usize>> {
        let doc: DocumentMut = content
            .parse()
            .map_err(|e| EditError::Parse(format!("{:?}", e)))?;

        if key_path.is_empty() {
            return Err(EditError::KeyNotFound("Empty key path".to_string()));
        }

        let last_key = key_path[key_path.len() - 1];

        // Navigate to the item
        let mut current = doc.as_item();

        for (i, key) in key_path.iter().enumerate() {
            if i == key_path.len() - 1 {
                break;
            }

            match current {
                Item::Table(table) => {
                    current = table
                        .get(key)
                        .ok_or_else(|| EditError::KeyNotFound(key.to_string()))?;
                }
                _ => {
                    return Err(EditError::KeyNotFound(format!(
                        "Cannot navigate into non-table at '{}'",
                        key
                    )));
                }
            }
        }

        // Check if key exists
        match current {
            Item::Table(table) => {
                if !table.contains_key(last_key) {
                    return Err(EditError::KeyNotFound(last_key.to_string()));
                }
            }
            Item::Value(Value::Array(arr)) => {
                if let Ok(index) = last_key.parse::<usize>() {
                    if index >= arr.len() {
                        return Err(EditError::KeyNotFound(format!(
                            "Array index {} out of bounds",
                            index
                        )));
                    }
                } else {
                    return Err(EditError::KeyNotFound(format!(
                        "'{}' is not a valid array index",
                        last_key
                    )));
                }
            }
            _ => {
                return Err(EditError::KeyNotFound(format!(
                    "Cannot access key '{}'",
                    last_key
                )));
            }
        }

        // toml_edit doesn't easily expose byte ranges
        // Return full document range as fallback
        Ok(0..content.len())
    }
}

/// Convenience function for single value updates
pub fn update_toml_value<T: Serialize>(
    content: &mut String,
    key_path: &[&str],
    new_value: &T,
) -> Result<()> {
    TomlEditor::new().update_value(content, key_path, new_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_preserves_comments() {
        let mut content = r#"# This is a comment
[settings]
volume = 50  # Audio volume
"#
        .to_string();

        let editor = TomlEditor::new();
        editor
            .update_value(&mut content, &["settings", "volume"], &75)
            .unwrap();

        assert!(content.contains("# This is a comment"));
        assert!(content.contains("volume = 75"));
        assert!(content.contains("# Audio volume"));
    }

    #[test]
    fn test_update_preserves_formatting() {
        let mut content = r#"[settings]
volume = 50
enabled = true
"#
        .to_string();

        let editor = TomlEditor::new();
        editor
            .update_value(&mut content, &["settings", "volume"], &75)
            .unwrap();

        assert!(content.contains("enabled = true"));
        assert!(content.contains("volume = 75"));
    }

    #[test]
    fn test_arrays_replaced_entirely() {
        let mut content = r#"items = [1, 2, 3]"#.to_string();

        let editor = TomlEditor::new();
        let new_array = vec![4, 5, 6];
        editor
            .update_value(&mut content, &["items"], &new_array)
            .unwrap();

        assert!(
            content.contains("items = [4, 5, 6]") || content.contains("items = [4.0, 5.0, 6.0]")
        );
    }

    #[test]
    fn test_nested_table_creation() {
        let mut content = r#"[settings]
volume = 50
"#
        .to_string();

        let editor = TomlEditor::new();
        editor
            .update_value(&mut content, &["settings", "audio", "volume"], &75)
            .unwrap();

        assert!(content.contains("[settings.audio]") || content.contains("[settings]"));
        assert!(content.contains("volume = 75"));
    }

    #[test]
    fn test_remove_value() {
        let mut content = r#"[settings]
volume = 50
enabled = true
"#
        .to_string();

        let editor = TomlEditor::new();
        editor
            .remove_value(&mut content, &["settings", "volume"])
            .unwrap();

        assert!(!content.contains("volume = 50"));
        assert!(content.contains("enabled = true"));
    }

    #[test]
    fn test_insert_new_value() {
        let mut content = r#"[settings]
volume = 50
"#
        .to_string();

        let editor = TomlEditor::new();
        editor
            .insert_value(&mut content, &["settings", "enabled"], &true)
            .unwrap();

        assert!(content.contains("volume = 50"));
        assert!(content.contains("enabled = true"));
    }

    #[test]
    fn test_insert_fails_if_key_exists() {
        let mut content = r#"[settings]
volume = 50
"#
        .to_string();

        let editor = TomlEditor::new();
        let result = editor.insert_value(&mut content, &["settings", "volume"], &75);

        assert!(result.is_err());
    }

    #[test]
    fn test_remove_key_not_found() {
        let mut content = r#"[settings]
volume = 50
"#
        .to_string();

        let editor = TomlEditor::new();
        let result = editor.remove_value(&mut content, &["settings", "nonexistent"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_key_not_found_creates_table() {
        let mut content = r#""#.to_string();

        let editor = TomlEditor::new();
        editor
            .update_value(&mut content, &["settings", "volume"], &50)
            .unwrap();

        assert!(content.contains("[settings]"));
        assert!(content.contains("volume = 50"));
    }

    #[test]
    fn test_complex_nested_structure() {
        let mut content = r#"[app]
name = "MyApp"

[app.settings]
debug = false
"#
        .to_string();

        let editor = TomlEditor::new();
        editor
            .update_value(&mut content, &["app", "settings", "debug"], &true)
            .unwrap();

        assert!(content.contains("name = \"MyApp\""));
        assert!(content.contains("debug = true"));
    }

    #[test]
    fn test_update_top_level_value() {
        let mut content = r#"version = "1.0.0"
name = "test"
"#
        .to_string();

        let editor = TomlEditor::new();
        editor
            .update_value(&mut content, &["version"], &"2.0.0")
            .unwrap();

        assert!(content.contains("version = \"2.0.0\""));
        assert!(content.contains("name = \"test\""));
    }

    #[test]
    fn test_convenience_function() {
        let mut content = r#"value = 10"#.to_string();

        update_toml_value(&mut content, &["value"], &20).unwrap();

        assert!(content.contains("value = 20"));
    }

    #[test]
    fn test_different_value_types() {
        let mut content = r#"[config]
"#
        .to_string();

        let editor = TomlEditor::new();

        // Integer
        editor
            .update_value(&mut content, &["config", "int_val"], &42i32)
            .unwrap();
        // Float
        editor
            .update_value(&mut content, &["config", "float_val"], &3.14f64)
            .unwrap();
        // String
        editor
            .update_value(&mut content, &["config", "str_val"], &"hello")
            .unwrap();
        // Bool
        editor
            .update_value(&mut content, &["config", "bool_val"], &true)
            .unwrap();
        // Array
        editor
            .update_value(&mut content, &["config", "arr_val"], &vec![1, 2, 3])
            .unwrap();

        assert!(content.contains("int_val = 42"));
        assert!(content.contains("float_val = 3.14") || content.contains("float_val = 3.14"));
        assert!(content.contains("str_val = \"hello\""));
        assert!(content.contains("bool_val = true"));
    }
}
