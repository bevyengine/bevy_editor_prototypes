use bevy::{
    prelude::warn,
    reflect::{List, ListInfo, attributes::CustomAttributes},
};

use crate::MergeStrategy;

use super::LoadStructure;

pub struct LoadList<'a> {
    pub list_info: &'a ListInfo,
    pub list: &'a mut dyn List,
    pub toml_array: &'a toml::value::Array,
    pub custom_attributes: Option<&'a CustomAttributes>,
}

impl LoadList<'_> {
    pub fn load_list(self) {
        let merge_strategy = self
            .custom_attributes
            .and_then(|attrs| attrs.get::<MergeStrategy>())
            .cloned()
            .unwrap_or_default();

        let Some(item_info) = self.list_info.item_info() else {
            warn!("Preferences: Expected List item info");
            return;
        };

        if let MergeStrategy::Replace = merge_strategy {
            self.list.drain();
        }

        for toml_value in self.toml_array.iter() {
            let Some(mut value) = super::default::default_data_type(item_info) else {
                warn!("Unable to create default value for list item");
                return;
            };

            LoadStructure {
                type_info: item_info,
                table: toml_value,
                structure: value.as_mut(),
                custom_attributes: None,
            }
            .load();

            self.list.push(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::{DynamicTyped as _, Reflect};

    use super::*;

    #[tracing_test::traced_test]
    #[test]
    fn load_list() {
        let mut list: Vec<u32> = Vec::new();

        let toml_value = toml::Value::Array(vec![toml::Value::Integer(1), toml::Value::Integer(2)]);
        LoadList {
            list_info: list.reflect_type_info().as_list().unwrap(),
            list: &mut list,
            toml_array: toml_value.as_array().unwrap(),
            custom_attributes: None,
        }
        .load_list();
        assert_eq!(list, vec![1, 2]);
    }

    #[derive(Debug, Clone, PartialEq, Reflect, Default)]
    struct TestMergeStrategy {
        #[reflect(@MergeStrategy::Append)]
        pub list: Vec<u32>,
    }

    fn list_test_toml() -> toml::Value {
        toml::Value::Array(vec![toml::Value::Integer(3), toml::Value::Integer(4)])
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_list_with_merge_strategy() {
        let mut list = TestMergeStrategy::default();
        list.list.push(1);
        list.list.push(2);

        let attrs = list
            .reflect_type_info()
            .as_struct()
            .unwrap()
            .field_at(0)
            .unwrap()
            .custom_attributes();

        let toml_value = list_test_toml();
        LoadList {
            list_info: list.list.reflect_type_info().as_list().unwrap(),
            list: &mut list.list,
            toml_array: toml_value.as_array().unwrap(),
            custom_attributes: Some(attrs),
        }
        .load_list();
        assert_eq!(list.list, vec![1, 2, 3, 4]);
    }
}
