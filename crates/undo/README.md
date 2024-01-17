This crate responsible for managing the systems that handle editor undo functionality.

This crates automaticly collect changes of component in Change chain and revert change with UndoRedo::Undo event.

Example usage
```rust
use bevy::prelude::*;
use undo::*;

fn main() {
    let mut app = App::new()
      .add_plugins((DefaultPlugins, UndoPlugin)) //Add Undo plugin
      .auto_undo::<Name>() //register Name component in undo system
      .auto_reflected_undo::<Parent>() //example register components that not implement Clone, but implement Reflect
      .auto_reflected_undo::<Children>()
      .add_systems(Startup, setup)
      .run();
  }
  
fn setup(
        mut cmds : Commands) {
    cmds.spawn((
    UndoMarker, //Mark entity to be used in undo logic
    Name::new("Some name")
  ));
}
  
  fn do_undo(
      mut events: EventWriter<UndoRedo>) {
    events.send(UndoRedo::Undo); //Will delete Name component
  }
```
