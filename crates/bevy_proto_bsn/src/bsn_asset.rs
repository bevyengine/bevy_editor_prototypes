use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
};
use thiserror::Error;

use bevy_proto_bsn_ast::{
    quote::ToTokens,
    syn::{Expr, ExprCall, ExprLit, ExprStruct, Lit, Member},
    *,
};

pub(crate) fn bsn_asset_plugin(app: &mut App) {
    app.init_asset::<Bsn>();
    app.init_asset_loader::<BsnLoader>();
}

/// Asset loader for loading `.bsn`-files as [`Bsn`]s.
#[derive(Default)]
pub struct BsnLoader;

/// Error type for [`BsnLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BsnLoaderError {
    /// An [IO](std::io) Error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// A syntax error
    #[error("Syntax error: {0}")]
    SyntaxError(String),
}

impl AssetLoader for BsnLoader {
    type Asset = Bsn;
    type Settings = ();
    type Error = BsnLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut content = String::new();
        reader.read_to_string(&mut content).await?;

        let ast = syn::parse_str::<BsnAstEntity>(&content).map_err(|e| {
            let start = e.span().start();
            BsnLoaderError::SyntaxError(format!("{} at {}:{}", e, start.line, start.column))
        })?;

        let bsn = Bsn::from(&ast);

        Ok(bsn)
    }

    fn extensions(&self) -> &[&str] {
        &["proto_bsn", "bsn"]
    }
}

/// A non type-aware representation of a BSN-tree.
///
/// Can be loaded from a `.bsn` file using [`BsnLoader`].
#[derive(Default, Debug, Clone, Asset, TypePath, Hash)]
pub struct Bsn {
    /// The root entity of the BSN-tree.
    pub root: BsnEntity,
}

/// A non type-aware BSN entity.
#[derive(Default, Debug, Clone, Hash)]
pub struct BsnEntity {
    /// Components of the entity.
    pub components: Vec<BsnComponent>,
    /// Child entities
    pub children: Vec<BsnEntity>,
    /// Optional key used for retaining.
    pub key: Option<BsnKey>,
}

/// A non type-aware representation of a BSN key.
#[derive(Debug, Clone, Hash)]
pub enum BsnKey {
    /// A static key: `key: ...`
    Static(String),
    /// A dynamic key: `{<expr>}: ...`
    Dynamic(String),
}

/// A non type-aware representation of a BSN component.
#[derive(Debug, Clone, Hash)]
pub enum BsnComponent {
    /// A component patch with a type-path for the component.
    ///
    /// Has optional struct-like `{ ... }` or tuple-like props `( ... )`.
    Patch(String, BsnProps),
    /// A braced `{ ... }` unknown expression.
    BracedExpr(String),
}

/// A non type-aware representation of BSN component patch props.
#[derive(Default, Debug, Clone, Hash)]
pub enum BsnProps {
    /// No props to patch.
    #[default]
    None,
    /// Struct-like props. Meaning the target is either a struct or an enum with named fields.
    StructLike(Vec<(String, BsnProp)>),
    /// Tuple-like props. Meaning the target is a tuple struct, tuple enum, or a function/method call.
    TupleLike(Vec<BsnProp>),
}

/// A non type-aware representation of a construct prop for a BSN component patch field.
#[derive(Debug, Clone, Hash)]
pub enum BsnProp {
    /// A value with no leading `@`.
    Value(BsnValue),
    /// A value with a leading `@`, indicating it needs construction.
    Props(BsnValue),
}

impl BsnProp {
    /// Returns the value of the prop.
    pub fn value(&self) -> &BsnValue {
        match self {
            BsnProp::Props(value) | BsnProp::Value(value) => value,
        }
    }

    /// Returns the variant name of the prop.
    pub fn variant_name(&self) -> &'static str {
        match self {
            BsnProp::Value(_) => "Value",
            BsnProp::Props(_) => "Props",
        }
    }
}

/// A value in a BSN tree.
#[derive(Debug, Clone, Hash)]
pub enum BsnValue {
    /// A literal bool
    Bool(bool),
    /// A literal number
    Number(String),
    /// A literal string
    String(String),
    /// A literal character
    Char(char),
    /// A path or ident. Could be a unit struct, unit enum variant, etc.
    Path(String),
    /// A struct or enum with named fields.
    StructLike(String, Vec<(String, BsnValue)>),
    /// A tuple struct, tuple enum, or function/method call.
    Call(String, Vec<BsnValue>),
    /// A tuple of values.
    Tuple(Vec<BsnValue>),
    /// An unknown expression.
    UnknownExpr(String),
}

