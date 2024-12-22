use bevy::reflect::Tuple;

use super::{tuple_utils::TupleLikeInfo, LoadStructure};

pub struct LoadTuple<'a> {
    pub tuple_info: &'a dyn TupleLikeInfo,
    pub table: &'a toml::value::Array,
    pub tuple: &'a mut dyn Tuple,
}

impl<'a> LoadTuple<'a> {
    pub fn load_tuple(self) {
        for i in 0..self.tuple_info.field_len() {
            let Some(toml_value) = self.table.get(i) else {
                continue;
            };

            let field_mut = self.tuple.field_mut(i).unwrap();
            let field_attrs = self.tuple_info.field_at(i).unwrap().custom_attributes();

            LoadStructure {
                type_info: field_mut.get_represented_type_info().unwrap(),
                table: toml_value,
                structure: field_mut,
                custom_attributes: Some(field_attrs),
            }
            .load();
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
    fn load_tuple() {
        let mut tuple = (0, 0);

        let toml_value = tuple_test_toml();
        LoadTuple {
            tuple_info: tuple.reflect_type_info().as_tuple().unwrap(),
            table: toml_value.as_array().unwrap(),
            tuple: &mut tuple,
        }
        .load_tuple();
        assert_eq!(tuple, (1, 2));
    }

    fn tuple_struct_struct_toml() -> toml::Value {
        toml::Value::Array(vec![tuple_test_toml(), tuple_test_toml()])
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_tuple_struct_struct() {
        let mut tuple = ((0, 0), (0, 0));

        let toml_value = tuple_struct_struct_toml();
        LoadTuple {
            tuple_info: tuple.reflect_type_info().as_tuple().unwrap(),
            table: toml_value.as_array().unwrap(),
            tuple: &mut tuple,
        }
        .load_tuple();

        assert_eq!(tuple, ((1, 2), (1, 2)));
    }
}
