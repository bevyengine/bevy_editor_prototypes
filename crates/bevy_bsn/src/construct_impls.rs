use alloc::borrow::Cow;

use bevy::{
    ecs::component::{ComponentHooks, Immutable, StorageType},
    prelude::*,
    text::{FontSmoothing, LineHeight},
};

use crate::*;

pub(crate) fn register_construct_impls(app: &mut App) {
    app.register_type::<ConstructEntity>();
    app.register_type::<ConstructTextFont>();
    app.register_type::<ConstructTextFontProps>();
}

/// Constructable asset handle (because [`Handle<T>`] implements Default in Bevy right now)
#[derive(Deref, DerefMut, Clone, Reflect, Debug)]
#[reflect(Construct)]
pub struct ConstructHandle<T: Asset>(pub Handle<T>);

impl<T: Asset> From<Handle<T>> for ConstructHandle<T> {
    fn from(value: Handle<T>) -> Self {
        ConstructHandle(value)
    }
}

impl<T: Asset> From<ConstructHandle<T>> for Handle<T> {
    fn from(value: ConstructHandle<T>) -> Self {
        value.0
    }
}

impl<T: Asset> Construct for ConstructHandle<T> {
    // TODO: AssetPath currently doesn't work with reflected assets because of Into. Is it reflectable?
    //type Props = AssetPath<'static>;
    type Props = String;

    fn construct(
        context: &mut ConstructContext,
        path: Self::Props,
    ) -> Result<Self, ConstructError> {
        Ok(context.world.resource::<AssetServer>().load(path).into())
    }
}

/// [`Entity`] constructable using [`EntityPath`], allowing passing either entity name or id as prop.
///
/// This exists because we can't implement [`Construct`] for any foreign types. When [`Construct`] is available upstream this should no longer be needed.
#[derive(Deref, DerefMut, Debug, Copy, Clone, Reflect)]
#[reflect(Construct)]
pub struct ConstructEntity(pub Entity);

impl From<Entity> for ConstructEntity {
    fn from(value: Entity) -> Self {
        ConstructEntity(value)
    }
}

impl From<ConstructEntity> for Entity {
    fn from(value: ConstructEntity) -> Self {
        value.0
    }
}

/// The construct prop for [`ConstructEntity`].
#[derive(Default, Debug, Clone, Reflect)]
pub enum EntityPath {
    /// None
    #[default]
    None,
    /// Name
    Name(Cow<'static, str>),
    /// Entity
    Entity(Entity),
}

impl From<&'static str> for EntityPath {
    fn from(value: &'static str) -> Self {
        Self::Name(value.into())
    }
}

impl From<String> for EntityPath {
    fn from(value: String) -> Self {
        Self::Name(value.into())
    }
}

impl From<Entity> for EntityPath {
    fn from(value: Entity) -> Self {
        Self::Entity(value)
    }
}

impl Construct for ConstructEntity {
    type Props = EntityPath;

    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError> {
        match props {
            EntityPath::Name(name) => {
                let mut query = context.world.query::<(Entity, &Name)>();
                let mut matching_entities = query
                    .iter(context.world)
                    .filter(|(_, q_name)| q_name.as_str() == name);

                match matching_entities.next() {
                    Some((entity, _)) => {
                        if matching_entities.next().is_some() {
                            return Err(ConstructError::InvalidProps {
                                message: format!("multiple entities with name '{}'", name).into(),
                            });
                        }
                        Ok(ConstructEntity(entity))
                    }
                    None => Err(ConstructError::InvalidProps {
                        message: format!("no entity with name '{}'", name).into(),
                    }),
                }
            }
            EntityPath::Entity(entity) => Ok(ConstructEntity(entity)),
            _ => Err(ConstructError::InvalidProps {
                message: "no entity supplied".into(),
            }),
        }
    }
}

/// Constructable text font. Workaround for default-implemented [`TextFont`] in Bevy.
#[derive(Clone, Debug, Reflect)]
#[reflect(Component, Construct)]
pub struct ConstructTextFont {
    /// Font
    pub font: ConstructHandle<Font>,
    /// Font size
    pub font_size: f32,
    /// Font smoothing
    pub font_smoothing: FontSmoothing,
    /// Line height
    pub line_height: LineHeight,
}

#[allow(missing_docs)]
#[derive(Clone, Reflect)]
pub struct ConstructTextFontProps {
    pub font: ConstructProp<ConstructHandle<Font>>,
    pub font_size: f32,
    pub font_smoothing: FontSmoothing,
    pub line_height: LineHeight,
}

impl Default for ConstructTextFontProps {
    fn default() -> Self {
        let default = TextFont::default();
        Self {
            font: ConstructProp::Value(default.font.into()),
            font_size: default.font_size,
            font_smoothing: default.font_smoothing,
            line_height: default.line_height,
        }
    }
}

impl Construct for ConstructTextFont {
    type Props = ConstructTextFontProps;
    fn construct(
        context: &mut ConstructContext,
        props: Self::Props,
    ) -> Result<Self, ConstructError> {
        Ok(Self {
            font: props.font.construct(context)?,
            font_size: props.font_size,
            font_smoothing: props.font_smoothing,
            line_height: props.line_height,
        })
    }
}

impl Component for ConstructTextFont {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    type Mutability = Immutable;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, context| {
            let constructable = world
                .get::<ConstructTextFont>(context.entity)
                .unwrap()
                .clone();
            world.commands().entity(context.entity).insert(TextFont {
                font: constructable.font.into(),
                font_size: constructable.font_size,
                font_smoothing: constructable.font_smoothing,
                line_height: constructable.line_height,
            });
        });
        hooks.on_remove(|mut world, context| {
            if let Some(mut entity) = world.commands().get_entity(context.entity) {
                entity.remove::<TextFont>();
            }
        });
    }
}
