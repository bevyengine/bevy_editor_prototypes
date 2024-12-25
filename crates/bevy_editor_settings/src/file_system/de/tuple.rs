use bevy::reflect::Tuple;

use super::{tuple_utils::TupleLikeInfo, LoadStructure, StructureLoader};

pub struct LoadTuple<'a> {
    pub tuple_info: &'a dyn TupleLikeInfo,
    pub tuple: &'a mut dyn Tuple,
}

impl<'a> StructureLoader for LoadTuple<'a> {
    type Input = &'a toml::value::Array;

    fn load(self, input: Self::Input) {
        for i in 0..self.tuple_info.field_len() {
            let Some(toml_value) = input.get(i) else {
                continue;
            };

            let field_mut = self.tuple.field_mut(i).unwrap();
            let field_attrs = self.tuple_info.field_at(i).unwrap().custom_attributes();

            LoadStructure {
                type_info: field_mut.get_represented_type_info().unwrap(),
                structure: field_mut,
                custom_attributes: Some(field_attrs),
            }
            .load(toml_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::DynamicTyped as _;

    use super::*;

    fn tuple_test_toml() -> toml::Value {
        toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)])
    }

    #[tracing_test::traced_test]
    #[test]
    fn load() {
        let mut tuple = (0, 0);

        let toml_value = tuple_test_toml();
        LoadTuple {
            tuple_info: tuple.reflect_type_info().as_tuple().unwrap(),
            tuple: &mut tuple,
        }
        .load(toml_value.as_array().unwrap());
        assert_eq!(tuple, (1, 2));
    }

    fn tuple_struct_struct_toml() -> toml::Value {
        toml::Value::Array(vec![tuple_test_toml(), tuple_test_toml()])
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_struct_struct() {
        let mut tuple = ((0, 0), (0, 0));

        let toml_value = tuple_struct_struct_toml();
        LoadTuple {
            tuple_info: tuple.reflect_type_info().as_tuple().unwrap(),
            tuple: &mut tuple,
        }
        .load(toml_value.as_array().unwrap());

        assert_eq!(tuple, ((1, 2), (1, 2)));
    }
}
