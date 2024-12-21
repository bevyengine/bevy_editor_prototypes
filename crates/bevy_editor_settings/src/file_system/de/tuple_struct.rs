use bevy::{
    prelude::*,
    reflect::{ReflectMut, TupleStruct, TupleStructInfo, TypeInfo},
};

use super::{structs::LoadStruct, tuple_utils::TupleLikeInfo, value::LoadValue, LoadStructure};

pub struct LoadTupleStruct<'a> {
    pub tuple_struct_info: &'a dyn TupleLikeInfo,
    pub table: &'a toml::value::Array,
    pub tuple_struct: &'a mut dyn TupleStruct,
}

impl<'a> LoadTupleStruct<'a> {
    pub fn load_tuple_struct(self) {
        for i in 0..self.tuple_struct_info.field_len() {
            let Some(toml_value) = self.table.get(i) else {
                continue;
            };

            let field_mut = self.tuple_struct.field_mut(i).unwrap();
            let field_attrs = self
                .tuple_struct_info
                .field_at(i)
                .unwrap()
                .custom_attributes();

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

    #[derive(Debug, Clone, PartialEq, Reflect, Default)]
    struct TupleStructTest(u32, u32);

    fn tuple_struct_test_toml() -> toml::Value {
        toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)])
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_tuple_struct() {
        let mut tuple_struct = TupleStructTest::default();

        let toml_value = tuple_struct_test_toml();
        LoadTupleStruct {
            tuple_struct_info: tuple_struct.reflect_type_info().as_tuple_struct().unwrap(),
            table: toml_value.as_array().unwrap(),
            tuple_struct: &mut tuple_struct,
        }
        .load_tuple_struct();
        assert_eq!(tuple_struct, TupleStructTest(1, 2));
    }

    #[derive(Debug, Clone, PartialEq, Reflect, Default)]
    struct TupleStructStruct(TupleStructTest);

    fn tuple_struct_struct_toml() -> toml::Value {
        toml::Value::Array(vec![tuple_struct_test_toml()])
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_tuple_struct_struct() {
        let mut tuple_struct = TupleStructStruct::default();

        let toml_value = tuple_struct_struct_toml();
        LoadTupleStruct {
            tuple_struct_info: tuple_struct.reflect_type_info().as_tuple_struct().unwrap(),
            table: toml_value.as_array().unwrap(),
            tuple_struct: &mut tuple_struct,
        }
        .load_tuple_struct();

        assert_eq!(tuple_struct, TupleStructStruct(TupleStructTest(1, 2)));
    }
}
