use std::any::TypeId;

use bevy::{
    prelude::*,
    reflect::{
        attributes::CustomAttributes, Array, ArrayInfo, DynamicEnum, DynamicTuple, DynamicVariant,
        Enum, EnumInfo, List, ListInfo, ReflectFromPtr, ReflectMut, StructInfo, TypeInfo,
        ValueInfo, VariantInfo,
    },
    scene::ron::{de, value},
};
use heck::ToSnakeCase;

use crate::{MergeStrategy, SettingsTags, SettingsType};

/// Load a toml file from the given path
pub fn load_toml_file(path: impl AsRef<std::path::Path>) -> Result<toml::Table, LoadError> {
    let path = path.as_ref();
    let file = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&file)?)
}

/// Errors that can occur when loading a TOML file.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
}

/// check that the settings type matchs the settings type of the file
/// if they don't match, skip the settings
macro_rules! check_settings_type {
    ($settings_type:expr, $file_settings_type:expr) => {
        if $settings_type != $file_settings_type {
            continue;
        }
    };
}

pub fn load_preferences(world: &mut World, table: toml::Table, settings_type: SettingsType) {
    let registry = world.get_resource::<AppTypeRegistry>().unwrap().clone();
    // get all resources that
    let resources = world
        .iter_resources()
        .filter_map(|(res, _)| res.type_id().map(|type_id| (type_id, res.id())))
        .collect::<Vec<_>>();

    for (type_id, res_id) in resources {
        if let Some(type_reg) = registry.read().get(type_id) {
            match type_reg.type_info() {
                TypeInfo::Struct(struct_info) => {
                    let s_type = struct_info.custom_attributes().get::<SettingsType>();
                    if let Some(s_type) = s_type {
                        check_settings_type!(settings_type, *s_type);
                        let mut ptr = world.get_resource_mut_by_id(res_id).unwrap();
                        let reflect_from_ptr = type_reg.data::<ReflectFromPtr>().unwrap();
                        // SAFE: `value` is of type `Reflected`, which the `ReflectFromPtr` was created for
                        #[allow(unsafe_code)]
                        let ReflectMut::Struct(strct) =
                            unsafe { reflect_from_ptr.as_reflect_mut(ptr.as_mut()) }.reflect_mut()
                        else {
                            panic!("Expected Struct");
                        };

                        let name = strct.reflect_type_ident().unwrap().to_snake_case();

                        if let Some(table) = table.get(&name).and_then(|v| v.as_table()) {
                            load_struct(strct, struct_info, table);
                        }
                    }
                }
                TypeInfo::Enum(enum_info) => {
                    let s_type = enum_info.custom_attributes().get::<SettingsType>();
                    if let Some(s_type) = s_type {
                        check_settings_type!(settings_type, *s_type);
                        let mut ptr = world.get_resource_mut_by_id(res_id).unwrap();
                        let reflect_from_ptr = type_reg.data::<ReflectFromPtr>().unwrap();
                        // SAFE: `value` is of type `Reflected`, which the `ReflectFromPtr` was created for
                        #[allow(unsafe_code)]
                        let ReflectMut::Enum(enm) =
                            unsafe { reflect_from_ptr.as_reflect_mut(ptr.as_mut()) }.reflect_mut()
                        else {
                            panic!("Expected Struct");
                        };

                        let name = enm.reflect_type_ident().unwrap().to_snake_case();

                        if let Some(table) = table.get(&name).and_then(|v| v.as_table()) {
                            if let Some(value) = table.get("variant") {
                                load_enum(enm, enum_info, value);
                            }
                        }
                    }
                }

                _ => {
                    warn!("Preferences: Unsupported type: {:?}", type_reg.type_info());
                }
            }
        }
        // println!("Saving preferences for {:?}", res.name());
    }
}

