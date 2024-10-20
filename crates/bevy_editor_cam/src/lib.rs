//! A production-ready camera controller for 3D editors; intended for anyone who needs to rapidly
//! and intuitively navigate virtual spaces.
//!
//! Camera controllers are very subjective! As someone who has spent years using camera controllers
//! in mechanical engineering CAD software, I've developed my own opinions about what matters in a
//! camera controller. This is my attempt to make the controller I've always wanted, that fixes the
//! annoyances I've encountered.
//!
//! *Because* camera controllers are so subjective, I felt the need to write out the impetus for
//! making this thing, what matters to me, and how I decided between conflicting goals. Somehow,
//! this ended up as a manifesto of sorts. If you came here to learn how to use or extend this
//! plugin, I've boiled the manifesto down into two sentences:
//!
//! > A camera controller needs to be responsive, robust, and satisfying to use. When there is
//! > conflict between these needs, they should be prioritized in that order.
//!
//! Now that you've absorbed my wisdom, feel free to skip ahead to the [Usage](crate#usage) section.
//!
//! Or don't. It's up to you.
//!
//! # Philosophy
//!
//! These are the properties of a good editor camera controller, in order of importance. These are
//! the driving values for the choices I've made here. You might disagree and have different values
//! or priorities!
//!
//! ## Responsive
//!
//! A good camera controller should never feel floaty or disconnected. It should go exactly where
//! the user commands it to go. Responsiveness isn't simply "low latency", it's about respecting the
//! user's intent.
//!
//! #### First-order input
//!
//! The most precise inputs are first-order, that is, controlling the position of something
//! directly, instead of its velocity (second-order) or acceleration (third-order). An example of
//! this is using a mouse vs. a gamepad for controlling the rotation of a first person view. The
//! mouse is first order, the position of the mouse on the mousepad directly corresponds with the
//! direction the player is facing. Conversely, a joystick controls the velocity of the view
//! rotation. All that is to say, where possible, the camera controller should use pointer inputs
//! *directly*.
//!
//! #### Pixel-perfect panning
//!
//! When you click and drag to pan the scene, the thing you click on should stick to your pointer,
//! and never drift. This should hold true even if inputs are being smoothed.
//!
//! #### Intuitive zoom
//!
//! The camera should zoom in and out in the direction you are pointing. If the user is hovering
//! over something, the speed of the camera should automatically adjust to quickly zoom up to it
//! without clipping through it.
//!
//! #### Predictable rotation
//!
//! When you click and drag to orbit the scene in 3d, the center of rotation should be located where
//! your pointer was when the drag started.
//!
//! #### Intuitive perspective toggle
//!
//! Toggling between different fields of view, or between perspective and orthographic projections,
//! should not cause the camera view to jump or change suddenly. The view should smoothly warp,
//! keeping the last interacted point stationary on the screen.
//!
//! ## Robust
//!
//! A camera controller should work in any scenario, and handle failure gracefully and
//! unsurprisingly when inputs are ambiguous.
//!
//! #### Works in all conditions:
//!
//! All of features in the previous section should work regardless of framerate, distance, scale,
//! camera field of view, and camera projection - including orthographic.
//!
//! #### Graceful fallback
//!
//! if nothing is under the pointer when a camera motion starts, the last-known depth should be
//! used, to prevent erratic behavior when the hit test fails. If a user was orbiting around a point
//! on an object, then clicks to rotate about empty space, the camera should not shoot off into
//! space because nothing was under the cursor.
//!
//! ### Satisfying
//!
//! The controller should *feel* good to use.
//!
//! #### Momentum
//!
//! Panning and orbiting should support configurable momentum, to allow you to "flick" the camera
//! through the scene to cover distance and make the feel of the camera tunable. This is especially
//! useful for trackpad and touch users.
//!
//! #### Smoothness
//!
//! The smoothness of inputs should be configurable as a tradeoff between fluidity of motion and
//! responsiveness. This is particularly useful when showing the screen to other people, where fast
//! motions can be disorienting or even nauseating.
//!
//! # Usage
//!
//! This plugin only requires three things to work. The `bevy_mod_picking` plugin for hit tests, the
//! [`DefaultEditorCamPlugins`] plugin group, and the [`EditorCam`](crate::prelude::EditorCam)
//! component. Controller settings are configured per-camera in the
//! [`EditorCam`](crate::prelude::EditorCam) component.
//!
//! ## Getting Started
//!
//! #### 1. Add `bevy_mod_picking`
//!
//! The camera controller uses [`bevy_picking_core`] for pointer interactions. If you already use
//! the picking plugin, then using this camera controller is essentially free because it can reuse
//! those same hit tests you are already running.
//!
//! If you are not using the picking plugin yet, all you need to get started are the default
//! plugins:
//!
//! ```
//! # let mut app = bevy::app::App::new();
//! app.add_plugins(bevy_mod_picking::DefaultPickingPlugins);
//! ```
//!
//! #### 2. Add `DefaultEditorCamPlugins`
//!
//! This is a plugin group that adds the camera controller, as well as all the [extensions]. You can
//! instead add [`controller::MinimalEditorCamPlugin`], though you will need to add your own input
//! plugin if you do.
//!
//! ```
//! # let mut app = bevy::app::App::new();
//! app.add_plugins(bevy_editor_cam::DefaultEditorCamPlugins);
//! ```
//!
//! #### 3. Insert the `EditorCam` component
//!
//! Finally, insert [`controller::component::EditorCam`] onto any cameras that you want to control.
//! This marks the cameras as controllable and holds all camera controller settings.
//!
//! ```
//! # use bevy::ecs::system::Commands;
//! # use bevy_editor_cam::prelude::*;
//! # fn test(mut commands: Commands) {
//! commands.spawn((
//!     // Camera
//!     EditorCam::default(),
//! ));
//! # }
//! ```
//!
//! # Other notable features
//!
//! I've also implemented a few other features that are handy for a camera controller like this.
//!
//! ### Compatible with floating origins and other controllers
//!
//! This controller does all computations in view space. The result of this is that you can move the
//! camera wherever you want, update its transform, and it will continue to behave normally, as long
//! as the camera isn't being controlled by the user while you do this. This means you can control
//! this camera with another camera controller, or use it in a floating origin system.
//!
//! ### Independent skybox
//!
//! When working in a CAD context, it is common to use orthographic projections to remove
//! perspective distortion from the image. However, because an ortho projection has zero field of
//! view, the view of the skybox is infinitesimally small, i.e. only a single pixel of the skybox is
//! visible. To fix this, an [extension](extensions) is provided to attach a skybox to a camera that
//! is independent from that camera's field of view.
//!
//! ### Pointer and Hit Test Agnostic
//!
//! Users of this library shouldn't be forced into using any particular hit testing method, like CPU
//! raycasting. The controller uses [`bevy_picking_core`] to work with:
//!
//! - Arbitrary hit testing backends, including those written by users. See
//!   [`bevy_picking_core::backend`] for more information.
//! - Any number of pointing inputs, including touch.
//! - Viewports and multi-pass rendering.

#![warn(missing_docs)]

pub mod controller;
pub mod extensions;
pub mod input;

/// Common imports.
pub mod prelude {
    pub use crate::{
        controller::{component::*, *},
        DefaultEditorCamPlugins,
    };
}

use bevy_app::{prelude::*, PluginGroupBuilder};

/// Adds [`bevy_editor_cam`](crate) functionality with all extensions and the default input plugin.
pub struct DefaultEditorCamPlugins;

impl PluginGroup for DefaultEditorCamPlugins {
    #[allow(clippy::let_and_return)] // Needed for conditional compilation
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<Self>()
            .add(input::DefaultInputPlugin)
            .add(controller::MinimalEditorCamPlugin)
            .add(extensions::dolly_zoom::DollyZoomPlugin)
            .add(extensions::look_to::LookToPlugin);

        #[cfg(feature = "extension_anchor_indicator")]
        let group = group.add(extensions::anchor_indicator::AnchorIndicatorPlugin);

        #[cfg(feature = "extension_independent_skybox")]
        let group = group.add(extensions::independent_skybox::IndependentSkyboxPlugin);

        group
    }
}
