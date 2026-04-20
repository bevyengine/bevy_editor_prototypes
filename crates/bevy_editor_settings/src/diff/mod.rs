mod opaque;
mod structs;

use bevy::{
    prelude::warn,
    reflect::{PartialReflect, Reflect, ReflectRef, Struct, TypeInfo},
    utils::HashMap,
};
use opaque::DiffOpaque;
use structs::DiffStructs;

#[derive(Debug, Clone)]
pub(crate) enum DiffType {
    Opaque,
    Struct(HashMap<String, DiffType>),
}

pub trait Diff {
    type Input;

    fn diff(&self, input1: Self::Input, input2: Self::Input) -> Option<DiffType>;
}

pub struct DiffStructures<'a> {
    pub type_info: &'a TypeInfo,
}

impl<'a> Diff for DiffStructures<'a> {
    type Input = &'a dyn PartialReflect;

    fn diff(&self, input1: Self::Input, input2: Self::Input) -> Option<DiffType> {
        match self.type_info {
            TypeInfo::Struct(struct_info) => match (input1.reflect_ref(), input2.reflect_ref()) {
                (ReflectRef::Struct(struct1), ReflectRef::Struct(struct2)) => {
                    DiffStructs { struct_info }.diff(struct1, struct2)
                }
                _ => {
                    warn!("Diffing not implemented for type: {:?}", self.type_info);
                    None
                }
            },
            TypeInfo::Opaque(opaque_info) => DiffOpaque { opaque_info }.diff(input1, input2),
            _ => {
                warn!("Diffing not implemented for type: {:?}", self.type_info);
                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DiffResult {
    field_name: String,
    old_value: String,
    new_value: String,
}

#[derive(Debug, Clone, Reflect)]
struct SomeStruct {
    a: u32,
    b: u32,
}

#[derive(Debug, Clone, Reflect)]
struct Wrapper {
    struct1: SomeStruct,
    something: u32,
}

#[cfg(test)]
mod tests {
    use bevy::reflect::DynamicTyped as _;

    use super::*;

    #[test]
    fn test_diff() {
        let struct1 = SomeStruct { a: 1, b: 2 };
        let struct2 = SomeStruct { a: 1, b: 3 };

        let wrapper1 = Wrapper {
            struct1,
            something: 1,
        };

        let wrapper2 = Wrapper {
            struct1: struct2,
            something: 1,
        };

        let diff = DiffStructures {
            type_info: wrapper1.reflect_type_info(),
        }
        .diff(&wrapper1, &wrapper2)
        .unwrap();

        println!("{:?}", diff);

        // assert_eq!(
        //     diff,
        //     vec![DiffResult {
        //         field_name: "b".to_string(),
        //         old_value: "2".to_string(),
        //         new_value: "3".to_string(),
        //     }]
        // );
    }
}