fn load_struct(strct: &mut dyn Struct, struct_info: &StructInfo, table: &toml::Table) {
    for i in 0..strct.field_len() {
        let key = strct.name_at(i).unwrap().to_string();
        let field_mut = strct.field_at_mut(i).unwrap();
        let field_attrs = struct_info.field_at(i).unwrap().custom_attributes();
        match field_mut.get_represented_type_info().unwrap() {
            TypeInfo::Value(value_info) => {
                if let Some(value) = table.get(&key) {
                    load_value(field_mut, value_info, value)
                }
            }
            TypeInfo::Struct(struct_info) => {
                if let Some(table) = table.get(&key).and_then(|v| v.as_table()) {
                    let ReflectMut::Struct(strct) = field_mut.reflect_mut() else {
                        warn!("Preferences: Expected Struct");
                        continue;
                    };
                    load_struct(strct, struct_info, table);
                }
            }
            TypeInfo::List(list_info) => {
                if let Some(table) = table.get(&key).and_then(|v| v.as_array()) {
                    let ReflectMut::List(list) = field_mut.reflect_mut() else {
                        warn!("Preferences: Expected List");
                        continue;
                    };
                    load_list(list, list_info, table, field_attrs);
                }
            }
            TypeInfo::Array(array_info) => {
                if let Some(table) = table.get(&key).and_then(|v| v.as_array()) {
                    let ReflectMut::Array(array) = field_mut.reflect_mut() else {
                        warn!("Preferences: Expected Array");
                        continue;
                    };
                    load_array(array, array_info, table);
                }
            }
            TypeInfo::Enum(enum_info) => {
                let ReflectMut::Enum(enm) = field_mut.reflect_mut() else {
                    warn!("Preferences: Expected Enum");
                    continue;
                };
                if let Some(value) = table.get(&key) {
                    load_enum(enm, enum_info, value);
                }
            }
            _ => {
                warn!(
                    "Preferences: Unsupported type: {:?}",
                    field_mut.get_represented_type_info()
                );
            }
        }
    }
}

fn load_enum(enm: &mut dyn Enum, enum_info: &EnumInfo, toml_value: &toml::Value) {
    match toml_value {
        toml::Value::String(str_val) => {
            if let Some(VariantInfo::Unit(variant)) = enum_info.variant(str_val) {
                let dyn_enum = DynamicEnum::new(variant.name(), DynamicVariant::Unit);
                enm.apply(&dyn_enum);
            } else {
                warn!("Preferences: Unknown variant: {}", str_val);
            }
        }
        toml::Value::Table(table) => {}
        _ => {
            warn!("Preferences: Unsupported type: {:?}", toml_value);
        }
    }
}

fn load_list(
    list: &mut dyn List,
    list_info: &ListInfo,
    array: &toml::value::Array,
    attrs: &CustomAttributes,
) {
    let default = MergeStrategy::default();
    let merge_strategy = attrs.get::<MergeStrategy>().unwrap_or(&default);

    if let Some(item_info) = list_info.item_info() {
        match merge_strategy {
            MergeStrategy::Replace => {
                while list.len() > 0 {
                    list.remove(list.len() - 1);
                }
            }
            MergeStrategy::Append => {
                // do nothing
            }
        }
        for value in array.iter() {
            match item_info {
                TypeInfo::Value(value_info) => {
                    let value = load_value_boxed(value_info, value);
                    if let Some(value) = value {
                        list.push(value);
                    }
                }
                TypeInfo::Enum(enum_info) => {
                    let mut enum_value = DynamicEnum::default();
                    load_enum(&mut enum_value, enum_info, value);
                    list.push(Box::new(enum_value));
                }
                // TODO support more then values in lists
                _ => {
                    warn!("Preferences: Unsupported type: {:?}", item_info);
                }
            }
        }
    }
}

fn load_array(array: &mut dyn Array, array_info: &ArrayInfo, table: &toml::value::Array) {
    warn!("Preferences: Arrays are not supported yet");
}

