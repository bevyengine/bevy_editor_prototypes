use bevy::{
    prelude::warn,
    reflect::{Map, MapInfo},
};

use super::LoadStructure;

pub struct LoadMap<'a> {
    pub map: &'a mut dyn Map,
    pub map_info: &'a MapInfo,
    pub table: &'a toml::value::Table,
}

impl<'a> LoadMap<'a> {
    pub fn load_map(self) {
        if !self
            .map_info
            .key_info()
            .map(|info| info.is::<String>())
            .unwrap_or(false)
        {
            warn!("Preferences: Map key must be a String");
            return;
        }

        for (key, toml_value) in self.table.iter() {
            let Some(value_info) = self.map_info.value_info() else {
                warn!("Preferences: Expected Map value info");
                return;
            };

            let Some(mut value) = super::default::default_data_type(value_info) else {
                warn!("Unable to create default value for map item");
                return;
            };

            LoadStructure {
                type_info: value_info,
                table: toml_value,
                structure: value.as_mut(),
                custom_attributes: None,
            }
            .load();

            self.map.insert_boxed(Box::new(key.clone()), value);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::DynamicTyped as _;

    use super::*;

    #[tracing_test::traced_test]
    #[test]
    fn load_map() {
        let mut map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        let mut table = toml::value::Table::default();
        table.insert("key".to_string(), toml::Value::Integer(1));

        LoadMap {
            map_info: map.reflect_type_info().as_map().unwrap(),
            map: &mut map,
            table: &table,
        }
        .load_map();

        assert_eq!(map.get("key"), Some(&1));
    }
}
