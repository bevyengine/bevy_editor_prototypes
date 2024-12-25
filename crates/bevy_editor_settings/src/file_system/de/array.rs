use bevy::{
    prelude::warn,
    reflect::{Array, ArrayInfo},
};

use super::{LoadStructure, StructureLoader};

pub struct LoadArray<'a> {
    pub array_info: &'a ArrayInfo,
    pub array: &'a mut dyn Array,
    pub toml_array: &'a toml::value::Array,
}

impl StructureLoader for LoadArray<'_> {
    fn load(self) {
        if self.toml_array.len() != self.array_info.capacity() {
            warn!(
                "Preferences: Expected Array length {}, got {}",
                self.array_info.capacity(),
                self.toml_array.len()
            );
            return;
        }

        for i in 0..self.array_info.capacity() {
            let Some(toml_value) = self.toml_array.get(i) else {
                continue;
            };

            let field_mut = self.array.get_mut(i).unwrap();

            LoadStructure {
                type_info: field_mut.get_represented_type_info().unwrap(),
                table: toml_value,
                structure: field_mut,
                custom_attributes: None,
            }
            .load();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::DynamicTyped as _;

    use super::*;

    #[tracing_test::traced_test]
    #[test]
    fn load_array() {
        let mut array = [0, 0];

        let toml_value = toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)]);
        LoadArray {
            array_info: array.reflect_type_info().as_array().unwrap(),
            toml_array: toml_value.as_array().unwrap(),
            array: &mut array,
        }
        .load();
        assert_eq!(array, [1, 2]);
    }
}
