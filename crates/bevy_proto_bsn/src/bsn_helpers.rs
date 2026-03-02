//! Helper functions for cleaner BSN templates.
use core::marker::PhantomData;

use bevy::{
    color::Color,
    ecs::{
        bundle::Bundle, component::Component, entity::Entity, event::Event, hierarchy::ChildOf,
        lifecycle::HookContext, observer::Observer, system::IntoObserverSystem,
        world::DeferredWorld,
    },
    ui::{UiRect, Val},
};

use crate::{Construct, ConstructEntity, ConstructProp, EntityPath};

/// Shorthand for [`UiRect::all`] + [`Val::Px`].
pub fn px_all(value: impl Into<f32>) -> UiRect {
    UiRect::all(Val::Px(value.into()))
}

/// Shorthand for [`Color::srgb_u8`].
pub fn rgb8(red: u8, green: u8, blue: u8) -> Color {
    Color::srgb_u8(red, green, blue)
}

/// Shorthand for [`Color::srgba_u8`].
pub fn rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
    Color::srgba_u8(red, green, blue, alpha)
}

/// A helper for adding observers to a `pbsn` macro invocation.
///
/// By default, observers will observe their parent entity. Optionally, a second argument can be passed to specify the entity to observe.
///
/// ```
/// # use bevy_proto_bsn::*;
/// # use bevy::prelude::*;
/// pbsn! {
///     {Name::new("MyEntity")} [
///         Obs(|trigger: On<Pointer<Click>>| {
///             // Do something when "MyEntity" is clicked.
///         }),
///         Obs(|trigger: On<Pointer<Drag>>| {
///             // Do something when "MyEntity" is dragged.
///         }),
///         {Name::new("MyChild")} [
///             Obs(|trigger: On<Pointer<Click>>| {
///                 // Do something when "MyEntity" is clicked.
///             }, @"MyEntity"),
///         ]
///     ]
/// };
/// ```
#[derive(Component)]
#[component(on_insert = insert_callback::<E, B, M, S>)]
#[component(on_remove = remove_callback)]
pub struct Obs<E, B, M, S>
where
    E: Event,
    B: Bundle,
    M: Sync + Send + 'static,
    S: IntoObserverSystem<E, B, M> + Sync + Send + 'static,
{
    observer: Option<Observer>,
    entity: Entity,
    marker: PhantomData<(E, B, M, S)>,
}

fn insert_callback<E, B, M, S>(mut world: DeferredWorld, context: HookContext)
where
    E: Event,
    B: Bundle,
    M: Sync + Send + 'static,
    S: IntoObserverSystem<E, B, M> + Sync + Send + 'static,
{
    let mut callback = world.get_mut::<Obs<E, B, M, S>>(context.entity).unwrap();
    let Some(mut observer) = core::mem::take(&mut callback.observer) else {
        return;
    };

    observer.watch_entity(callback.entity);

    let mut commands = world.commands();
    let mut entity_commands = commands.entity(context.entity);
    entity_commands.remove::<Observer>();
    entity_commands.insert(observer);
}

fn remove_callback(mut world: DeferredWorld, context: HookContext) {
    let mut commands = world.commands();
    commands.entity(context.entity).remove::<Observer>();
}

/// Props for constructing observers in bsn-macros. See [`Obs`].
pub struct ObsProps<E, B, M, S>(
    pub Option<S>,
    pub ConstructProp<ConstructEntity>,
    PhantomData<(E, B, M)>,
)
where
    E: Event,
    B: Bundle,
    S: IntoObserverSystem<E, B, M>;

impl<E, B, M, S> Default for ObsProps<E, B, M, S>
where
    E: Event,
    B: Bundle,
    S: IntoObserverSystem<E, B, M>,
{
    fn default() -> Self {
        Self(
            Default::default(),
            ConstructProp::Props(Default::default()),
            Default::default(),
        )
    }
}

impl<E, B, M, S> Clone for ObsProps<E, B, M, S>
where
    E: Event,
    B: Bundle,
    S: IntoObserverSystem<E, B, M>,
{
    fn clone(&self) -> Self {
        Self(None, self.1.clone(), Default::default())
    }
}

impl<E, B, M, S> Construct for Obs<E, B, M, S>
where
    E: Event,
    B: Bundle,
    M: Sync + Send + 'static,
    S: IntoObserverSystem<E, B, M> + Sync + Send + 'static,
{
    type Props = ObsProps<E, B, M, S>;

    fn construct(
        context: &mut crate::ConstructContext,
        props: Self::Props,
    ) -> Result<Self, crate::ConstructError> {
        let entity = match props.1 {
            ConstructProp::Value(entity) => entity.into(),
            ConstructProp::Props(props) => match props {
                EntityPath::None => context
                    .world
                    .get::<ChildOf>(context.id)
                    .map(ChildOf::parent)
                    .unwrap_or(context.id),
                EntityPath::Name(name) => context
                    .construct::<ConstructEntity>(EntityPath::Name(name))?
                    .into(),
                EntityPath::Entity(entity) => entity,
            },
        };

        Ok(Self {
            observer: props.0.map(Observer::new),
            entity,
            marker: Default::default(),
        })
    }
}
