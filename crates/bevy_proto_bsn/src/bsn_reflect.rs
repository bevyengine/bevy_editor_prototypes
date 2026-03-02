use core::{any::TypeId, cell::RefCell, hash::BuildHasher, ops::Deref, str::FromStr};

use bevy::{
    app::App,
    asset::{Asset, AssetLoader, AssetPath, AssetServer, Handle, LoadContext, io::Reader},
    ecs::{
        reflect::AppTypeRegistry,
        world::{FromWorld, World},
    },
    platform::hash::FixedState,
    reflect::{
        DynamicEnum, DynamicList, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant,
        FromType, NamedField, PartialReflect, Reflect, ReflectKind, StructInfo, StructVariantInfo,
        TypeInfo, TypePath, TypeRegistration, TypeRegistry, TypeRegistryArc,
    },
};
use thiserror::Error;

use crate::{
    Bsn, BsnComponent, BsnEntity, BsnKey, BsnLoader, BsnLoaderError, BsnProp, BsnProps, BsnValue,
    DynamicPatch, DynamicScene, ReflectConstruct,
};

pub(crate) fn bsn_reflect_plugin(app: &mut App) {
    app.init_asset::<ReflectedBsn>();
    app.init_asset_loader::<ReflectedBsnLoader>();

    /// Register `ReflectHandle`s for some upstream assets to ensure the hacky asset loading workaround works.
    use bevy::{asset::Handle, prelude::*, sprite_render::Wireframe2dMaterial};
    app.register_type_data::<Handle<Scene>, ReflectHandleLoad>();
    app.register_type_data::<Handle<Bsn>, ReflectHandleLoad>();
    app.register_type_data::<Handle<Font>, ReflectHandleLoad>();
    app.register_type_data::<Handle<Mesh>, ReflectHandleLoad>();
    app.register_type_data::<Handle<Gltf>, ReflectHandleLoad>();
    app.register_type_data::<Handle<Image>, ReflectHandleLoad>();
    app.register_type_data::<Handle<AnimationGraph>, ReflectHandleLoad>();
    app.register_type_data::<Handle<AudioSource>, ReflectHandleLoad>();
    app.register_type_data::<Handle<Shader>, ReflectHandleLoad>();
    app.register_type_data::<Handle<DynamicScene>, ReflectHandleLoad>();
    app.register_type_data::<Handle<ColorMaterial>, ReflectHandleLoad>();
    app.register_type::<Handle<Wireframe2dMaterial>>();
    app.register_type_data::<Handle<Wireframe2dMaterial>, ReflectHandleLoad>();
}

/// A reflected BSN scene. Implements [`DynamicPatch`] so it can be applied to/or spawned as a [`DynamicScene`].
#[derive(Debug, Clone, TypePath, Asset)]
pub struct ReflectedBsn {
    /// Hash of the BSN ast that produced this reflected BSN scene
    pub hash: u64,
    /// Component patches to be applied to the scene
    pub component_patches: Vec<ReflectedComponentPatch>,
    /// Children of the scene
    pub children: Vec<ReflectedBsn>,
    /// Static key for the scene
    pub key: Option<String>,
}

impl ReflectedBsn {
    /// Try to reflect a [`BsnEntity`] to a [`ReflectedBsn`].
    pub fn try_from_bsn(bsn: &BsnEntity, reflector: &BsnReflector) -> ReflectResult<Self> {
        let key = match &bsn.key {
            Some(BsnKey::Static(key)) => Some(key.clone()),
            Some(BsnKey::Dynamic(key)) => {
                return Err(ReflectError::DynamicKeyNotSupported(key.clone()));
            }
            None => None,
        };

        let component_patches = bsn
            .components
            .iter()
            .map(|component| reflector.reflect_component_patch(component))
            .collect::<Result<_, _>>()?;

        let children = bsn
            .children
            .iter()
            .map(|child| Self::try_from_bsn(child, reflector))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            hash: FixedState::default().hash_one(bsn),
            component_patches,
            children,
            key,
        })
    }
}

