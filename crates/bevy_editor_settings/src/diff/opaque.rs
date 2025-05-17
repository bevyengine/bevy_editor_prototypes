use bevy::reflect::{OpaqueInfo, PartialReflect};

use super::{Diff, DiffResult, DiffType};

macro_rules! diff_types {
    ($type_info:expr, $input1:expr, $input2:expr, $($type_v:ty),*) => {
        $(
            if diff_type::<$type_v>(&$type_info, $input1, $input2){
                return Some(DiffType::Opaque);
            };
        )*
    };

}

pub struct DiffOpaque<'a> {
    pub opaque_info: &'a OpaqueInfo,
}

impl<'a> Diff for DiffOpaque<'a> {
    type Input = &'a dyn PartialReflect;

    fn diff(&self, input1: Self::Input, input2: Self::Input) -> Option<DiffType> {
        diff_types!(
            self.opaque_info,
            input1,
            input2,
            bool,
            u8,
            u16,
            u32,
            u64,
            i32,
            i64,
            f32,
            f64,
            String,
            &str
        );

        None
    }
}

#[inline]
fn diff_type<T>(
    opaque_info: &OpaqueInfo,
    input1: &dyn PartialReflect,
    input2: &dyn PartialReflect,
) -> bool
where
    T: PartialEq + 'static,
{
    if opaque_info.is::<T>() {
        let value1 = input1.try_downcast_ref::<T>();
        let value2 = input2.try_downcast_ref::<T>();

        if value1.is_none() || value2.is_none() {
            return false;
        }
        let value1 = value1.unwrap();
        let value2 = value2.unwrap();
        if value1 != value2 {
            return true;
        }
    }
    false
}
