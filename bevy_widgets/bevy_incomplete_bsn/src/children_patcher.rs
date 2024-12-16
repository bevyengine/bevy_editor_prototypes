use std::sync::Arc;

use bevy::{prelude::*, reflect::FromType};

use crate::entity_diff_tree::DiffTree;

/// A mark component that want to transform diff tree
/// For example, collapsing header want to wrap all children in a collapsable node
pub trait ChildrenPatcher: Send + Sync + 'static {
    fn children_patch(&mut self, children: &mut Vec<DiffTree>);
}

#[derive(Clone)]
pub struct ReflectChildrenPatcher {
    pub func: Arc<dyn Fn(&mut dyn Reflect, &mut Vec<DiffTree>) + Send + Sync + 'static>,
}

impl<T: Reflect + ChildrenPatcher> FromType<T> for ReflectChildrenPatcher {
    fn from_type() -> Self {
        Self {
            func: Arc::new(move |reflect, children| {
                let typed = reflect.downcast_mut::<T>().unwrap();
                typed.children_patch(children)
            }),
        }
    }
}
