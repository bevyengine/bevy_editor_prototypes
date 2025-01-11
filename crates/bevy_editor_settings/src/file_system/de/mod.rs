mod array;
mod default;
mod enums;
mod list;
mod map;
mod set;
mod struct_utils;
mod structs;
mod tuple;
mod tuple_struct;
mod tuple_utils;
mod value;

use array::LoadArray;
use bevy::{
    prelude::*,
    reflect::{attributes::CustomAttributes, ReflectFromPtr, ReflectMut, TypeInfo},
};
use enums::LoadEnum;
use heck::ToSnakeCase;
use list::LoadList;
use map::LoadMap;
use set::LoadSet;
use structs::LoadStruct;
use tuple::LoadTuple;
use tuple_struct::LoadTupleStruct;
use value::LoadValue;

use crate::{SettingKey, SettingsType};

/// Errors that can occur when loading a TOML file.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
}

/// Load a toml file from the given path
pub fn load_toml_file(path: impl AsRef<std::path::Path>) -> Result<toml::Table, LoadError> {
    let path = path.as_ref();
    let file = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&file)?)
}

pub struct LoadStructure<'a> {
    pub type_info: &'static TypeInfo,
    pub table: &'a toml::Value,
    pub structure: &'a mut dyn PartialReflect,
    pub custom_attributes: Option<&'a CustomAttributes>,
}

