use bevy::reflect::{Set, SetInfo};

use super::{value::LoadValue, StructureLoader};

pub struct LoadSet<'a> {
    pub set: &'a mut dyn Set,
    pub set_info: &'a SetInfo,
}

impl<'a> StructureLoader for LoadSet<'a> {
    type Input = &'a toml::value::Array;

    fn load(self, input: Self::Input) {
        for toml_value in input.iter() {
            let mut value = super::default::default_value(&self.set_info.value_ty()).unwrap();

            LoadValue {
                value_info: &self.set_info.value_ty(),
                value: value.as_mut(),
            }
            .load(toml_value);

            self.set.insert_boxed(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::{DynamicTyped as _, TypeInfo};

    use super::*;

    #[tracing_test::traced_test]
    #[test]
    fn load_set() {
        let mut set: std::collections::HashSet<u32> = std::collections::HashSet::new();

        let toml_value = toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)]);

        let TypeInfo::Set(set_info) = set.reflect_type_info() else {
            panic!("Expected Set TypeInfo");
        };

        LoadSet {
            set_info,
            set: &mut set,
        }
        .load(toml_value.as_array().unwrap());

        assert_eq!(set.len(), 2);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
    }
}