impl DynamicPatch for ReflectedBsn {
    fn dynamic_patch(self, scene: &mut DynamicScene) {
        for patch in self.component_patches {
            scene.patch_reflected(patch.type_id, move |props: &mut dyn PartialReflect| {
                props.apply(patch.props.instance.as_ref());
            });
        }

        self.children.dynamic_patch_as_children(scene);
    }
}

/// Asset loader for loading `.bsn`-files as [`ReflectedBsn`].
#[derive(Debug)]
pub struct ReflectedBsnLoader {
    type_registry: TypeRegistryArc,
}

impl FromWorld for ReflectedBsnLoader {
    fn from_world(world: &mut World) -> Self {
        let type_registry = world.resource::<AppTypeRegistry>();
        ReflectedBsnLoader {
            type_registry: type_registry.0.clone(),
        }
    }
}

/// Error type for [`ReflectedBsnLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ReflectedBsnLoaderError {
    /// A [`BsnLoaderError`]
    #[error("Load error: {0}")]
    LoadError(#[from] BsnLoaderError),
    /// A reflection error
    #[error("Reflection error: {0}")]
    ReflectError(#[from] ReflectError),
}

impl AssetLoader for ReflectedBsnLoader {
    type Asset = ReflectedBsn;
    type Settings = ();
    type Error = ReflectedBsnLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let bsn = BsnLoader.load(reader, &(), load_context).await?;
        let registry = self.type_registry.read();
        let reflector = BsnReflector::new(&bsn, registry.deref()).with_asset_load(load_context);
        let reflected_bsn = ReflectedBsn::try_from_bsn(&bsn.root, &reflector)?;
        Ok(reflected_bsn)
    }

    fn extensions(&self) -> &[&str] {
        &["proto_bsn", "bsn"]
    }
}

/// A reflection error returned when reflecting [`Bsn`].
#[derive(Error, Debug, Hash)]
pub enum ReflectError {
    /// Unknown type
    #[error("Can't resolve type `{0}`, is it registered?")]
    UnknownType(String),
    /// Unknown enum variant
    #[error("Unknown enum variant `{0}` for `{1}`")]
    UnknownEnumVariant(String, String),
    /// Unknown field
    #[error("Unknown field `{0}` in `{1}`")]
    UnknownField(String, String),
    /// Unexpected type
    #[error("Unexpected type for '{0}', expected `{1}`")]
    UnexpectedType(String, String),
    /// Expression not supported
    #[error("Expression not supported: {0}")]
    ExpressionNotSupported(String),
    /// Not reflectable
    #[error("Not reflectable")]
    NotReflectable,
    /// Missing registered type data
    #[error("No registered `{0}` for type: {1}")]
    MissingTypeData(String, String),
    /// Dynamic key not supported
    #[error("Dynamic keys are not supported in reflected BSN: {0}")]
    DynamicKeyNotSupported(String),
}

/// A result from reflecting [`Bsn`]
pub type ReflectResult<T> = Result<T, ReflectError>;

/// Wrapper borrowing a [`Bsn`] and a [`TypeRegistry`] to provide type-aware operations.
///
/// Allows doing things like:
///  - Converting a [`Bsn`] asset to a spawnable [`DynamicScene`].
///  - TODO: Querying/retrieving type-aware information from a [`Bsn`] asset.
///  - TODO: Editing a [`Bsn`] asset in a type-aware way.
pub struct BsnReflector<'a, 'b> {
    bsn: &'a Bsn,
    registry: &'a TypeRegistry,
    asset_loader: Option<BsnReflectorAssetLoader<'a, 'b>>,
}

/// A reflected instance of a type containing type id and the (maybe dynamic) instance itself.
#[derive(Debug, TypePath)]
pub struct ReflectedValue {
    /// The type id of the type that the `instance` represents.
    pub type_id: TypeId,
    /// The instance of the type. Note that this may not be a concrete instance,
    /// but a dynamic one.
    pub instance: Box<dyn PartialReflect>,
}

