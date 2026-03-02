use alloc::borrow::Cow;
use bevy::{
    ecs::{
        bundle::{BundleFromComponents, DynamicBundle},
        component::{ComponentId, Components, ComponentsRegistrator, StorageType},
        system::EntityCommands,
        world::error::EntityMutableFetchError,
    },
    prelude::*,
    ptr::{MovingPtr, OwningPtr, deconstruct_moving_ptr},
};
use thiserror::Error;
use variadics_please::all_tuples;

/// Error resulting from failing to [`Construct`] an instance of a type.
#[derive(Error, Debug)]
pub enum ConstructError {
    /// Custom error message
    #[error("{0}")]
    Custom(&'static str),
    /// Missing entity
    #[error(transparent)]
    MissingEntity(#[from] EntityMutableFetchError),
    /// Missing resource
    #[error("Resource {type_name} does not exist")]
    MissingResource {
        /// Name of the missing resource
        type_name: &'static str,
    },
    /// Invalid props
    #[error("Props were invalid: {message}")]
    InvalidProps {
        /// Message describing the invalid props
        message: Cow<'static, str>,
    },
}

/// A trait for types that require [`World`] state to be constructed, meaning they can't reliably implement [`Default`].
///
/// [`Construct`] types are constructed from their [`Construct::Props`].
///
/// This trait is blanket implemented with a "passthrough" for all types that are [`Default`] + [`Clone`]
/// to allow these types to be used in any place expecting a [`Construct`].
///
/// [`Construct`] can be derived using `#[derive(Construct)]`.
pub trait Construct: Sized {
    /// The type of props used to construct this type.
    type Props: Default + Clone;

    /// Construct an instance of this type from the given props.
    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError>;
}

// Blanket-implement Construct "passthrough" for all Default + Clone types.
impl<T: Default + Clone> Construct for T {
    type Props = T;
    #[inline]
    fn construct(
        _context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError> {
        Ok(props)
    }
}

/// Allows construction of [`Construct`] types either using their props or directly with an instance of the type itself.
#[derive(Reflect)]
pub enum ConstructProp<T: Construct> {
    /// Construct with an instance of `T`.
    Value(T),
    /// Construct using `T::Props`.
    Props(T::Props),
}

impl<T: Construct> ConstructProp<T> {
    /// Consumes the [`ConstructProp`] and returns the inner value, constructed if necessary.
    pub fn construct(self, context: &mut ConstructContext) -> Result<T, ConstructError> {
        match self {
            ConstructProp::Props(p) => Construct::construct(context, p),
            ConstructProp::Value(v) => Ok(v),
        }
    }
}

impl<C: Construct + Clone> Clone for ConstructProp<C>
where
    C::Props: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Props(p) => Self::Props(p.clone()),
            Self::Value(v) => Self::Value(v.clone()),
        }
    }
}

impl<T: Construct> From<T> for ConstructProp<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

/// Context for construction. Holds the entity id and a mutable borrow of the [`World`].
#[derive(Debug)]
pub struct ConstructContext<'a> {
    /// Entity to use for construction
    pub id: Entity,
    /// World
    pub world: &'a mut World,
}

impl<'a> ConstructContext<'a> {
    /// Create a new context for the given entity id and world.
    pub fn new(id: Entity, world: &'a mut World) -> Self {
        Self { id, world }
    }

    /// Construct an instance of `T` from the given props.
    pub fn construct<T: Construct>(
        &mut self,
        props: impl Into<T::Props>,
    ) -> Result<T, ConstructError> {
        T::construct(self, props.into())
    }
}

/// Construct extension
pub trait ConstructEntityCommandsExt {
    /// Construct a bundle using the given props and insert it onto the entity.
    fn construct<T: Construct + Bundle>(
        &mut self,
        props: impl Into<T::Props>,
    ) -> EntityCommands<'_>
    where
        <T as Construct>::Props: Send;
}

impl ConstructEntityCommandsExt for EntityCommands<'_> {
    fn construct<T: Construct + Bundle>(&mut self, props: impl Into<T::Props>) -> EntityCommands<'_>
    where
        <T as Construct>::Props: Send,
    {
        let props = props.into();
        self.queue(|mut entity: EntityWorldMut| {
            let id = entity.id();
            entity.world_scope(|world| {
                let mut context = ConstructContext { id, world };
                match T::construct(&mut context, props) {
                    Ok(c) => world.entity_mut(id).insert(c),
                    Err(e) => panic!("construction failed: {e}"),
                };
            });
        });
        self.reborrow()
    }
}

/// Allows construction of tuples of [`Construct`].
#[derive(Deref, DerefMut)]
pub struct ConstructTuple<C>(C);

macro_rules! impl_construct_tuple {
    ($(#[$meta:meta])* $(($T:ident, $t:ident)),*) => {
        $(#[$meta])*
        impl<$($T: Construct),*> Construct for ConstructTuple<($($T,)*)> {
            type Props = ($(<$T as Construct>::Props,)*);

            fn construct(
                _context: &mut ConstructContext,
                props: Self::Props,
            ) -> Result<Self, ConstructError> {
                let ($($t,)*) = props;
                $(let $t = $T::construct(_context, $t)?;)*
                Ok(Self(($($t,)*)))
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_construct_tuple,
    0,
    12,
    T,
    t
);

#[allow(unsafe_code)]
/// SAFETY: This just passes through to the inner [`Bundle`] implementation.
unsafe impl<B: Bundle> Bundle for ConstructTuple<B> {
    fn component_ids(components: &mut ComponentsRegistrator, ids: &mut impl FnMut(ComponentId)) {
        B::component_ids(components, ids);
    }

    fn get_component_ids(components: &Components, ids: &mut impl FnMut(Option<ComponentId>)) {
        B::get_component_ids(components, ids);
    }
}

#[allow(unsafe_code)]
/// SAFETY: This just passes through to the inner [`BundleFromComponents`] implementation.
unsafe impl<B: BundleFromComponents> BundleFromComponents for ConstructTuple<B> {
    unsafe fn from_components<T, F>(ctx: &mut T, func: &mut F) -> Self
    where
        // Ensure that the `OwningPtr` is used correctly
        F: for<'a> FnMut(&'a mut T) -> OwningPtr<'a>,
        Self: Sized,
    {
        ConstructTuple(
            // SAFETY: B::from_components has the same constraints as Self::from_components
            unsafe { B::from_components(ctx, func) },
        )
    }
}

impl<B: Bundle> DynamicBundle for ConstructTuple<B> {
    type Effect = ();

    #[allow(unused_variables, unsafe_code)]
    #[inline]
    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        deconstruct_moving_ptr!({
            let ConstructTuple { 0: field0 } = ptr;
        });

        // SAFETY: B::get_components has the same constraints as Self::get_components
        unsafe { <B as DynamicBundle>::get_components(field0, func) };
    }

    #[allow(unused_variables, unsafe_code)]
    #[inline]
    unsafe fn apply_effect(
        _ptr: MovingPtr<'_, std::mem::MaybeUninit<Self>>,
        _entity: &mut EntityWorldMut,
    ) {
    }
}
