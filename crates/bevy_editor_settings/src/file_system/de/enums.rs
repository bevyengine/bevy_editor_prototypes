use bevy::{
    prelude::warn,
    reflect::{DynamicEnum, DynamicVariant, Enum, EnumInfo, VariantInfo},
};

use super::{structs::LoadStruct, tuple::LoadTuple, StructureLoader};

pub struct LoadEnum<'a> {
    pub enum_info: &'a EnumInfo,
    pub toml_value: &'a toml::Value,
    pub enm: &'a mut dyn Enum,
}

impl StructureLoader for LoadEnum<'_> {
    fn load(self) {
        match self.toml_value {
            toml::Value::String(str_val) => {
                if let Some(VariantInfo::Unit(variant)) = self.enum_info.variant(str_val) {
                    let dyn_enum = DynamicEnum::new(variant.name(), DynamicVariant::Unit);
                    self.enm.apply(&dyn_enum);
                } else {
                    warn!("Preferences: Unknown variant: {}", str_val);
                }
            }
            toml::Value::Table(table) => {
                if let Some(value) = self
                    .enum_info
                    .variant_names()
                    .iter()
                    .find(|name| table.contains_key(**name))
                {
                    let variant_info = self.enum_info.variant(value).unwrap();
                    let value = table.get(*value).unwrap();

                    match variant_info {
                        VariantInfo::Unit(variant) => {
                            let dyn_enum = DynamicEnum::new(variant.name(), DynamicVariant::Unit);
                            self.enm.apply(&dyn_enum);
                        }
                        VariantInfo::Struct(struct_info) => {
                            let Some(map) = value.as_table() else {
                                warn!("Preferences: Table");
                                return;
                            };

                            let Some(mut dyn_struct) = super::default::default_struct(struct_info)
                            else {
                                warn!("Preferences: Expected Struct");
                                return;
                            };

                            LoadStruct {
                                struct_info,
                                table: map,
                                strct: &mut dyn_struct,
                            }
                            .load();

                            let dyn_enum = DynamicEnum::new(
                                variant_info.name(),
                                DynamicVariant::Struct(dyn_struct),
                            );
                            self.enm.apply(&dyn_enum);
                        }
                        // TODO: handle single field tuple structs differently this could just be a raw value instead of an array
                        // VariantInfo::Tuple(tuple_variant_info)
                        //     if tuple_variant_info.field_len() == 1 && !value.is_array() => {
                        //         // TODO: This is a hack to support single field tuple structs
                        //     }
                        VariantInfo::Tuple(tuple_variant_info) => {
                            let Some(array) = value.as_array() else {
                                warn!("Preferences: Expected Array");
                                return;
                            };

                            let Some(mut dyn_tuple) =
                                super::default::default_tuple(tuple_variant_info)
                            else {
                                warn!("Preferences: Expected TupleStruct");
                                return;
                            };

                            LoadTuple {
                                tuple_info: tuple_variant_info,
                                table: array,
                                tuple: &mut dyn_tuple,
                            }
                            .load();

                            let dyn_enum = DynamicEnum::new(
                                variant_info.name(),
                                DynamicVariant::Tuple(dyn_tuple),
                            );
                            self.enm.apply(&dyn_enum);
                        }
                    }
                }
            }
            _ => {
                warn!("Preferences: Unsupported type: {:?}", self.toml_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::reflect::{DynamicTyped as _, Reflect};

    #[derive(Debug, Clone, PartialEq, Reflect, Default)]
    enum TestEnum {
        #[default]
        Variant1,
        Variant2(u32),
        Variant3 {
            name: String,
            age: u32,
        },
        Variant4(u32, u32),
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_enum_unit() {
        let mut enum_test = TestEnum::Variant2(0);

        let toml_value = toml::Value::String("Variant1".to_string());
        LoadEnum {
            enum_info: enum_test.reflect_type_info().as_enum().unwrap(),
            toml_value: &toml_value,
            enm: &mut enum_test,
        }
        .load();

        assert_eq!(enum_test, TestEnum::Variant1);
    }

    fn enum_test_toml() -> toml::Value {
        let mut table = toml::value::Table::new();
        let mut var3 = toml::value::Table::new();
        var3.insert("name".to_string(), toml::Value::String("John".to_string()));
        var3.insert("age".to_string(), toml::Value::Integer(10));
        table.insert("Variant3".to_string(), toml::Value::Table(var3));
        toml::Value::Table(table)
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_enum_struct() {
        let mut enum_test = TestEnum::default();

        let toml_value = enum_test_toml();
        LoadEnum {
            enum_info: enum_test.reflect_type_info().as_enum().unwrap(),
            toml_value: &toml_value,
            enm: &mut enum_test,
        }
        .load();

        assert_eq!(
            enum_test,
            TestEnum::Variant3 {
                name: "John".to_string(),
                age: 10,
            }
        );
    }

    fn enum_test_tuple_toml() -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert(
            "Variant4".to_string(),
            toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)]),
        );
        toml::Value::Table(table)
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_enum_tuple() {
        let mut enum_test = TestEnum::default();

        let toml_value = enum_test_tuple_toml();
        LoadEnum {
            enum_info: enum_test.reflect_type_info().as_enum().unwrap(),
            toml_value: &toml_value,
            enm: &mut enum_test,
        }
        .load();

        assert_eq!(enum_test, TestEnum::Variant4(1, 2));
    }
}
