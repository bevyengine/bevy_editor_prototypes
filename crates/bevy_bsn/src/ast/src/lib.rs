//! Shared BSN AST core used by both macro and assets.

use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self, Brace, Paren},
    Expr, Member, Path, Result, Token,
};

pub use quote;
pub use syn;

/// Low-level `syn`-based bsn AST that may be used by both the macro and the asset loader.
pub struct BsnAstEntity {
    /// Inherited entities
    pub inherits: Punctuated<BsnAstInherit, Token![,]>,
    /// Comoponents patch
    pub patch: BsnAstPatch,
    /// Child entities
    pub children: Punctuated<BsnAstEntity, Token![,]>,
}

impl Parse for BsnAstEntity {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inherits = Punctuated::new();
        let patch;
        if input.peek(Paren) {
            let content;
            parenthesized![content in input];

            let mut patch_tuple = Punctuated::new();

            loop {
                if content.is_empty() {
                    break;
                }

                if content.peek(Token![:]) {
                    content.parse::<Token![:]>()?;
                    inherits = content.parse_terminated(BsnAstInherit::parse, Token![,])?;
                    break;
                }

                let patch = content.parse::<BsnAstPatch>()?;
                patch_tuple.push_value(patch);
                if content.is_empty() {
                    break;
                }

                if content.peek(Token![:]) || (content.peek(Token![,]) && content.peek2(Token![:]))
                {
                    content.parse::<Token![,]>().ok();
                    content.parse::<Token![:]>()?;
                    inherits = content.parse_terminated(BsnAstInherit::parse, Token![,])?;
                    break;
                }

                if content.peek(Token![,]) {
                    let punct = content.parse()?;
                    patch_tuple.push_punct(punct);
                }
            }

            patch = BsnAstPatch::Tuple(patch_tuple);
        } else {
            patch = BsnAstPatch::parse(input)?;
        }

        let children = if input.peek(token::Bracket) {
            let content;
            bracketed![content in input];
            content.parse_terminated(BsnAstEntity::parse, Token![,])?
        } else {
            Punctuated::new()
        };

        Ok(Self {
            inherits,
            patch,
            children,
        })
    }
}

/// AST for a BSN patch.
pub enum BsnAstPatch {
    /// Patch for a struct or enum
    Patch(Path, Vec<(Member, BsnAstProp)>),
    /// A tuple of patches separated by `,`
    Tuple(Punctuated<BsnAstPatch, Token![,]>),
    /// An expression, surrounded by braces in the BSN
    Expr(Expr),
}

impl Parse for BsnAstPatch {
    fn parse(input: ParseStream) -> Result<BsnAstPatch> {
        if input.peek(Paren) {
            // Tuple
            let content;
            parenthesized![content in input];
            let tuple = content.parse_terminated(BsnAstPatch::parse, Token![,])?;
            Ok(BsnAstPatch::Tuple(tuple))
        } else if input.peek(Brace) {
            // Expression
            let content;
            braced![content in input];
            let expr = content.parse::<Expr>()?;
            Ok(BsnAstPatch::Expr(expr))
        } else {
            let path = input.parse::<Path>()?;

            let fields = if input.peek(Paren) {
                // Tuple struct
                let content;
                parenthesized![content in input];
                content
                    .parse_terminated(BsnAstProp::parse, Token![,])?
                    .iter()
                    .enumerate()
                    .map(|(i, prop)| (Member::from(i), prop.clone()))
                    .collect()
            } else if input.peek(Brace) {
                // Struct (braced)
                let content;
                braced![content in input];
                content
                    .parse_terminated(
                        |input| {
                            let member: Member = input.parse()?;
                            let _colon_token: Token![:] = input.parse()?;
                            let prop: BsnAstProp = input.parse()?;
                            Ok((member, prop))
                        },
                        Token![,],
                    )?
                    .iter()
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };

            Ok(BsnAstPatch::Patch(path, fields))
        }
    }
}

/// AST for a BSN property.
#[derive(Clone)]
pub enum BsnAstProp {
    /// An expression not prefixed with `@`
    Value(Expr),
    /// An expression prefixed with `@`
    Props(Expr),
}

impl From<BsnAstProp> for Expr {
    fn from(val: BsnAstProp) -> Self {
        match val {
            BsnAstProp::Value(expr) | BsnAstProp::Props(expr) => expr,
        }
    }
}

impl<'a> From<&'a BsnAstProp> for &'a Expr {
    fn from(val: &'a BsnAstProp) -> Self {
        match val {
            BsnAstProp::Value(expr) | BsnAstProp::Props(expr) => expr,
        }
    }
}

impl Parse for BsnAstProp {
    fn parse(input: ParseStream) -> Result<BsnAstProp> {
        let is_prop = input.parse::<Token![@]>().is_ok();
        let expr = input.parse::<Expr>()?;
        match is_prop {
            true => Ok(BsnAstProp::Props(expr)),
            false => Ok(BsnAstProp::Value(expr)),
        }
    }
}

/// An inherited patch (a type path or ident) with optional `,`-separated parameters surrounded by `()`.
#[derive(Clone)]
pub struct BsnAstInherit(pub Path, pub Punctuated<Expr, Token![,]>);

impl Parse for BsnAstInherit {
    fn parse(input: ParseStream) -> Result<BsnAstInherit> {
        let path = input.parse::<Path>()?;

        // Optional params
        let params = if input.peek(Paren) {
            let content;
            parenthesized![content in input];
            content.parse_terminated(Expr::parse, Token![,])?
        } else {
            Punctuated::new()
        };
        Ok(BsnAstInherit(path, params))
    }
}
