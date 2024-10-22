//! This module provides the construct trait, which is used to create widgets

use bevy::prelude::*;

use crate::{construct_context::ConstructContext, construct_patch::ConstructPatch};

pub trait Construct: Sized {
    type Props: Default + Clone;
    fn construct(context: &mut ConstructContext, props: Self::Props) -> Result<Self, ConstructError>;

    fn patch(func: impl FnMut(&mut Self::Props)) -> ConstructPatch<Self, impl FnMut(&mut Self::Props)> {
        ConstructPatch::new(func)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct ConstructError(pub String);

impl<T: Default + Clone> Construct for T {
    type Props = T;

    #[inline]
    fn construct(_context: &mut ConstructContext, props: Self::Props) -> Result<Self, ConstructError> {
        Ok(props)
    }
}



#[cfg(test)]
mod tests {
    use crate::patch::Patch;

    use super::*;

    #[derive(Default, Clone, Component)]
    struct TestProps {
        value: i32,
    }

    struct TestConstruct;

    impl Construct for TestConstruct {
        type Props = TestProps;

        fn construct(context: &mut ConstructContext, props: Self::Props) -> Result<Self, ConstructError> {
            context.world.entity_mut(context.id).insert(props.clone());
            Ok(TestConstruct)
        }
    }

    #[test]
    fn test_construct_trait() {
        let mut world = World::default();
        let entity = world.spawn_empty().id();
        let mut context = ConstructContext {
            id: entity,
            world: &mut world,
        };

        let props = TestProps { value: 42 };
        let result = TestConstruct::construct(&mut context, props);

        assert!(result.is_ok());
        assert_eq!(world.entity(entity).get::<TestProps>().map(|p| p.value), Some(42));
    }

    #[test]
    fn test_default_construct_implementation() {
        let mut world = World::default();
        let entity = world.spawn_empty().id();
        let mut context = ConstructContext {
            id: entity,
            world: &mut world,
        };

        let props = 123;
        let result = i32::construct(&mut context, props);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);
    }
    
    #[test]
    fn test_construct_transform() {
        let mut world = World::default();

        let mut transform = Transform::default();
        let mut patch = Transform::patch(|props| {
            props.translation.x = 1.0;
        });
        patch.patch(&mut transform);
        assert_eq!(transform.translation.x, 1.0);
    }
}

