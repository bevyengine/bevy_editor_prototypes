//! This module provides the patch trait, which is used to update widget tree

use bevy::prelude::*;
use crate::construct::Construct;

pub trait Patch: Send + Sync + 'static {
    type Construct: Construct + Bundle + Default + Clone;

    fn patch(&mut self, props: &mut <<Self as Patch>::Construct as Construct>::Props);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Clone, Component)]
    struct TestProps {
        value: i32,
    }

    struct TestPatch;

    impl Patch for TestPatch {
        type Construct = TestProps;
        fn patch(&mut self, props: &mut TestProps) {
            props.value += 1;
        }
    }

    #[test]
    fn test_patch_trait() {
        let mut props = TestProps { value: 0 };
        let mut patch = TestPatch;
        patch.patch(&mut props);
        assert_eq!(props.value, 1);
    }
}
