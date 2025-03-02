//! BSN macros

use proc_macro::TokenStream;

mod bsn;
mod derive_construct;

/// Macro is used to author scenes using BSN syntax.
///
/// See the [BSN proposal](https://github.com/bevyengine/bevy/discussions/14437) for more information on the syntax.
///
/// # Example
/// ```ignore
/// use bevy::{color::palettes::css, prelude::*};
/// use bevy_proto_bsn::{Scene, *};
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(BsnPlugin)
///         .add_systems(Startup, |mut commands: Commands| {
///             commands.spawn(Camera2d);
///             commands.spawn_scene(my_scene());
///         })
///         .run();
/// }
///
/// fn my_scene() -> impl Scene {
///     pbsn! {
///         Node {
///             position_type: PositionType::Absolute,
///             flex_direction: FlexDirection::Column,
///             row_gap: px(10.0),
///             padding: px_all(10.0),
///         } [
///             // A text node with a custom font constructed from its asset path
///             (
///                 Text("Hello, World!"),
///                 ConstructTextFont { font_size: 24.0, font: @"Inter-Regular.ttf" }
///             ),
///
///             // A button with a click observer
///             (Button, Node { padding: px_all(5.0) }, BackgroundColor(css::LIGHT_SALMON)) [
///                 Text("Click me!"),
///                 On(|_: Trigger<Pointer<Click>>| {
///                     info!("Button clicked!");
///                 }),
///             ],
///         ]
///     }
/// }
/// ```
#[proc_macro]
pub fn pbsn(item: TokenStream) -> TokenStream {
    bsn::bsn(item.into()).into()
}

/// Derive macro for the `Construct` trait.
#[proc_macro_derive(Construct, attributes(no_reflect, construct))]
pub fn derive_construct(item: TokenStream) -> TokenStream {
    derive_construct::derive_construct(item.into()).into()
}