impl Clone for ReflectedValue {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            instance: self.instance.as_ref().to_dynamic(),
        }
    }
}

impl ReflectedValue {
    fn new(type_id: TypeId, instance: Box<dyn PartialReflect>) -> Self {
        Self { type_id, instance }
    }
}

/// A reflected component patch containing the type id of the component and the props to be applied.
#[derive(Debug, Clone, TypePath)]
pub struct ReflectedComponentPatch {
    /// The type id of the component.
    pub type_id: TypeId,
    /// The patch to be applied to the component props.
    pub props: ReflectedValue,
}

impl<'a, 'b> BsnReflector<'a, 'b> {
    /// Create a new reflector for the given [`Bsn`] asset and [`TypeRegistry`].
    pub fn new(bsn: &'a Bsn, registry: &'a TypeRegistry) -> Self {
        Self {
            bsn,
            registry,
            asset_loader: None,
        }
    }

    /// A hacky workaround to allow loading assets using @-syntax during BSN reflection.
    ///
    /// This exists because a proper [`crate::Construct`] implementation for [`Handle`] is not possible without upstream changes.
    ///
    /// Takes either an [`AssetServer`] or a mutable reference to a [`LoadContext`] to load assets.
    ///
    /// NOTE: Any assets loaded during BSN reflection needs to have a [`ReflectHandleLoad`] registered for the [`Handle`] type.
    pub fn with_asset_load(
        mut self,
        asset_loader: impl Into<BsnReflectorAssetLoader<'a, 'b>> + 'a,
    ) -> Self {
        self.asset_loader = Some(asset_loader.into());
        self
    }

    fn try_resolve_type(&self, type_path: &str) -> ReflectResult<&TypeRegistration> {
        // TODO: `use`-declarations instead of short_path
        // TODO: FunctionRegistry
        match self.registry.get_with_short_type_path(type_path) {
            Some(t) => Ok(t),
            None => {
                let last_segment_index = type_path.rfind("::");
                if let Some(last_segment_index) = last_segment_index {
                    // Try without last segment, in case of enum variant
                    match self
                        .registry
                        .get_with_short_type_path(&type_path[..last_segment_index])
                    {
                        Some(t) => Ok(t),
                        _ => Err(ReflectError::UnknownType(type_path.into())),
                    }
                } else {
                    Err(ReflectError::UnknownType(type_path.into()))
                }
            }
        }
    }

    fn try_resolve_type_info(&self, type_path: &str) -> ReflectResult<&TypeInfo> {
        Ok(self.try_resolve_type(type_path)?.type_info())
    }

    /// Reflect a [`Bsn`] asset to a [`DynamicScene`].
    pub fn reflect_dynamic_scene(&self) -> ReflectResult<DynamicScene> {
        self.reflect_dynamic_scene_internal(&self.bsn.root)
    }

    fn reflect_dynamic_scene_internal(
        &self,
        bsn_entity: &BsnEntity,
    ) -> ReflectResult<DynamicScene> {
        let mut dynamic_scene = DynamicScene::default();

        // Add component patches
        for component in bsn_entity.components.iter() {
            let patch_data = self.reflect_component_patch(component)?;
            dynamic_scene.patch_reflected(
                patch_data.type_id,
                move |props: &mut dyn PartialReflect| {
                    props.apply(patch_data.props.instance.as_ref());
                },
            );
        }

        // Add children
        for child in bsn_entity.children.iter() {
            let child_dynamic_scene = self.reflect_dynamic_scene_internal(child)?;
            dynamic_scene.push_child(child_dynamic_scene);
        }

        Ok(dynamic_scene)
    }

