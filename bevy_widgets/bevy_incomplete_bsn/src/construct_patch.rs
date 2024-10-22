//! Construct patch is a auto generated patch for every type that implements the Construct trait

use bevy::prelude::*;

use crate::{construct::Construct, patch::Patch};

pub struct ConstructPatch<T: Construct, F> {
    pub func: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T, F> ConstructPatch<T, F>
where
    T: Construct,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, F> Patch for ConstructPatch<T, F>
where
    T: Construct + Bundle + Default + Clone,
    F: FnMut(&mut T::Props) + Send + Sync + 'static,
{
    type Construct = T;
    fn patch(&mut self, props: &mut T::Props) {
        (self.func)(props);
    }
}