impl From<&BsnAstEntity> for Bsn {
    fn from(ast: &BsnAstEntity) -> Self {
        Bsn {
            root: BsnEntity::from(ast),
        }
    }
}

impl From<&BsnAstEntity> for BsnEntity {
    fn from(ast: &BsnAstEntity) -> Self {
        BsnEntity {
            components: BsnComponent::vec_from_ast_patch(&ast.patch),
            children: ast
                .children
                .iter()
                .filter_map(|c| match c {
                    BsnAstChild::Entity(entity) => Some(BsnEntity::from(entity)),
                    _ => None,
                })
                .collect(),
            key: ast.key.as_ref().map(BsnKey::from),
        }
    }
}

impl From<&BsnAstKey> for BsnKey {
    fn from(key: &BsnAstKey) -> Self {
        match key {
            BsnAstKey::Static(key) => BsnKey::Static(key.clone()),
            BsnAstKey::Dynamic(key) => BsnKey::Dynamic(key.to_token_stream().to_string()),
        }
    }
}

impl BsnComponent {
    /// Converts and flattens a [`BsnAstPatch`] patch to a [`Vec<BsnComponent>`].
    fn vec_from_ast_patch(patch: &BsnAstPatch) -> Vec<BsnComponent> {
        let mut components = Vec::new();
        Self::convert_components(&mut components, patch);
        components
    }

    fn convert_components(components: &mut Vec<BsnComponent>, patch: &BsnAstPatch) {
        match patch {
            BsnAstPatch::Tuple(patches) => {
                for patch in patches {
                    Self::convert_components(components, patch);
                }
            }
            BsnAstPatch::Patch(path, fields) => {
                let path = path.to_compact_string();

                let props = if fields.is_empty() {
                    BsnProps::None
                } else if matches!(fields[0].0, Member::Unnamed(_)) {
                    BsnProps::TupleLike(fields.iter().map(|(_, prop)| prop.into()).collect())
                } else {
                    BsnProps::StructLike(
                        fields
                            .iter()
                            .map(|(name, prop)| match name {
                                Member::Named(name) => (name.to_string(), prop.into()),
                                _ => unreachable!(),
                            })
                            .collect(),
                    )
                };

                components.push(BsnComponent::Patch(path, props));
            }
            BsnAstPatch::Expr(expr) => {
                components.push(BsnComponent::BracedExpr(expr.to_token_stream().to_string()));
            }
        }
    }
}

impl From<&BsnAstProp> for BsnProp {
    fn from(prop: &BsnAstProp) -> Self {
        match prop {
            BsnAstProp::Value(value) => BsnProp::Value(value.into()),
            BsnAstProp::Props(value) => BsnProp::Props(value.into()),
        }
    }
}

impl From<&Expr> for BsnValue {
    fn from(expr: &Expr) -> Self {
        match &expr {
            Expr::Lit(lit) => lit.into(),
            Expr::Tuple(expr_tuple) => {
                let mut tuple = Vec::new();
                for expr in &expr_tuple.elems {
                    tuple.push(expr.into());
                }
                BsnValue::Tuple(tuple)
            }
            Expr::Path(path) => BsnValue::Path(path.to_compact_string()),
            Expr::Struct(strct) => strct.into(),
            Expr::Call(call) => call.into(),
            Expr::Paren(paren) => paren.expr.as_ref().into(),
            expr => BsnValue::UnknownExpr(expr.to_token_stream().to_string()),
        }
    }
}

impl From<&ExprLit> for BsnValue {
    fn from(lit: &ExprLit) -> Self {
        match &lit.lit {
            Lit::Bool(b) => BsnValue::Bool(b.value),
            Lit::Int(i) => BsnValue::Number(i.to_token_stream().to_string()),
            Lit::Float(f) => BsnValue::Number(f.to_token_stream().to_string()),
            Lit::Str(s) => BsnValue::String(s.value()),
            Lit::Char(c) => BsnValue::Char(c.value()),
            _ => BsnValue::UnknownExpr(lit.to_token_stream().to_string()),
        }
    }
}

