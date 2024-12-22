use bevy::reflect::Struct;

use super::{struct_utils::StructLikeInfo, LoadStructure};

pub struct LoadStruct<'a> {
    pub struct_info: &'a dyn StructLikeInfo,
    pub table: &'a toml::Table,
    pub strct: &'a mut dyn Struct,
}

impl<'a> LoadStruct<'a> {
    pub fn load_struct(self) {
        let struct_info = self.struct_info;
        let table = self.table;
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
        table.insert("float".to_string(), toml::Value::Float(3.14));
        table.insert("float32".to_string(), toml::Value::Float(3.14));
        table
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_struct_basic_values() {
        let mut struct_info = Values::default();
        let table = values_toml();

        LoadStruct {
            struct_info: struct_info.reflect_type_info().as_struct().unwrap(),
            table: &table,
            strct: &mut struct_info,
        }
        .load_struct();

        assert_eq!(
            struct_info,
            Values {
                string: "Hello".to_string(),
                float: 3.14,
                float32: 3.14,
            }
        );
    }

    #[derive(Debug, Clone, Reflect, Default, PartialEq)]
    struct StructWithStruct {
        values: Values,
    }

    fn load_struct_with_struct_toml() -> toml::value::Table {
        let mut table = toml::value::Table::default();
        table.insert("values".to_string(), toml::Value::Table(values_toml()));
        table
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_struct_with_struct() {
        let mut struct_info = StructWithStruct::default();

        let table = load_struct_with_struct_toml();

        LoadStruct {
            struct_info: struct_info.reflect_type_info().as_struct().unwrap(),
            table: &table,
            strct: &mut struct_info,
        }
        .load_struct();

        assert_eq!(
            struct_info,
            StructWithStruct {
                values: Values {
                    string: "Hello".to_string(),
                    float: 3.14,
                    float32: 3.14,
                },
            }
        );
    }
}

// fn load_struct<S: StructLikeInfo>(
//     strct: &mut dyn Struct,
//     struct_info: &'static S,
//     table: &toml::Table,
// ) {
//     let mut dynamic_struct = DynamicStruct::default();
//     for i in 0..struct_info.field_len() {
//         let field = struct_info.field_at(i).unwrap();
//         let key = field.name();

//         let Some(toml_value) = table.get(key) else {
//             continue;
//         };

//         let field_mut = strct.field_at_mut(i).unwrap();
//         let field_attrs = field.custom_attributes();
//         match field.type_info().unwrap() {
//             TypeInfo::Value(value_info) => {
//                 let value = load_value_boxed(value_info, toml_value);
//                 if let Some(value) = value {
//                     field_mut.apply(value.as_ref());
//                 }
//                 // load_value(field_mut, value_info, value)
//             }
// TypeInfo::Struct(struct_info) => {
//     if let Some(table) = toml_value.as_table() {
//         let ReflectMut::Struct(strct) = field_mut.reflect_mut() else {
//             warn!("Preferences: Expected Struct");
//             continue;
//         };
//         load_struct(strct, struct_info, table);
//     }
// }
// TypeInfo::List(list_info) => {
//     if let Some(table) = toml_value.as_array() {
//         let ReflectMut::List(list) = field_mut.reflect_mut() else {
//             warn!("Preferences: Expected List");
//             continue;
//         };
//         load_list(list, list_info, table, field_attrs);
//     }
// }
// TypeInfo::Array(array_info) => {
//     if let Some(table) = toml_value.as_array() {
//         let ReflectMut::Array(array) = field_mut.reflect_mut() else {
//             warn!("Preferences: Expected Array");
//             continue;
//         };
//         load_array(array, array_info, table);
//     }
// }
// TypeInfo::Enum(enum_info) => {
//     let ReflectMut::Enum(enm) = field_mut.reflect_mut() else {
//         warn!("Preferences: Expected Enum");
//         continue;
//     };

//     load_enum(enm, enum_info, toml_value);
// }
// TypeInfo::TupleStruct(tuple_struct_info) => {
//     let ReflectMut::TupleStruct(tuple_struct) = field_mut.reflect_mut() else {
//         warn!("Preferences: Expected TupleStruct");
//         continue;
//     };
//     if let Some(array_value) = toml_value.as_array() {
//         load_tuple_struct(tuple_struct, tuple_struct_info, array_value);
//     }
// }
//             _ => {
//                 warn!(
//                     "Preferences: Unsupported type: {:?}",
//                     field_mut.get_represented_type_info()
//                 );
//             }
//         }
//     }
// }
