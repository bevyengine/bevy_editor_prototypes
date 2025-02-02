use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse2,
    punctuated::{Pair, Punctuated},
    Path,
};

use bevy_bsn_ast::*;

pub fn bsn(item: TokenStream) -> TokenStream {
    match parse2::<BsnAstEntity>(item) {
        Ok(bsn) => bsn.to_token_stream(),
        Err(e) => e.to_compile_error(),
    }
}

fn bevy_bsn_path() -> Path {
    Path::from(format_ident!("bevy_bsn"))
}

trait ToTokensInternal {
    fn to_tokens(&self, tokens: &mut TokenStream);

    fn to_token_stream(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        self.to_tokens(&mut tokens);
        tokens
    }
}

impl<T, P> ToTokensInternal for Punctuated<T, P>
where
    T: ToTokensInternal,
    P: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for pair in self.pairs() {
            match pair {
                Pair::Punctuated(a, b) => {
                    a.to_tokens(tokens);
                    b.to_tokens(tokens);
                }
                Pair::End(a) => a.to_tokens(tokens),
            }
        }
    }
}

impl ToTokensInternal for BsnAstEntity {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let bevy_bsn = bevy_bsn_path();
        let patch = &self.patch.to_token_stream();
        let inherits = self.inherits.iter().map(ToTokensInternal::to_token_stream);
        let children = self.children.iter().map(ToTokensInternal::to_token_stream);
        let key = self.key.to_token_stream();
        quote! {
            #bevy_bsn::EntityPatch {
                inherit: (#(#inherits,)*),
                patch: #patch,
                children: (#(#children,)*),
                key: #key,
            }
        }
        .to_tokens(tokens);
    }
}

impl ToTokensInternal for Option<BsnAstKey> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Some(BsnAstKey::Static(key)) => quote! {
                Some(#key.into())
            },
            Some(BsnAstKey::Dynamic(block)) => quote! {
                Some(#block.into())
            },
            None => quote! { None },
        }
        .to_tokens(tokens);
    }
}

impl ToTokensInternal for BsnAstPatch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let bevy_bsn = bevy_bsn_path();
        match self {
            BsnAstPatch::Patch(path, fields) => {
                let assignments = fields.iter().map(|(member, prop)| {
                    let member = member.to_token_stream();
                    let prop = prop.to_token_stream();
                    quote! {
                        props.#member = #prop;
                    }
                });
                quote! {
                    #path::patch(move |props| {
                        #(#assignments)*
                    })
                }
            }
            BsnAstPatch::Tuple(tuple) => {
                let tuple = tuple.to_token_stream();
                quote! {
                    (#tuple)
                }
            }
            BsnAstPatch::Expr(expr) => quote! {
                #bevy_bsn::ConstructPatch::new_inferred(move |props| {
                    *props = #expr;
                })
            },
        }
        .to_tokens(tokens);
    }
}

impl ToTokensInternal for BsnAstProp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let bevy_bsn = bevy_bsn_path();
        match self {
            BsnAstProp::Value(expr) => quote! {
                (#expr).into()
            },
            BsnAstProp::Props(expr) => quote! {
                #bevy_bsn::ConstructProp::Props((#expr).into())
            },
        }
        .to_tokens(tokens);
    }
}

impl ToTokensInternal for BsnAstInherit {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let BsnAstInherit(path, params) = &self;
        quote! {
            (#path (#params))
        }
        .to_tokens(tokens);
    }
}