    fn reflect_component_patch(
        &self,
        component: &BsnComponent,
    ) -> ReflectResult<ReflectedComponentPatch> {
        match component {
            BsnComponent::Patch(path, props) => {
                let component_type = self.try_resolve_type(path)?;
                let Some(reflect_construct) = component_type.data::<ReflectConstruct>() else {
                    return Err(ReflectError::MissingTypeData(
                        "ReflectConstruct".into(),
                        path.into(),
                    ));
                };

                let Some(props_type) = self.registry.get(reflect_construct.props_type) else {
                    return Err(ReflectError::UnknownType(format!("props for {path}")));
                };

                let props = match props {
                    BsnProps::None => self.reflect_path(path, Some(props_type.type_info()))?,
                    BsnProps::StructLike(props) => self.reflect_struct_like(
                        path,
                        props,
                        |prop, type_info| self.reflect_prop_value(prop, type_info),
                        props_type.type_info(),
                    )?,
                    BsnProps::TupleLike(props) => self.reflect_call_like(
                        path,
                        props,
                        |prop, type_info| self.reflect_prop_value(prop, type_info),
                        props_type.type_info(),
                    )?,
                };

                Ok(ReflectedComponentPatch {
                    type_id: component_type.type_id(),
                    props,
                })
            }
            BsnComponent::BracedExpr(expr) => {
                Err(ReflectError::ExpressionNotSupported(format!("{{{expr}}}",)))
            }
        }
    }

    fn reflect_prop_value(
        &self,
        prop: &BsnProp,
        ty: &TypeInfo,
    ) -> ReflectResult<Box<dyn PartialReflect>> {
        // HACK: Allows constructing Handles from asset paths in BSN assets by triggering loads during reflection.
        // This should be removed when we have an upstream Construct implementation for Handle.
        if ty.type_path().starts_with("bevy_asset::handle::Handle<")
            && self.asset_loader.is_some()
            && let BsnProp::Props(BsnValue::String(asset_path)) = prop
        {
            let Some(reflect_handle_load) = self
                .registry
                .get_type_data::<ReflectHandleLoad>(ty.type_id())
            else {
                return Err(ReflectError::MissingTypeData(
                    "ReflectHandleLoad".into(),
                    ty.type_path().into(),
                ));
            };
            let handle = reflect_handle_load.load(asset_path, self.asset_loader.as_ref().unwrap());
            return Ok(handle.into_partial_reflect());
        }

        // This is fine : )
        if ty
            .type_path()
            .starts_with("bevy_proto_bsn::construct::ConstructProp<")
        {
            let generic = ty.generics().get_named("T").unwrap();
            let generic_ty = self.registry.get(generic.type_id()).unwrap();

            let Some(reflect_construct) = generic_ty.data::<ReflectConstruct>() else {
                return Err(ReflectError::MissingTypeData(
                    "ReflectConstruct".into(),
                    generic_ty.type_info().type_path().into(),
                ));
            };

            let Some(props_type) = self.registry.get(reflect_construct.props_type) else {
                return Err(ReflectError::UnknownType(format!(
                    "props for {}",
                    generic_ty.type_info().type_path()
                )));
            };

            let val = self.reflect_value(prop.value(), props_type.type_info())?;
            let mut dynamic_tuple = DynamicTuple::default();
            dynamic_tuple.insert_boxed(val.instance.into_partial_reflect());
            Ok(Box::new(DynamicEnum::new(
                prop.variant_name(),
                DynamicVariant::Tuple(dynamic_tuple),
            )))
        } else {
            Ok(self.reflect_value(prop.value(), ty)?.instance)
        }
    }

    fn reflect_value(&self, value: &BsnValue, ty: &TypeInfo) -> ReflectResult<ReflectedValue> {
        let type_id = ty.type_id();
        match value {
            BsnValue::UnknownExpr(expr) => Err(ReflectError::ExpressionNotSupported(expr.into())),
            BsnValue::Bool(b) if type_id == TypeId::of::<bool>() => {
                Ok(ReflectedValue::new(type_id, Box::new(*b)))
            }
            BsnValue::String(s) if type_id == TypeId::of::<String>() => {
                Ok(ReflectedValue::new(type_id, Box::new(s.clone())))
            }
            BsnValue::Char(c) if type_id == TypeId::of::<char>() => {
                Ok(ReflectedValue::new(type_id, Box::new(*c)))
            }
            BsnValue::Number(number) => self.reflect_number(number, ty),
            BsnValue::Path(path) => self.reflect_path(path, Some(ty)),
            BsnValue::StructLike(path, fields) => self.reflect_struct_like(
                path,
                fields,
                |value, ty| Ok(self.reflect_value(value, ty)?.instance),
                ty,
            ),
            BsnValue::Call(path, args) => self.reflect_call_like(
                path,
                args.iter().collect::<Vec<_>>().as_ref(),
                |value, ty| Ok(self.reflect_value(value, ty)?.instance),
                ty,
            ),
            BsnValue::Tuple(items) => self.reflect_tuple(items, ty),
            BsnValue::List(items) => self.reflect_list(items, ty),
            _ => Err(ReflectError::UnexpectedType(
                format!("{value:?}"),
                ty.type_path().into(),
            )),
        }
    }

