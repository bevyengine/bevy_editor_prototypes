use bevy::reflect::Struct;

use crate::utils::struct_utils::StructLikeInfo;

use super::{LoadStructure, StructureLoader};

pub struct LoadStruct<'a> {
    pub struct_info: &'a dyn StructLikeInfo,
    pub strct: &'a mut dyn Struct,
}

impl<'a> StructureLoader for LoadStruct<'a> {
    type Input = &'a toml::Table;

    fn load(self, input: Self::Input) {
        let struct_info = self.struct_info;
        let table = input;
        let strct = self.strct;
        for i in 0..struct_info.field_len() {
            let field = struct_info.field_at(i).unwrap();
            let key = field.name();

            let Some(toml_value) = table.get(key) else {
                continue;
            };

            let field_mut = strct.field_at_mut(i).unwrap();
            let field_attrs = field.custom_attributes();
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
    use bevy::reflect::{DynamicTyped, Reflect};

    use super::*;

    #[derive(Debug, Clone, Reflect, Default, PartialEq)]
    struct Values {
        pub string: String,
        pub float: f64,
        pub float32: f32,
    }

    fn values_toml() -> toml::value::Table {
        let mut table = toml::value::Table::default();
        table.insert(
            "string".to_string(),
            toml::Value::String("Hello".to_string()),
        );
        table.insert(
            "float".to_string(),
            toml::Value::Float(std::f64::consts::PI),
        );
        table.insert(
            "float32".to_string(),
            toml::Value::Float(std::f64::consts::PI),
        );
        table
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_basic_values() {
        let mut struct_info = Values::default();
        let table = values_toml();

        LoadStruct {
            struct_info: struct_info.reflect_type_info().as_struct().unwrap(),
            strct: &mut struct_info,
        }
        .load(&table);

        assert_eq!(
            struct_info,
            Values {
                string: "Hello".to_string(),
                float: std::f64::consts::PI,
                float32: std::f32::consts::PI,
            }
        );
    }

    #[derive(Debug, Clone, Reflect, Default, PartialEq)]
    struct StructWithStruct {
        values: Values,
    }

    fn load_with_struct_toml() -> toml::value::Table {
        let mut table = toml::value::Table::default();
        table.insert("values".to_string(), toml::Value::Table(values_toml()));
        table
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_with_struct() {
        let mut struct_info = StructWithStruct::default();

        let table = load_with_struct_toml();

        LoadStruct {
            struct_info: struct_info.reflect_type_info().as_struct().unwrap(),
            strct: &mut struct_info,
        }
        .load(&table);

        assert_eq!(
            struct_info,
            StructWithStruct {
                values: Values {
                    string: "Hello".to_string(),
                    float: std::f64::consts::PI,
                    float32: std::f32::consts::PI,
                },
            }
        );
    }
}