fn load_value_boxed(
    value_info: &ValueInfo,
    value: &toml::Value,
) -> Option<Box<dyn PartialReflect>> {
    match value {
        toml::Value::String(str_val) => {
            if value_info.is::<String>() {
                Some(Box::new(str_val.clone()))
            } else {
                warn!("Preferences: Expected {:?}, got String", value_info);
                None
            }
        }
        toml::Value::Integer(int_val) => {
            if value_info.is::<f64>() {
                Some(Box::new(*int_val as f64))
            } else if value_info.is::<f32>() {
                Some(Box::new(
                    (*int_val).clamp(f32::MIN as i64, f32::MAX as i64) as f32
                ))
            } else if value_info.is::<i64>() {
                Some(Box::new(*int_val))
            } else if value_info.is::<i32>() {
                Some(Box::new(
                    (*int_val).clamp(i32::MIN as i64, i32::MAX as i64) as i32
                ))
            } else if value_info.is::<i16>() {
                Some(Box::new(
                    (*int_val).clamp(i16::MIN as i64, i16::MAX as i64) as i16
                ))
            } else if value_info.is::<i8>() {
                Some(Box::new(
                    (*int_val).clamp(i8::MIN as i64, i8::MAX as i64) as i8
                ))
            } else if value_info.is::<u64>() {
                Some(Box::new((*int_val).max(0) as u64))
            } else if value_info.is::<u32>() {
                Some(Box::new((*int_val).max(0) as u32))
            } else if value_info.is::<u16>() {
                Some(Box::new((*int_val).max(0) as u16))
            } else if value_info.is::<u8>() {
                Some(Box::new((*int_val).max(0) as u8))
            } else {
                warn!("Preferences: Expected {:?}, got Integer", value_info);
                None
            }
        }
        toml::Value::Float(float_val) => {
            if value_info.is::<f64>() {
                Some(Box::new(*float_val))
            } else if value_info.is::<f32>() {
                Some(Box::new(
                    float_val.clamp(f32::MIN as f64, f32::MAX as f64) as f32
                ))
            } else {
                warn!("Preferences: Expected {:?}, got Float", value_info);
                None
            }
        }
        toml::Value::Boolean(bool_val) => {
            if value_info.is::<bool>() {
                Some(Box::new(*bool_val))
            } else {
                warn!("Preferences: Expected {:?}, got Bool", value_info);
                None
            }
        }
        _ => {
            warn!("Preferences: Unsupported type: {:?}", value);
            None
        }
    }
}

fn load_value(field: &mut dyn PartialReflect, value_info: &ValueInfo, value: &toml::Value) {
    match value {
        toml::Value::String(str_val) => {
            if value_info.is::<String>() {
                field.apply(str_val);
            } else {
                warn!("Preferences: Expected {:?}, got String", value_info);
            }
        }
        toml::Value::Integer(int_val) => {
            if value_info.is::<f64>() {
                field.apply(&(*int_val as f64));
            } else if value_info.is::<f32>() {
                field.apply(&((*int_val).clamp(f32::MIN as i64, f32::MAX as i64) as f32));
            } else if value_info.is::<i64>() {
                field.apply(int_val);
            } else if value_info.is::<i32>() {
                field.apply(&((*int_val).clamp(i32::MIN as i64, i32::MAX as i64) as i32));
            } else if value_info.is::<i16>() {
                field.apply(&((*int_val).clamp(i16::MIN as i64, i16::MAX as i64) as i16));
            } else if value_info.is::<i8>() {
                field.apply(&((*int_val).clamp(i8::MIN as i64, i8::MAX as i64) as i8));
            } else if value_info.is::<u64>() {
                field.apply(&((*int_val).max(0) as u64));
            } else if value_info.is::<u32>() {
                field.apply(&((*int_val).max(0) as u32));
            } else if value_info.is::<u16>() {
                field.apply(&((*int_val).max(0) as u16));
            } else if value_info.is::<u8>() {
                field.apply(&((*int_val).max(0) as u8));
            } else {
                warn!("Preferences: Expected {:?}, got Integer", value_info);
            }
        }
        toml::Value::Float(float_val) => {
            if value_info.is::<f64>() {
                field.apply(float_val);
            } else if value_info.is::<f32>() {
                field.apply(&(float_val.clamp(f32::MIN as f64, f32::MAX as f64) as f32));
            } else {
                warn!("Preferences: Expected {:?}, got Float", value_info);
            }
        }
        toml::Value::Boolean(bool_val) => {
            if value_info.is::<bool>() {
                field.apply(bool_val);
            } else {
                warn!("Preferences: Expected {:?}, got Bool", value_info);
            }
        }
        _ => {
            warn!("Preferences: Unsupported type: {:?}", value);
        }
    }
}