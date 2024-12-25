mod structs;

use bevy::reflect::{PartialReflect, Reflect, Struct, TypeInfo};

pub trait Diff: Reflect {
    fn diff(&self, other: &Self) -> Vec<DiffResult>;
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

impl Diff for SomeStruct {
    fn diff(&self, other: &Self) -> Vec<DiffResult> {
        let mut results = Vec::new();
        let struct_self = self.as_partial_reflect().reflect_ref().as_struct().unwrap();
        let struct_other = other
            .as_partial_reflect()
            .reflect_ref()
            .as_struct()
            .unwrap();

        for index in 0..struct_self.field_len() {
            let field = struct_self.field_at(index).unwrap();
            let field_other = struct_other.field_at(index).unwrap();
            let type_info = field.get_represented_type_info().unwrap();
            if let TypeInfo::Opaque(_) = type_info {
                if field.represents::<u32>() {
                    let value = field.try_downcast_ref::<u32>().unwrap();
                    let value_other = field_other.try_downcast_ref::<u32>().unwrap();
                    if value != value_other {
                        results.push(DiffResult {
                            field_name: struct_self.name_at(index).unwrap().to_string(),
                            old_value: value.to_string(),
                            new_value: value_other.to_string(),
                        });
                    }
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff() {
        let a = SomeStruct { a: 1, b: 2 };
        let b = SomeStruct { a: 1, b: 3 };
        let res = a.diff(&b);
        println!("{:?}", res);
        assert_eq!(
            res,
            vec![DiffResult {
                field_name: "b".to_string(),
                old_value: "2".to_string(),
                new_value: "3".to_string(),
            }]
        );
    }
}