    fn reflect_number(&self, number: &str, ty: &TypeInfo) -> ReflectResult<ReflectedValue> {
        fn parse_number<T: FromStr + PartialReflect>(
            s: &str,
            t: &TypeInfo,
        ) -> ReflectResult<ReflectedValue> {
            let num = s
                .parse::<T>()
                .map_err(|_| ReflectError::UnexpectedType(s.into(), t.type_path().into()))?;
            Ok(ReflectedValue::new(TypeId::of::<T>(), Box::new(num)))
        }

        let type_id = ty.type_id();
        if type_id == TypeId::of::<u8>() {
            parse_number::<u8>(number, ty)
        } else if type_id == TypeId::of::<u16>() {
            parse_number::<u16>(number, ty)
        } else if type_id == TypeId::of::<u32>() {
            parse_number::<u32>(number, ty)
        } else if type_id == TypeId::of::<u64>() {
            parse_number::<u64>(number, ty)
        } else if type_id == TypeId::of::<u128>() {
            parse_number::<u128>(number, ty)
        } else if type_id == TypeId::of::<usize>() {
            parse_number::<usize>(number, ty)
        } else if type_id == TypeId::of::<i8>() {
            parse_number::<i8>(number, ty)
        } else if type_id == TypeId::of::<i16>() {
            parse_number::<i16>(number, ty)
        } else if type_id == TypeId::of::<i32>() {
            parse_number::<i32>(number, ty)
        } else if type_id == TypeId::of::<i64>() {
            parse_number::<i64>(number, ty)
        } else if type_id == TypeId::of::<i128>() {
            parse_number::<i128>(number, ty)
        } else if type_id == TypeId::of::<f32>() {
            parse_number::<f32>(number, ty)
        } else if type_id == TypeId::of::<f64>() {
            parse_number::<f64>(number, ty)
        } else {
            Err(ReflectError::UnexpectedType(
                number.into(),
                ty.type_path().into(),
            ))
        }
    }

    fn reflect_tuple(&self, items: &[BsnValue], ty: &TypeInfo) -> ReflectResult<ReflectedValue> {
        let tuple_info = ty.as_tuple().unwrap();
        if tuple_info.field_len() != items.len() {
            return Err(ReflectError::UnexpectedType(
                format!("{items:?}"),
                format!("Tuple with {} fields", tuple_info.field_len()),
            ));
        }

        let mut dynamic_tuple = DynamicTuple::default();
        for (i, item) in items.iter().enumerate() {
            let ty = tuple_info.field_at(i).unwrap().type_info().unwrap();
            dynamic_tuple.insert_boxed(self.reflect_value(item, ty)?.instance);
        }
        Ok(ReflectedValue::new(ty.type_id(), Box::new(dynamic_tuple)))
    }

    fn reflect_list(&self, items: &[BsnValue], ty: &TypeInfo) -> ReflectResult<ReflectedValue> {
        if let Ok(list_info) = ty.as_list() {
            let mut dynamic_list = DynamicList::default();
            let item_type_info = list_info.item_info().expect("Expected typed list");
            for item in items.iter() {
                dynamic_list.push_box(self.reflect_value(item, item_type_info)?.instance);
            }
            Ok(ReflectedValue::new(ty.type_id(), Box::new(dynamic_list)))
        } else {
            Err(ReflectError::UnexpectedType(
                format!("{items:?}"),
                format!("{ty:?}"),
            ))
        }
    }

