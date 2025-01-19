//! BSN macros

use proc_macro::TokenStream;

mod bsn;
mod derive_construct;

/// The `bsn` macro is used to define scenes using BSN syntax.
///
/// Returns an `EntityPatch`.
#[proc_macro]
pub fn bsn(item: TokenStream) -> TokenStream {
    bsn::bsn(item.into()).into()
}

/// Derive macro for the `Construct` trait.
#[proc_macro_derive(Construct, attributes(construct))]
pub fn derive_construct(item: TokenStream) -> TokenStream {
    derive_construct::derive_construct(item.into()).into()
}
