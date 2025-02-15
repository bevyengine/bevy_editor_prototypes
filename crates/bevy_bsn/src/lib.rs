//! BSN Prototype

#![allow(internal_features)]
#![cfg_attr(any(docsrs, docsrs_dep), feature(rustdoc_internals))]

extern crate alloc;
extern crate self as bevy_bsn;

mod bsn_asset;
mod bsn_helpers;
mod bsn_reflect;
mod construct;
mod construct_impls;
mod construct_reflect;
mod dynamic;
mod entity_patch;
mod patch;
mod retain;

use bevy::app::App;
use bevy::app::Plugin;

pub use bsn_asset::*;
pub use bsn_helpers::*;
pub use bsn_reflect::*;
pub use construct::*;
pub use construct_impls::*;
pub use construct_reflect::*;
pub use dynamic::*;
pub use entity_patch::*;
pub use patch::*;
pub use retain::*;

pub use bevy_bsn_macros::bsn;
pub use bevy_bsn_macros::Construct;

/// Adds support for BSN assets and reflection-based dynamic scenes.
pub struct BsnPlugin;

impl Plugin for BsnPlugin {
    fn build(&self, app: &mut App) {
        register_reflect_construct(app);
        bsn_asset_plugin(app);
        register_construct_impls(app);
    }
}
