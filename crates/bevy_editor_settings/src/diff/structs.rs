use bevy::{reflect::Struct, utils::hashbrown::HashMap};

use crate::utils::struct_utils::StructLikeInfo;

use super::{Diff, DiffResult, DiffType};

pub struct DiffStructs<'a> {
    pub struct_info: &'a dyn StructLikeInfo,
}

impl<'a> Diff for DiffStructs<'a> {
    type Input = &'a dyn Struct;

    fn diff(&self, input1: Self::Input, input2: Self::Input) -> Option<DiffType> {
        let mut results = HashMap::new();

        for index in 0..self.struct_info.field_len() {
            let field = input1.field_at(index).unwrap();
            let field_other = input2.field_at(index).unwrap();
            let type_info = field.get_represented_type_info().unwrap();
            if let Some(output) = (super::DiffStructures { type_info }).diff(field, field_other) {
                results.insert(self.struct_info.field_at(index).unwrap().name().to_string(), output);
            }
        }
        
        if results.is_empty() {
            None
        } else {
            Some(DiffType::Struct(results))
        }
    }
}
