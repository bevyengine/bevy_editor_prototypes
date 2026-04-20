mod opaque;
mod structs;

use bevy::reflect::{PartialReflect, ReflectRef};
use opaque::OpaqueSaver;
use structs::StructSaver;

use crate::diff::DiffType;

trait StructureSaver {
    fn save(self, input: DiffType);
}

struct SaveStructures<'a> {
    toml: &'a mut toml::Value,
    structure: &'a dyn PartialReflect,
}

impl<'a> StructureSaver for SaveStructures<'a> {
    fn save(self, input: DiffType) {
        match input {
            DiffType::Opaque => {
                let ReflectRef::Opaque(opaque_info) = self.structure.reflect_ref() else {
                    return;
                };

                OpaqueSaver {
                    value: opaque_info,
                    toml: self.toml,
                }
                .save(DiffType::Opaque);
            }
            DiffType::Struct(fields) => {
                let ReflectRef::Struct(structure) = self.structure.reflect_ref() else {
                    return;
                };
                let toml::Value::Table(table) = self.toml else {
                    return;
                };
                StructSaver {
                    strct: structure,
                    toml: table,
                }
                .save(DiffType::Struct(fields));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::DynamicTyped as _;
    use bevy::reflect::Reflect;
    use bevy::reflect::Struct;

    use crate::diff::Diff;
    use crate::diff::DiffStructures;

    use super::*;

    #[derive(Debug, Clone, Reflect, Default, PartialEq)]
    struct Values {
        pub int1: u32,
        pub int2: String,
    }

    #[test]
    fn test_save_struct() {
        let values1 = Values { int1: 1, int2: "Hello".to_string() };
        let values2 = Values { int1: 3, int2: "World".to_string() };
        let mut toml_value = toml::Value::Table(toml::value::Table::default());

        let toml_table = toml_value.as_table_mut().unwrap();
        toml_table.insert(
            "testing".to_string(),
            toml::Value::Table(toml::value::Table::default()),
        );

        let diff = DiffStructures {
            type_info: values1.reflect_type_info(),
        }
        .diff(&values1, &values2)
        .unwrap();

        SaveStructures {
            toml: toml_table.get_mut("testing").unwrap(),
            structure: &values2,
        }
        .save(diff);

        println!("{}", toml::to_string_pretty(&toml_table).unwrap());
    }
}
