//! An interactive, reflection-based inspector for Bevy ECS data in running applications.
//!
//! Data can be viewed and modified in real-time, with changes being reflected in the application.

use bevy::prelude::*;
use bevy_pane_layout::prelude::*;

/// Pane for displaying the properties of the selected object.
#[derive(Component)]
pub struct PropertiesPane;

impl Pane for PropertiesPane {
    const NAME: &str = "Properties";
    const ID: &str = "properties";

    fn build(_app: &mut App) {
        // todo
    }

    fn creation_system() -> impl System<In = In<PaneStructure>, Out = ()> {
        IntoSystem::into_system(|_structure: In<PaneStructure>| {
            //todo
        })
    }
}

/// an add function that adds two numbers
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
