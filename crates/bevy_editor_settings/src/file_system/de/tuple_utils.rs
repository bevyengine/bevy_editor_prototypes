use bevy::reflect::{TupleInfo, TupleStructInfo, TupleVariantInfo, UnnamedField};

pub(super) trait TupleLikeInfo {
    fn field_at(&self, index: usize) -> Option<&UnnamedField>;
    fn field_len(&self) -> usize;
}

impl TupleLikeInfo for TupleInfo {
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    fn field_at(&self, index: usize) -> Option<&UnnamedField> {
        Self::field_at(self, index)
    }
}

impl TupleLikeInfo for TupleStructInfo {
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    fn field_at(&self, index: usize) -> Option<&UnnamedField> {
        Self::field_at(self, index)
    }
}

impl TupleLikeInfo for TupleVariantInfo {
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    fn field_at(&self, index: usize) -> Option<&UnnamedField> {
        Self::field_at(self, index)
    }
}
