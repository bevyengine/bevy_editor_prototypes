use bevy::reflect::TupleStruct;

use crate::utils::tuple_utils::TupleLikeInfo;

use super::{LoadStructure, StructureLoader};

pub struct LoadTupleStruct<'a> {
    pub tuple_struct_info: &'a dyn TupleLikeInfo,
    pub tuple_struct: &'a mut dyn TupleStruct,
}

impl<'a> StructureLoader for LoadTupleStruct<'a> {
    type Input = &'a toml::value::Array;

    fn load(self, input: Self::Input) {
        for i in 0..self.tuple_struct_info.field_len() {
            let Some(toml_value) = input.get(i) else {
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
                structure: field_mut,
                custom_attributes: Some(field_attrs),
            }
            .load(toml_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::{DynamicTyped as _, Reflect};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Reflect, Default)]
    struct TupleStructTest(u32, u32);

    fn tuple_struct_test_toml() -> toml::Value {
        toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)])
    }

    #[tracing_test::traced_test]
    #[test]
    fn load() {
        let mut tuple_struct = TupleStructTest::default();

        let toml_value = tuple_struct_test_toml();
        LoadTupleStruct {
            tuple_struct_info: tuple_struct.reflect_type_info().as_tuple_struct().unwrap(),
            tuple_struct: &mut tuple_struct,
        }
        .load(toml_value.as_array().unwrap());
        assert_eq!(tuple_struct, TupleStructTest(1, 2));
    }

    #[derive(Debug, Clone, PartialEq, Reflect, Default)]
    struct TupleStructStruct(TupleStructTest);

    fn tuple_struct_struct_toml() -> toml::Value {
        toml::Value::Array(vec![tuple_struct_test_toml()])
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_struct() {
        let mut tuple_struct = TupleStructStruct::default();

        let toml_value = tuple_struct_struct_toml();
        LoadTupleStruct {
            tuple_struct_info: tuple_struct.reflect_type_info().as_tuple_struct().unwrap(),
            tuple_struct: &mut tuple_struct,
        }
        .load(toml_value.as_array().unwrap());

        assert_eq!(tuple_struct, TupleStructStruct(TupleStructTest(1, 2)));
    }
}
