use bevy::reflect::{OpaqueInfo, PartialReflect, TypeInfo};

use super::{DiffType, StructureSaver};

pub struct OpaqueSaver<'a> {
    pub value: &'a dyn PartialReflect,
    pub toml: &'a mut toml::Value,
}

impl<'a> StructureSaver for OpaqueSaver<'a> {
    fn save(self, input: DiffType) {
        if let DiffType::Opaque = input {
            let type_info = self.value.get_represented_type_info().unwrap();

            check_and_save_opaque::<bool, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Boolean(*value);
            });
            check_and_save_opaque::<u8, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Integer(*value as i64);
            });
            check_and_save_opaque::<u16, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Integer(*value as i64);
            });
            check_and_save_opaque::<u32, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Integer(*value as i64);
            });
            check_and_save_opaque::<u64, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Integer(*value as i64);
            });
            check_and_save_opaque::<i32, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Integer(*value as i64);
            });
            check_and_save_opaque::<i64, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Integer(*value as i64);
            });
            check_and_save_opaque::<f32, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Float(f64::from(*value));
            });
            check_and_save_opaque::<f64, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::Float(*value);
            });
            check_and_save_opaque::<String, _>(type_info, self.value, self.toml, |value, toml| {
                *toml = toml::Value::String(value.clone());
            });
        }
    }
}

#[inline]
fn check_and_save_opaque<T, F>(
    type_info: &TypeInfo,
    value: &dyn PartialReflect,
    toml: &mut toml::Value,
    f: F,
) where
    T: 'static,
    F: FnOnce(&T, &mut toml::Value),
{
    if type_info.is::<T>() {
        let value = value.try_downcast_ref::<T>().unwrap();
        f(value, toml);
    }
}
