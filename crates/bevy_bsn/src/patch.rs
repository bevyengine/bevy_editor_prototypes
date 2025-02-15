use bevy::prelude::*;
use core::marker::PhantomData;
use variadics_please::all_tuples;

use crate::{Construct, ConstructContext, ConstructError, ConstructTuple};

/// A patch that can be applied to the props of a [`Construct`]able [`Bundle`].
///
/// [`Patch`] is implemented for functions that modify `Construct::Props`, aka [`ConstructPatch`]. It is also implemented for tuples of [`Patch`].
pub trait Patch: Send + Sync + 'static {
    /// The construct type whose props this patch can be applied to.
    type Construct: Construct + Bundle;
    /// Apply the patch to the supplied `props`.
    fn patch(self, props: &mut <Self::Construct as Construct>::Props);
}

// Tuple impls
macro_rules! impl_patch_for_tuple {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T: Patch),*> Patch for ($($T,)*) {
            type Construct = ConstructTuple<($($T::Construct,)*)>;

            #[allow(non_snake_case)]
            fn patch(self, props: &mut <Self::Construct as Construct>::Props) {
                let ($($T,)*) = self;
                let ($($t,)*) = props;
                $($T.patch($t);)*
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_patch_for_tuple,
    0,
    12,
    T,
    t
);

/// [`Patch`]-implementation that wraps a patch function and retains the [`Construct`] type.
#[derive(Debug)]
pub struct ConstructPatch<C: Construct, F> {
    pub(crate) func: F,
    pub(crate) _marker: PhantomData<C>,
}

impl<C, F> ConstructPatch<C, F>
where
    C: Construct<Props = C>,
    F: FnOnce(&mut C) + Sync + Send + 'static,
{
    /// Allows inferring the type of a bsn expression.
    ///
    /// Only works for types where the construct and props have the same type, as the [`Construct`] type cannot be inferred from props otherwise.
    pub fn new_inferred(func: F) -> Self {
        Self {
            func,
            _marker: PhantomData,
        }
    }
}

impl<C: Construct + Bundle, F: FnOnce(&mut C::Props) + Sync + Send + 'static> Patch
    for ConstructPatch<C, F>
{
    type Construct = C;
    fn patch(self, props: &mut <Self::Construct as Construct>::Props) {
        (self.func)(props);
    }
}

/// Extension trait for adding a [`ConstructPatchExt::patch`] helper to any types implementing [`Construct`].
pub trait ConstructPatchExt {
    /// Construct type
    type C: Construct;

    /// Returns a [`ConstructPatch`] wrapping the provided closure.
    fn patch<
        F: FnOnce(&mut <<Self as ConstructPatchExt>::C as Construct>::Props) + Send + Sync + 'static,
    >(
        func: F,
    ) -> ConstructPatch<Self::C, F> {
        ConstructPatch {
            func,
            _marker: PhantomData,
        }
    }
}

impl<C: Construct> ConstructPatchExt for C {
    type C = C;
}

/// Extension trait implementing patch utilities for [`ConstructContext`].
pub trait ConstructContextPatchExt {
    /// Construct an instance of `P::Construct` from a patch.
    fn construct_from_patch<P: Patch>(&mut self, patch: P) -> Result<P::Construct, ConstructError>
    where
        <<P as Patch>::Construct as Construct>::Props: Default;
}

impl ConstructContextPatchExt for ConstructContext<'_> {
    fn construct_from_patch<P: Patch>(&mut self, patch: P) -> Result<P::Construct, ConstructError>
    where
        <<P as Patch>::Construct as Construct>::Props: Default,
    {
        let mut props = <<P as Patch>::Construct as Construct>::Props::default();
        patch.patch(&mut props);
        self.construct(props)
    }
}

impl ConstructContextPatchExt for EntityWorldMut<'_> {
    fn construct_from_patch<P: Patch>(&mut self, patch: P) -> Result<P::Construct, ConstructError>
    where
        <<P as Patch>::Construct as Construct>::Props: Default,
    {
        let mut props = <<P as Patch>::Construct as Construct>::Props::default();
        patch.patch(&mut props);
        let id = self.id();
        self.world_scope(|world| ConstructContext::new(id, world).construct(props))
    }
}