impl From<&ExprStruct> for BsnValue {
    fn from(strct: &ExprStruct) -> Self {
        let path = strct.path.to_compact_string();
        let fields = strct
            .fields
            .iter()
            .map(|field| match &field.member {
                Member::Named(name) => (name.to_string(), BsnValue::from(&field.expr)),
                _ => unreachable!(),
            })
            .collect();
        BsnValue::StructLike(path, fields)
    }
}

impl From<&ExprCall> for BsnValue {
    fn from(call: &ExprCall) -> Self {
        let path = match call.func.as_ref() {
            Expr::Path(path) => path.to_compact_string(),
            _ => {
                return BsnValue::UnknownExpr(call.to_token_stream().to_string());
            }
        };

        let fields_or_params = call.args.iter().map(BsnValue::from).collect();

        BsnValue::Call(path, fields_or_params)
    }
}

trait ToTokensExt {
    fn to_compact_string(&self) -> String;
}

impl<T: ToTokens> ToTokensExt for T {
    fn to_compact_string(&self) -> String {
        self.to_token_stream().to_string().replace(" ", "")
    }
}

/// Trait for types that can be converted to `.bsn`-strings. Powers saving of BSN assets.
pub trait ToBsnString {
    /// Convert to a string in `.bsn` format.
    fn to_bsn_string(&self) -> String;
}

trait Joined {
    fn joined(&self, separator: &str) -> String;
}

impl<T> Joined for Vec<T>
where
    T: ToBsnString,
{
    fn joined(&self, separator: &str) -> String {
        self.iter()
            .map(ToBsnString::to_bsn_string)
            .collect::<Vec<_>>()
            .join(separator)
    }
}

impl<T> ToBsnString for (String, T)
where
    T: ToBsnString,
{
    fn to_bsn_string(&self) -> String {
        format!("{}: {}", self.0, self.1.to_bsn_string())
    }
}

impl ToBsnString for Bsn {
    fn to_bsn_string(&self) -> String {
        self.root.to_bsn_string()
    }
}

impl ToBsnString for BsnEntity {
    fn to_bsn_string(&self) -> String {
        let components = match self.components.len() {
            0 => "()".to_string(),
            1 => self.components[0].to_bsn_string(),
            _ => format!("({})", self.components.joined(", ")),
        };
        let children = if self.children.is_empty() {
            "".to_string()
        } else {
            format!(" [{}]", self.children.joined(", "))
        };
        format!("{}{}", components, children)
    }
}

impl ToBsnString for BsnKey {
    fn to_bsn_string(&self) -> String {
        match self {
            BsnKey::Static(key) => format!("{}: ", key),
            BsnKey::Dynamic(key) => format!("{{{}}}: ", key),
        }
    }
}

impl ToBsnString for BsnComponent {
    fn to_bsn_string(&self) -> String {
        match self {
            BsnComponent::Patch(path, props) => match props {
                BsnProps::None => path.clone(),
                BsnProps::StructLike(fields) => format!("{} {{ {} }}", path, fields.joined(", ")),
                BsnProps::TupleLike(fields) => format!("{}({})", path, fields.joined(", ")),
            },
            BsnComponent::BracedExpr(expr) => expr.clone(),
        }
    }
}

impl ToBsnString for BsnProp {
    fn to_bsn_string(&self) -> String {
        match self {
            BsnProp::Value(value) => value.to_bsn_string(),
            BsnProp::Props(value) => format!("@{}", value.to_bsn_string()),
        }
    }
}

impl ToBsnString for BsnValue {
    fn to_bsn_string(&self) -> String {
        match self {
            BsnValue::Bool(b) => b.to_string(),
            BsnValue::Number(n) => n.clone(),
            BsnValue::String(s) => format!("\"{}\"", s),
            BsnValue::Char(c) => format!("'{}'", c),
            BsnValue::Path(p) => p.clone(),
            BsnValue::StructLike(path, fields) => {
                format!("{} {{ {} }}", path, fields.joined(", "))
            }
            BsnValue::Call(path, args) => {
                format!("{}({})", path, args.joined(", "))
            }
            BsnValue::Tuple(fields) => format!("({})", fields.joined(", ")),
            BsnValue::UnknownExpr(expr) => expr.clone(),
        }
    }
}
