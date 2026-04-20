use bevy::{reflect::Struct, utils::hashbrown::HashMap};

use crate::utils::struct_utils::StructLikeInfo;

use super::{DiffType, SaveStructures, StructureSaver};

pub struct StructSaver<'a> {
    pub strct: &'a dyn Struct,
    pub toml: &'a mut toml::value::Table,
}

impl<'a> StructureSaver for StructSaver<'a> {
    fn save(self, input: DiffType) {
        if let DiffType::Struct(results) = input {
            for (key, value) in results {
                if let Some(field) = self.strct.field(&key) {
                    let mut toml_value = toml::Value::Boolean(false);
                    SaveStructures {
                        toml: &mut toml_value,
                        structure: field,
                    }
                    .save(value);
                    self.toml.insert(key, toml_value);
                }
            }
        }
    }
}