impl LoadStructure<'_> {
    pub fn load(self) {
        match self.type_info {
            TypeInfo::Opaque(opaque_info) => {
                LoadValue {
                    value_info: opaque_info.ty(),
                    toml_value: self.table,
                    value: self.structure,
                }
                .load_value();
            }
            TypeInfo::Struct(struct_info) => {
                if let Some(table) = self.table.as_table() {
                    let ReflectMut::Struct(strct) = self.structure.reflect_mut() else {
                        warn!("Preferences: Expected Struct");
                        return;
                    };
                    LoadStruct {
                        struct_info,
                        table,
                        strct,
                    }
                    .load_struct();
                }
            }
            TypeInfo::TupleStruct(tuple_struct_info) => {
                if let Some(array_value) = self.table.as_array() {
                    let ReflectMut::TupleStruct(tuple_struct) = self.structure.reflect_mut() else {
                        warn!("Preferences: Expected TupleStruct");
                        return;
                    };
                    LoadTupleStruct {
                        tuple_struct_info,
                        table: array_value,
                        tuple_struct,
                    }
                    .load_tuple_struct();
                }
            }
            TypeInfo::Tuple(tuple_info) => {
                if let Some(array_value) = self.table.as_array() {
                    let ReflectMut::Tuple(tuple) = self.structure.reflect_mut() else {
                        warn!("Preferences: Expected Tuple");
                        return;
                    };
                    LoadTuple {
                        tuple_info,
                        table: array_value,
                        tuple,
                    }
                    .load_tuple();
                }
            }
            TypeInfo::List(list_info) => {
                if let Some(array_value) = self.table.as_array() {
                    let ReflectMut::List(list) = self.structure.reflect_mut() else {
                        warn!("Preferences: Expected List");
                        return;
                    };
                    LoadList {
                        list_info,
                        list,
                        toml_array: array_value,
                        custom_attributes: self.custom_attributes,
                    }
                    .load_list();
                }
            }
            TypeInfo::Array(array_info) => {
                if let Some(array_value) = self.table.as_array() {
                    let ReflectMut::Array(array) = self.structure.reflect_mut() else {
                        warn!("Preferences: Expected Array");
                        return;
                    };
                    LoadArray {
                        array_info,
                        array,
                        toml_array: array_value,
                    }
                    .load_array();
                }
            }
            TypeInfo::Map(map_info) => {
                if let Some(toml_map) = self.table.as_table() {
                    let ReflectMut::Map(map) = self.structure.reflect_mut() else {
                        warn!("Preferences: Expected Map");
                        return;
                    };
                    LoadMap {
                        map_info,
                        map,
                        table: toml_map,
                    }
                    .load_map();
                }
            }
            TypeInfo::Set(set_info) => {
                if let Some(toml_array) = self.table.as_array() {
                    let ReflectMut::Set(set) = self.structure.reflect_mut() else {
                        warn!("Preferences: Expected Set");
                        return;
                    };
                    LoadSet {
                        set_info,
                        set,
                        toml_array,
                    }
                    .load_set();
                }
            }
            TypeInfo::Enum(enum_info) => {
                let ReflectMut::Enum(enm) = self.structure.reflect_mut() else {
                    warn!("Preferences: Expected Enum");
                    return;
                };

                LoadEnum {
                    enum_info,
                    enm,
                    toml_value: self.table,
                }
                .load_enum();
            }
        }
    }
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
                    let toml_key = struct_info.custom_attributes().get::<SettingKey>();
                    if let Some(s_type) = s_type {
                        if settings_type != *s_type {
                            continue;
                        }
                        let mut ptr = world.get_resource_mut_by_id(res_id).unwrap();
                        let reflect_from_ptr = type_reg.data::<ReflectFromPtr>().unwrap();
                        #[allow(unsafe_code)]
                        let ReflectMut::Struct(strct) =
                            // SAFETY: `value` is of type `Reflected`, which the `ReflectFromPtr` was created for
                            unsafe { reflect_from_ptr.as_reflect_mut(ptr.as_mut()) }.reflect_mut()
                        else {
                            panic!("Expected Struct");
                        };

                        let name = toml_key
                            .map(|key| key.0.to_string())
                            .unwrap_or_else(|| strct.reflect_type_ident().unwrap().to_snake_case());

                        if let Some(table) = table.get(&name).and_then(|v| v.as_table()) {
                            LoadStruct {
                                struct_info,
                                table,
                                strct,
                            }
                            .load_struct();
                        }
                    }
                }
                TypeInfo::Enum(enum_info) => {
                    let s_type = enum_info.custom_attributes().get::<SettingsType>();
                    let toml_key = enum_info.custom_attributes().get::<SettingKey>();
                    if let Some(s_type) = s_type {
                        if settings_type != *s_type {
                            continue;
                        }
                        let mut ptr = world.get_resource_mut_by_id(res_id).unwrap();
                        let reflect_from_ptr = type_reg.data::<ReflectFromPtr>().unwrap();
                        #[allow(unsafe_code)]
                        let ReflectMut::Enum(enm) =
                            // SAFETY: `value` is of type `Reflected`, which the `ReflectFromPtr` was created for
                            unsafe { reflect_from_ptr.as_reflect_mut(ptr.as_mut()) }.reflect_mut()
                        else {
                            panic!("Expected Struct");
                        };

                        let name = toml_key
                            .map(|key| key.0.to_string())
                            .unwrap_or_else(|| enm.reflect_type_ident().unwrap().to_snake_case());

                        if let Some(table) = table.get(&name).and_then(|v| v.as_table()) {
                            if let Some(value) = table.get("variant") {
                                LoadEnum {
                                    enum_info,
                                    enm,
                                    toml_value: value,
                                }
                                .load_enum();
                            }
                        }
                    }
                }
                TypeInfo::TupleStruct(tuple_struct_info) => {
                    let s_type = tuple_struct_info.custom_attributes().get::<SettingsType>();
                    let toml_key = tuple_struct_info.custom_attributes().get::<SettingKey>();
                    if let Some(s_type) = s_type {
                        if settings_type != *s_type {
                            continue;
                        }
                        let mut ptr = world.get_resource_mut_by_id(res_id).unwrap();
                        let reflect_from_ptr = type_reg.data::<ReflectFromPtr>().unwrap();
                        #[allow(unsafe_code)]
                        let ReflectMut::TupleStruct(tuple_struct) =
                            // SAFETY: `value` is of type `Reflected`, which the `ReflectFromPtr` was created for
                            unsafe { reflect_from_ptr.as_reflect_mut(ptr.as_mut()) }.reflect_mut()
                        else {
                            panic!("Expected TupleStruct");
                        };

                        let name = toml_key.map(|key| key.0.to_string()).unwrap_or_else(|| {
                            tuple_struct.reflect_type_ident().unwrap().to_snake_case()
                        });

                        if let Some(table) = table.get(&name).and_then(|v| v.as_table()) {
                            if let Some(array_value) =
                                table.get("fields").and_then(|v| v.as_array())
                            {
                                LoadTupleStruct {
                                    tuple_struct_info,
                                    table: array_value,
                                    tuple_struct,
                                }
                                .load_tuple_struct();
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
