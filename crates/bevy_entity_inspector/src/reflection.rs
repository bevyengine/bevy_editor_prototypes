//! Reflection utilities for the entity inspector.
//!
//! This module provides helper functions for working with Bevy's reflection system
//! to extract and format component data for display in the inspector.

use bevy::reflect::*;

/// Extracts crate name and type name from a component type path.
///
/// Component names from the reflection system often include full module paths like:
/// - "bevy_transform::components::transform::Transform"
/// - "my_game::components::player::Player"
/// - "Transform" (for local/simple names)
///
/// This function extracts the crate name and the final type name for display purposes.
///
/// # Arguments
///
/// * `component_name` - Full component type path from reflection
///
/// # Returns
///
/// A tuple of (crate_name, type_name) where:
/// - For "bevy_transform::components::transform::Transform" -> ("bevy_transform", "Transform")
/// - For "my_game::player::Player" -> ("my_game", "Player")
/// - For "Transform" -> ("Local", "Transform")
///
/// # Examples
///
/// ```rust,no_run
/// # use bevy_entity_inspector::reflection::extract_crate_and_type;
/// let (crate_name, type_name) = extract_crate_and_type("bevy_transform::components::transform::Transform");
/// assert_eq!(crate_name, "bevy_transform");
/// assert_eq!(type_name, "Transform");
///
/// let (crate_name, type_name) = extract_crate_and_type("Transform");
/// assert_eq!(crate_name, "Local");
/// assert_eq!(type_name, "Transform");
/// ```
pub fn extract_crate_and_type(component_name: &str) -> (String, String) {
    if let Some(first_separator) = component_name.find("::") {
        // Extract the crate name (everything before the first "::")
        let crate_name = component_name[..first_separator].to_string();

        // Extract the type name (everything after the last "::")
        let type_name = component_name
            .split("::")
            .last()
            .unwrap_or(component_name)
            .to_string();

        (crate_name, type_name)
    } else {
        // No "::" found, treat as a local/simple type
        ("Local".to_string(), component_name.to_string())
    }
}

/// Extracts field information from a reflected value for display in the inspector.
///
/// This function traverses the reflected structure and extracts all displayable
/// field information, handling different reflection types appropriately.
///
/// # Arguments
///
/// * `reflect` - The reflected value to extract fields from
///
/// # Returns
///
/// A vector of (field_name, field_value) tuples representing the structure's contents.
///
/// # Supported Types
///
/// - **Struct**: Named fields with their values
/// - **TupleStruct**: Indexed fields (field_0, field_1, etc.)
/// - **Tuple**: Indexed items (item_0, item_1, etc.)
/// - **List/Array**: Indexed elements with bracket notation ([0], [1], etc.)
/// - **Map**: Key-value pairs
/// - **Enum**: Variant name and field data
///
/// # Examples
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_entity_inspector::reflection::extract_reflect_fields;
/// # use bevy::reflect::*;
///
/// #[derive(Reflect)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// let pos = Position { x: 1.0, y: 2.0 };
/// let fields = extract_reflect_fields(pos.as_partial_reflect());
/// // Returns: [("x", "1.0"), ("y", "2.0")]
/// ```
pub fn extract_reflect_fields(reflect: &dyn PartialReflect) -> Vec<(String, String)> {
    let mut fields = Vec::new();

    match reflect.reflect_ref() {
        ReflectRef::Struct(s) => {
            for i in 0..s.field_len() {
                if let Some(field) = s.field_at(i) {
                    let default_field_name = format!("field_{}", i);
                    let name = s.name_at(i).unwrap_or(&default_field_name);
                    let value = format!("{:?}", field);
                    fields.push((name.to_string(), value));
                }
            }
        }
        ReflectRef::TupleStruct(ts) => {
            for i in 0..ts.field_len() {
                if let Some(field) = ts.field(i) {
                    let name = format!("field_{}", i);
                    let value = format!("{:?}", field);
                    fields.push((name, value));
                }
            }
        }
        ReflectRef::Tuple(t) => {
            for i in 0..t.field_len() {
                if let Some(field) = t.field(i) {
                    let name = format!("item_{}", i);
                    let value = format!("{:?}", field);
                    fields.push((name, value));
                }
            }
        }
        ReflectRef::List(l) => {
            for i in 0..l.len() {
                if let Some(item) = l.get(i) {
                    let name = format!("[{}]", i);
                    let value = format!("{:?}", item);
                    fields.push((name, value));
                }
            }
        }
        ReflectRef::Array(a) => {
            for i in 0..a.len() {
                if let Some(item) = a.get(i) {
                    let name = format!("[{}]", i);
                    let value = format!("{:?}", item);
                    fields.push((name, value));
                }
            }
        }
        ReflectRef::Map(m) => {
            for (key, value) in m.iter() {
                let name = format!("{:?}", key);
                let value = format!("{:?}", value);
                fields.push((name, value));
            }
        }
        ReflectRef::Enum(e) => {
            fields.push(("variant".to_string(), e.variant_name().to_string()));
            for i in 0..e.field_len() {
                if let Some(field) = e.field_at(i) {
                    let default_field_name = format!("field_{}", i);
                    let name = e.name_at(i).unwrap_or(&default_field_name);
                    fields.push((name.to_string(), format!("{:?}", field)));
                }
            }
        }
        _ => {
            // For primitive values and any other cases, just show the value itself
            fields.push(("value".to_string(), format!("{:?}", reflect)));
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_crate_and_type() {
        // Test full path
        let (crate_name, type_name) =
            extract_crate_and_type("bevy_transform::components::transform::Transform");
        assert_eq!(crate_name, "bevy_transform");
        assert_eq!(type_name, "Transform");

        // Test simple path
        let (crate_name, type_name) = extract_crate_and_type("my_game::Player");
        assert_eq!(crate_name, "my_game");
        assert_eq!(type_name, "Player");

        // Test no path
        let (crate_name, type_name) = extract_crate_and_type("Transform");
        assert_eq!(crate_name, "Local");
        assert_eq!(type_name, "Transform");
    }
}