    fn reflect_path(&self, path: &str, ty: Option<&TypeInfo>) -> ReflectResult<ReflectedValue> {
        let ty = match ty {
            Some(ty) => ty,
            None => self.try_resolve_type_info(path)?,
        };

        match ty.kind() {
            ReflectKind::Struct => {
                // Unit struct
                Ok(ReflectedValue::new(
                    ty.type_id(),
                    Box::new(DynamicStruct::default()),
                ))
            }
            ReflectKind::Enum => {
                // Enum (unit-like)
                let variant_name = path.split("::").last().unwrap();
                let reflect_enum = ty.as_enum().unwrap();
                if !reflect_enum.contains_variant(variant_name) {
                    return Err(ReflectError::UnknownEnumVariant(
                        variant_name.into(),
                        ty.type_path().into(),
                    ));
                }
                Ok(ReflectedValue::new(
                    ty.type_id(),
                    Box::new(DynamicEnum::new(variant_name, DynamicVariant::Unit)),
                ))
            }
            _ => Err(ReflectError::NotReflectable),
        }
    }

    fn reflect_struct_like<T, F>(
        &self,
        path: &str,
        fields: &[(String, T)],
        get_value: F,
        ty: &TypeInfo,
    ) -> ReflectResult<ReflectedValue>
    where
        F: Fn(&T, &TypeInfo) -> ReflectResult<Box<dyn PartialReflect>>,
    {
        trait StructInfoLike {
            fn field(&self, name: &str) -> Option<&NamedField>;
        }

        impl StructInfoLike for StructInfo {
            fn field(&self, name: &str) -> Option<&NamedField> {
                self.field(name)
            }
        }

        impl StructInfoLike for StructVariantInfo {
            fn field(&self, name: &str) -> Option<&NamedField> {
                self.field(name)
            }
        }

        fn reflect_fields<T, F>(
            ty: &TypeInfo,
            info: &impl StructInfoLike,
            fields: &[(String, T)],
            get_value: F,
        ) -> ReflectResult<DynamicStruct>
        where
            F: Fn(&T, &TypeInfo) -> ReflectResult<Box<dyn PartialReflect>>,
        {
            let mut dynamic_struct = DynamicStruct::default();

            for (name, value) in fields.iter() {
                let Some(field) = info.field(name) else {
                    return Err(ReflectError::UnknownField(
                        (*name).clone(),
                        ty.type_path().into(),
                    ));
                };

                dynamic_struct.insert_boxed(name, get_value(value, field.type_info().unwrap())?);
            }

            Ok(dynamic_struct)
        }

        let dynamic: Box<dyn PartialReflect> = match ty {
            TypeInfo::Struct(info) => Box::new(reflect_fields(ty, info, fields, get_value)?),
            TypeInfo::Enum(info) => {
                // Enum (struct-like)
                let variant_name = path.split("::").last().unwrap();
                let Some(struct_info) = info
                    .variant(variant_name)
                    .and_then(|v| v.as_struct_variant().ok())
                else {
                    return Err(ReflectError::UnknownEnumVariant(
                        variant_name.into(),
                        ty.type_path().into(),
                    ));
                };
                let dynamic_struct = reflect_fields(ty, struct_info, fields, get_value)?;
                Box::new(DynamicEnum::new(
                    variant_name,
                    DynamicVariant::Struct(dynamic_struct),
                ))
            }
            _ => {
                return Err(ReflectError::NotReflectable);
            }
        };

        Ok(ReflectedValue::new(ty.type_id(), dynamic))
    }

