use bevy::reflect::{NamedField, StructInfo, StructVariantInfo};
use core::slice::Iter;

/// A helper trait for accessing type information from struct-like types.
pub trait StructLikeInfo {
    #[allow(dead_code)]
    fn field(&self, name: &str) -> Option<&NamedField>;
    fn field_at(&self, index: usize) -> Option<&NamedField>;
    fn field_len(&self) -> usize;
    #[allow(dead_code)]
    fn iter_fields(&self) -> Iter<'_, NamedField>;
}

impl StructLikeInfo for StructInfo {
    fn field(&self, name: &str) -> Option<&NamedField> {
        Self::field(self, name)
    }

    fn field_at(&self, index: usize) -> Option<&NamedField> {
        Self::field_at(self, index)
    }

    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    fn iter_fields(&self) -> Iter<'_, NamedField> {
        self.iter()
    }
}

impl StructLikeInfo for StructVariantInfo {
    fn field(&self, name: &str) -> Option<&NamedField> {
        Self::field(self, name)
    }

    fn field_at(&self, index: usize) -> Option<&NamedField> {
        Self::field_at(self, index)
    }

    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    fn iter_fields(&self) -> Iter<'_, NamedField> {
        self.iter()
    }
}
