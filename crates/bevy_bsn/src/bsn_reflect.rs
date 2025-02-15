use core::{any::TypeId, str::FromStr};

use bevy::reflect::{
    DynamicEnum, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant, NamedField,
    PartialReflect, ReflectKind, StructInfo, StructVariantInfo, TypeInfo, TypeRegistration,
    TypeRegistry,
};
use thiserror::Error;

use crate::{
    Bsn, BsnComponent, BsnEntity, BsnProp, BsnProps, BsnValue, DynamicScene, ReflectConstruct,
};

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
    /// Missing [`ReflectConstruct`]
    #[error("No registered `ReflectConstruct` for type: {0}")]
    MissingReflectConstruct(String),
}

/// A result from reflecting [`Bsn`]
pub type ReflectResult<T> = Result<T, ReflectError>;

/// Wrapper borrowing a [`Bsn`] and a [`TypeRegistry`] to provide type-aware operations.
///
/// Allows doing things like:
///  - Converting a [`Bsn`] asset to a spawnable [`DynamicScene`].
///  - TODO: Querying/retrieving type-aware information from a [`Bsn`] asset.
///  - TODO: Editing a [`Bsn`] asset in a type-aware way.
pub struct BsnReflector<'a> {
    bsn: &'a Bsn,
    registry: &'a TypeRegistry,
}

/// A reflected instance of a type containing type id and the (maybe dynamic) instance itself.
pub struct ReflectedValue {
    /// The type id of the type that the `instance` represents.
    pub type_id: TypeId,
    /// The instance of the type. Note that this may not be a concrete instance,
    /// but a dynamic one.
    pub instance: Box<dyn PartialReflect>,
}

impl ReflectedValue {
    fn new(type_id: TypeId, instance: Box<dyn PartialReflect>) -> Self {
        Self { type_id, instance }
    }
}

/// A reflected component patch containing the type id of the component and the props to be applied.
pub struct ReflectedComponentPatch {
    /// The type id of the component.
    pub type_id: TypeId,
    /// The patch to be applied to the component props.
    pub props: ReflectedValue,
}

impl<'a> BsnReflector<'a> {
    /// Create a new reflector for the given [`Bsn`] asset and [`TypeRegistry`].
    pub fn new(bsn: &'a Bsn, registry: &'a TypeRegistry) -> Self {
        Self { bsn, registry }
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
                    return Err(ReflectError::MissingReflectConstruct(path.into()));
                };

                let Some(props_type) = self.registry.get(reflect_construct.props_type) else {
                    return Err(ReflectError::UnknownType(format!("props for {}", path)));
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
            BsnComponent::BracedExpr(expr) => Err(ReflectError::ExpressionNotSupported(format!(
                "{{{}}}",
                expr
            ))),
        }
    }

    fn reflect_prop_value(
        &self,
        prop: &BsnProp,
        ty: &TypeInfo,
    ) -> ReflectResult<Box<dyn PartialReflect>> {
        // This is fine : )
        if ty
            .type_path()
            .starts_with("bevy_bsn::construct::ConstructProp<")
        {
            let generic = ty.generics().get_named("T").unwrap();
            let generic_ty = self.registry.get(generic.type_id()).unwrap();

            let Some(reflect_construct) = generic_ty.data::<ReflectConstruct>() else {
                return Err(ReflectError::MissingReflectConstruct(
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
            _ => Err(ReflectError::UnexpectedType(
                format!("{:?}", value),
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
                format!("{:?}", items),
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

                    //let val = self.reflect_value(value, field.type_info().unwrap())?;
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
