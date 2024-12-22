use bevy::reflect::{Set, SetInfo};

use super::value::LoadValue;

pub struct LoadSet<'a> {
    pub set: &'a mut dyn Set,
    pub set_info: &'a SetInfo,
    pub toml_array: &'a toml::value::Array,
}

impl<'a> LoadSet<'a> {
    pub fn load_set(self) {
        for toml_value in self.toml_array.iter() {
            let mut value = super::default::default_value(&self.set_info.value_ty()).unwrap();

            LoadValue {
                value_info: &self.set_info.value_ty(),
                toml_value,
                value: value.as_mut(),
            }
            .load_value();

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
            toml_array: toml_value.as_array().unwrap(),
            set: &mut set,
        }
        .load_set();

        assert_eq!(set.len(), 2);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
    }
}
