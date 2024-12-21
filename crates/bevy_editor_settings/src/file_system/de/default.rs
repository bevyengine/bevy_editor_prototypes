use bevy::reflect::{
    ArrayInfo, DynamicArray, DynamicEnum, DynamicList, DynamicMap, DynamicSet, DynamicStruct, DynamicTuple, EnumInfo, ListInfo, MapInfo, PartialReflect, SetInfo, Type, TypeInfo, ValueInfo
};

use super::{struct_utils::StructLikeInfo, tuple_utils::TupleLikeInfo};

pub fn default_data_type(type_info: &TypeInfo) -> Option<Box<dyn PartialReflect>> {
    match type_info {
        TypeInfo::Value(value_info) => default_value(value_info.ty()),
        TypeInfo::Struct(struct_info) => {
            default_struct(struct_info).map(|s| Box::new(s) as Box<dyn PartialReflect>)
        }
        TypeInfo::TupleStruct(tuple_struct_info) => {
            default_tuple(tuple_struct_info).map(|t| Box::new(t) as Box<dyn PartialReflect>)
        }
        TypeInfo::Tuple(tuple_info) => {
            default_tuple(tuple_info).map(|t| Box::new(t) as Box<dyn PartialReflect>)
        }
        TypeInfo::Array(type_info) => {
            default_array(type_info).map(|a| Box::new(a) as Box<dyn PartialReflect>)
        }
        TypeInfo::List(type_info) => {
            default_list(type_info).map(|l| Box::new(l) as Box<dyn PartialReflect>)
        }
        TypeInfo::Map(type_info) => {
            default_map(type_info).map(|m| Box::new(m) as Box<dyn PartialReflect>)
        }
        TypeInfo::Set(type_info) => {
            default_set(type_info).map(|s| Box::new(s) as Box<dyn PartialReflect>)
        }
        TypeInfo::Enum(type_info) => {
            default_enum(type_info).map(|e| Box::new(e) as Box<dyn PartialReflect>)
        }
    }
}

pub fn default_enum(_type_info: &EnumInfo) -> Option<DynamicEnum> {
    Some(DynamicEnum::default())
}

pub fn default_set(_type_info: &SetInfo) -> Option<DynamicSet> {
    let output = DynamicSet::default();
    Some(output)
}

pub fn default_map(_type_info: &MapInfo) -> Option<DynamicMap> {
    let output = DynamicMap::default();
    Some(output)
}

pub fn default_list(_type_info: &ListInfo) -> Option<DynamicList> {
    let output = DynamicList::default();
    Some(output)
}

pub fn default_array(_type_info: &ArrayInfo) -> Option<DynamicList> {
    let output = DynamicList::default();
    Some(output)
}

pub fn default_value(type_info: &Type) -> Option<Box<dyn PartialReflect>> {
    if type_info.is::<String>() {
        Some(Box::new(String::default()))
    } else if type_info.is::<f64>() {
        Some(Box::new(f64::default()))
    } else if type_info.is::<f32>() {
        Some(Box::new(f32::default()))
    } else if type_info.is::<i64>() {
        Some(Box::new(i64::default()))
    } else if type_info.is::<i32>() {
        Some(Box::new(i32::default()))
    } else if type_info.is::<i16>() {
        Some(Box::new(i16::default()))
    } else if type_info.is::<i8>() {
        Some(Box::new(i8::default()))
    } else if type_info.is::<u64>() {
        Some(Box::new(u64::default()))
    } else if type_info.is::<u32>() {
        Some(Box::new(u32::default()))
    } else if type_info.is::<u16>() {
        Some(Box::new(u16::default()))
    } else if type_info.is::<u8>() {
        Some(Box::new(u8::default()))
    } else if type_info.is::<bool>() {
        Some(Box::new(bool::default()))
    } else {
        None
    }
}

pub fn default_struct<S: StructLikeInfo>(type_info: &S) -> Option<DynamicStruct> {
    let mut dyn_struct = DynamicStruct::default();
    // dyn_struct.set_represented_type(type_info);

    for i in 0..type_info.field_len() {
        let field_at = type_info.field_at(i).unwrap();

        let Some(value) = default_data_type(field_at.type_info().unwrap()) else {
            return None;
        };

        dyn_struct.insert_boxed(field_at.name(), value);
    }

    Some(dyn_struct)
}

pub fn default_tuple<S: TupleLikeInfo>(type_info: &S) -> Option<DynamicTuple> {
    let mut tuple = DynamicTuple::default();

    for i in 0..type_info.field_len() {
        let field_at = type_info.field_at(i).unwrap();

        let Some(value) = default_data_type(field_at.type_info().unwrap()) else {
            return None;
        };

        tuple.insert_boxed(value);
    }

    Some(tuple)
}
