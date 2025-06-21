use facet_core::{Def, Facet, Field, Shape, StructKind, Type, UserType};
use facet_reflect::{Partial, Peek, ReflectError, VariantError};

mod array;
mod enum_;
mod map;
mod object;
mod pointer;
mod scalar;
mod set;

pub use object::Constructors;
use pointer::MarshalPointers;

struct MarshalState<'mem, 'scope, 'constructors, 'env> {
    // Cached null to avoid creating a huge number of locals.
    pub null: v8::Local<'scope, v8::Primitive>,

    /// Pointer tracking for shared pointers/references.
    pub pointers: MarshalPointers<'mem, 'scope>,

    /// Custom object constructors/prototypes.
    pub constructors: &'constructors mut object::Constructors<'scope, 'env>,
}

#[derive(Debug)]
pub enum Error<'shape> {
    Exception,
    Reflect(ReflectError<'shape>),
    Variant(VariantError),
    ClobberedTypeTag(&'shape Shape<'shape>),
}

impl std::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Exception => write!(f, "exception during serialization"),
            Error::Reflect(e) => write!(f, "reflection error: {}", e),
            Error::Variant(e) => write!(f, "variant error: {}", e),
            Error::ClobberedTypeTag(shape) => write!(
                f,
                "serializing this enum variant would clobber the type tag: {shape}"
            ),
        }
    }
}

impl std::error::Error for Error<'_> {}

impl<'shape> From<ReflectError<'shape>> for Error<'shape> {
    #[inline]
    fn from(e: ReflectError<'shape>) -> Self {
        Error::Reflect(e)
    }
}

impl From<VariantError> for Error<'_> {
    #[inline]
    fn from(e: VariantError) -> Self {
        Error::Variant(e)
    }
}

/// Convert any Rust value to a V8 JavaScript value.
pub fn to_v8<'facet, 'scope, T: Facet<'facet>>(
    scope: &mut v8::HandleScope<'scope>,
    value: &T,
) -> Result<v8::Local<'scope, v8::Value>, Error<'facet>> {
    to_v8_with_constructors(scope, value, &mut Constructors::default())
}

/// Convert any Rust value to a V8 JavaScript value, using custom constructors
/// for certain types.
pub fn to_v8_with_constructors<'facet, 'scope, 'env, T: Facet<'facet>>(
    scope: &mut v8::HandleScope<'scope>,
    value: &T,
    constructors: &mut Constructors<'scope, 'env>,
) -> Result<v8::Local<'scope, v8::Value>, Error<'facet>> {
    let mut state = MarshalState {
        null: v8::null(scope),
        pointers: MarshalPointers::default(),
        constructors,
    };
    let peek = Peek::new(value);
    marshal_value(peek, scope, &mut state, None)
}

/// Construct a Rust value from a V8 JavaScript value.
pub fn from_v8<'facet, 'scope, T: Facet<'facet>>(
    scope: &mut v8::HandleScope<'scope>,
    value: v8::Local<'scope, v8::Value>,
) -> Result<Box<T>, Error<'facet>> {
    let mut partial = Partial::alloc_shape(T::SHAPE)?;
    from_v8_partial(scope, value, &mut partial)?;
    let value = partial.build()?.materialize()?;
    Ok(value)
}

pub fn from_v8_partial<'scope, 'facet, 'shape>(
    scope: &mut v8::HandleScope<'scope>,
    value: v8::Local<'scope, v8::Value>,
    partial: &mut Partial<'facet, 'shape>,
) -> Result<(), Error<'facet>> {
    todo!()
}

/// Returns `true` if values with the given shape will be serialized as a JS
/// object, meaning references to them inside smart pointers can be cleverly
/// shared, such that object identities are preserved when
/// marshalling/unmarshalling.
fn will_marshal_as_object(shape: &Shape) -> bool {
    match shape.def {
        Def::Scalar(_) => false,
        Def::Map(_) | Def::Set(_) | Def::List(_) | Def::Array(_) | Def::Slice(_) => true,
        Def::Option(od) => will_marshal_as_object(od.t),
        Def::SmartPointer(spd) => spd.pointee().map(will_marshal_as_object).unwrap_or(false),
        _ => match shape.ty {
            Type::Primitive(_) => false,
            Type::Sequence(_) => true,
            Type::User(UserType::Enum(enum_type)) => enum_::will_serialize_as_object(enum_type),
            Type::User(UserType::Struct(_)) => true,
            Type::Pointer(_) => {
                // TODO: For now, only string pointers are serialized through
                // pointers, because of limitations and bugs.
                false
            }
            _ => false,
        },
    }
}

fn marshal_value<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    peek: Peek<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
    field: Option<&Field>,
) -> Result<v8::Local<'scope, v8::Value>, Error<'shape>> {
    let shape = peek.shape();

    if let (Def::Scalar(_), _) | (_, Type::Primitive(_)) = (shape.def, shape.ty) {
        return scalar::scalar_to_v8(peek, scope, state);
    }

    if let Ok(option) = peek.into_option() {
        match option.value() {
            Some(peek) => return marshal_value(peek, scope, state, field),
            None => return Ok(state.null.into()),
        }
    }

    if let Def::SmartPointer(_) = shape.def {
        return pointer::serialize_smart_pointer(
            peek.into_smart_pointer().unwrap(),
            scope,
            state,
            field,
        );
    }
    if let Type::Pointer(pointer_type) = shape.ty {
        return pointer::serialize_pointer(peek, pointer_type, scope, state);
    }
    if let Type::User(UserType::Enum(enum_type)) = shape.ty {
        if !enum_::will_serialize_as_object(enum_type) {
            return enum_::serialize_enum_unit(peek.into_enum()?, enum_type, scope);
        }
    }

    // At this point, it is guaranteed that the object will be serialized as a
    // JS object, so we hook into the constructors.
    let obj = object::create_object_for_shape(peek, scope, state, field)?;
    marshal_into_object(peek, scope, obj, state)?;
    Ok(obj.into())
}

fn marshal_into_object<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    peek: Peek<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<(), Error<'shape>> {
    let shape = peek.shape();
    debug_assert!(
        will_marshal_as_object(shape),
        "expected {shape} to serialize as an object"
    );

    match (shape.def, shape.ty) {
        (Def::Map(_), _) => map::serialize_map_into(peek.into_map()?, scope, object, state),
        (Def::Set(_), _) => set::serialize_set_into(peek, scope, object, state),
        (Def::List(_) | Def::Array(_) | Def::Slice(_), _) => {
            array::marshal_list_object(peek, scope, object, state)
        }
        (_, Type::User(UserType::Struct(struct_type))) if struct_type.kind == StructKind::Tuple => {
            array::marshal_tuple_object(peek.into_tuple()?, scope, object, state)
        }
        (_, Type::User(UserType::Enum(_))) => {
            enum_::serialize_enum_object_into(peek.into_enum()?, scope, object, state)
        }
        (_, Type::User(UserType::Struct(_))) => {
            object::marshal_struct(peek.into_struct()?, scope, object, state)
        }
        _ => Err(ReflectError::OperationFailed {
            shape,
            operation: "unsupported type for serialization (unknown def or type)",
        }
        .into()),
    }
}
