use bevy::prelude::warn;
use bevy::reflect::{PartialReflect, Type};

use super::StructureLoader;

pub struct LoadValue<'a> {
    pub value_info: &'a Type,
    pub toml_value: &'a toml::Value,
    pub value: &'a mut dyn PartialReflect,
}

impl StructureLoader for LoadValue<'_> {
    fn load(self) {
        let value_info = self.value_info;
        match self.toml_value {
            toml::Value::String(str_val) => {
                if value_info.is::<String>() {
                    self.value.apply(str_val);
                } else {
                    warn!("Preferences: Expected {:?}, got String", value_info);
                }
            }
            toml::Value::Integer(int_val) => {
                if value_info.is::<f64>() {
                    self.value.apply(&(*int_val as f64));
                } else if value_info.is::<f32>() {
                    self.value
                        .apply(&((*int_val as f64).clamp(f32::MIN as f64, f32::MAX as f64) as f32));
                } else if value_info.is::<i64>() {
                    self.value.apply(int_val);
                } else if value_info.is::<i32>() {
                    self.value.apply(&(*int_val as i32));
                } else if value_info.is::<i16>() {
                    self.value.apply(&(*int_val as i16));
                } else if value_info.is::<i8>() {
                    self.value.apply(&(*int_val as i8));
                } else if value_info.is::<u64>() {
                    self.value.apply(&((*int_val).max(0) as u64));
                } else if value_info.is::<u32>() {
                    self.value.apply(&((*int_val).max(0) as u32));
                } else if value_info.is::<u16>() {
                    self.value.apply(&((*int_val).max(0) as u16));
                } else if value_info.is::<u8>() {
                    self.value.apply(&((*int_val).max(0) as u8));
                } else {
                    warn!("Preferences: Expected {:?}, got Integer", value_info);
                }
            }
            toml::Value::Float(float_val) => {
                if value_info.is::<f64>() {
                    self.value.apply(float_val);
                } else if value_info.is::<f32>() {
                    self.value
                        .apply(&(float_val.clamp(f32::MIN as f64, f32::MAX as f64) as f32));
                } else {
                    warn!("Preferences: Expected {:?}, got Float", value_info);
                }
            }
            toml::Value::Boolean(bool_val) => {
                if value_info.is::<bool>() {
                    self.value.apply(bool_val);
                } else {
                    warn!("Preferences: Expected {:?}, got Bool", value_info);
                }
            }
            value => {
                warn!("Preferences: Unsupported type: {:?}", value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::DynamicTyped as _;

    use super::*;

    #[tracing_test::traced_test]
    #[test]
    fn load_str() {
        let mut value = "".to_string();
        let toml_value = &toml::Value::String("Hello".to_string());
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, "Hello");
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_float_f64() {
        let mut value = 0.0;
        let toml_value = &toml::Value::Float(std::f64::consts::PI);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, std::f64::consts::PI);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_float_f32() {
        let mut value = 0.0_f32;
        let toml_value = &toml::Value::Float(std::f64::consts::PI);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, std::f32::consts::PI);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_bool() {
        let mut value = false;
        let toml_value = &toml::Value::Boolean(true);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert!(value);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_float_from_int_f64() {
        let mut value = 0.0;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42.0);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_float_from_int_f32() {
        let mut value = 0.0_f32;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42.0);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_int() {
        let mut value = 0;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_u8() {
        let mut value = 0_u8;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_u16() {
        let mut value = 0_u16;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_u32() {
        let mut value = 0_u32;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_u64() {
        let mut value = 0_u64;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_i8() {
        let mut value = 0_i8;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_i16() {
        let mut value = 0_i16;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_i32() {
        let mut value = 0_i32;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }

    #[tracing_test::traced_test]
    #[test]
    fn load_i64() {
        let mut value = 0_i64;
        let toml_value = &toml::Value::Integer(42);
        LoadValue {
            value_info: value.reflect_type_info().as_opaque().unwrap().ty(),
            toml_value,
            value: &mut value,
        }
        .load();
        assert_eq!(value, 42);
    }
}
