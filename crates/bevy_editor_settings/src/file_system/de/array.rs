use bevy::{
    prelude::warn,
    reflect::{Array, ArrayInfo},
};

use super::{LoadStructure, StructureLoader};

pub struct LoadArray<'a> {
    pub array_info: &'a ArrayInfo,
    pub array: &'a mut dyn Array,
}

impl<'a> StructureLoader for LoadArray<'a> {
    type Input = &'a toml::value::Array;

    fn load(self, input: Self::Input) {
        if input.len() != self.array_info.capacity() {
            warn!(
                "Preferences: Expected Array length {}, got {}",
                self.array_info.capacity(),
                input.len()
            );
            return;
        }

        for i in 0..self.array_info.capacity() {
            let Some(toml_value) = input.get(i) else {
                continue;
            };

            let field_mut = self.array.get_mut(i).unwrap();

            LoadStructure {
                type_info: field_mut.get_represented_type_info().unwrap(),
                structure: field_mut,
                custom_attributes: None,
            }
            .load(toml_value);
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
            array: &mut array,
        }
        .load(toml_value.as_array().unwrap());
        assert_eq!(array, [1, 2]);
    }
}