    fn reflect_call_like<T, F>(
        &self,
        path: &str,
        args: &[T],
        get_value: F,
        ty: &TypeInfo,
    ) -> ReflectResult<ReflectedValue>
    where
        F: Fn(&T, &TypeInfo) -> ReflectResult<Box<dyn PartialReflect>>,
    {
        match ty.kind() {
            ReflectKind::TupleStruct => {
                // Tuple struct
                let props_struct = ty.as_tuple_struct().unwrap();
                let mut dynamic_struct = DynamicTupleStruct::default();

                for (index, value) in args.iter().enumerate() {
                    let Some(field) = props_struct.field_at(index) else {
                        return Err(ReflectError::UnknownField(
                            index.to_string(),
                            ty.type_path().into(),
                        ));
                    };

                    dynamic_struct.insert_boxed(get_value(value, field.type_info().unwrap())?);
                }

                Ok(ReflectedValue::new(ty.type_id(), Box::new(dynamic_struct)))
            }
            ReflectKind::Enum => {
                // Enum (tuple-like)
                let reflect_enum = ty.as_enum().unwrap();
                let variant_name = path.split("::").last().unwrap();

                let Some(variant) = reflect_enum.variant(variant_name) else {
                    return Err(ReflectError::UnknownEnumVariant(
                        variant_name.into(),
                        ty.type_path().into(),
                    ));
                };
                let variant = variant.as_tuple_variant().unwrap();

                let mut dynamic_tuple = DynamicTuple::default();
                for (i, arg) in args.iter().enumerate() {
                    let field = variant.field_at(i).unwrap();
                    dynamic_tuple.insert_boxed(get_value(arg, field.type_info().unwrap())?);
                }

                Ok(ReflectedValue::new(
                    ty.type_id(),
                    Box::new(DynamicEnum::new(
                        variant_name,
                        DynamicVariant::Tuple(dynamic_tuple),
                    )),
                ))
            }
            _ => Err(ReflectError::NotReflectable),
        }
    }
}

/// Wraps either an [`AssetServer`] or a mutable reference to a [`LoadContext`] to allow loading assets during reflection.
pub enum BsnReflectorAssetLoader<'a, 'b> {
    /// Use an [`AssetServer`] to load assets.
    AssetServer(&'a AssetServer),
    /// Use a [`LoadContext`] to load assets.
    LoadContext(RefCell<&'a mut LoadContext<'b>>),
}

impl BsnReflectorAssetLoader<'_, '_> {
    fn load<'a, A: Asset>(&self, path: impl Into<AssetPath<'a>>) -> Handle<A> {
        let asset_path = path.into().into_owned();
        match self {
            BsnReflectorAssetLoader::AssetServer(asset_server) => {
                asset_server.load::<A>(asset_path)
            }
            BsnReflectorAssetLoader::LoadContext(load_context) => {
                load_context.borrow_mut().load::<A>(asset_path)
            }
        }
    }
}

impl<'a> From<&'a AssetServer> for BsnReflectorAssetLoader<'a, '_> {
    fn from(asset_server: &'a AssetServer) -> Self {
        BsnReflectorAssetLoader::AssetServer(asset_server)
    }
}

impl<'a, 'b> From<&'a mut LoadContext<'b>> for BsnReflectorAssetLoader<'a, 'b> {
    fn from(load_context: &'a mut LoadContext<'b>) -> Self {
        BsnReflectorAssetLoader::LoadContext(RefCell::new(load_context))
    }
}

/// Type data for handles to load assets during reflection, needed by the hacky asset loading workaround.
#[derive(Clone)]
pub struct ReflectHandleLoad {
    load: fn(&str, &BsnReflectorAssetLoader) -> Box<dyn Reflect>,
}

impl ReflectHandleLoad {
    /// Load a typed asset, returning the resulting handle.
    pub fn load(&self, path: &str, asset_loader: &BsnReflectorAssetLoader) -> Box<dyn Reflect> {
        (self.load)(path, asset_loader)
    }
}

impl<A: Asset> FromType<Handle<A>> for ReflectHandleLoad {
    fn from_type() -> Self {
        ReflectHandleLoad {
            load: |path, asset_loader| {
                let asset_path: AssetPath = path.to_string().into();
                Box::new(asset_loader.load::<A>(asset_path))
            },
        }
    }
}
