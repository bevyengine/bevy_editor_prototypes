//! A set of camera controllers suitable for controlling editor-style views and scene exploration.

pub mod editor_camera_2d;
#[allow(missing_docs)]
pub mod editor_camera_3d;

// TODO: Figure out if a prelude should be used instead here.
pub use editor_camera_2d::*;
#[expect(unused_imports)]
pub use editor_camera_3d::*;
