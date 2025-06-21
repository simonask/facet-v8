use std::collections::HashMap;

use super::{Error, MarshalState};
use facet_core::{ConstTypeId, Def, Facet, Field};
use facet_reflect::{HasFields as _, Peek, PeekStruct};

/// Customize how to map Rust types to JavaScript objects.
///
/// Rust structs may be created on the JS side using a constructor function
/// (`new Foo()`) or explicit prototype (`Object.create(prototype)`)
///
/// If a type is not registered with a constructor, it will be created on the JS
/// side using the equivalent of `Object.create(null)`.
///
/// Object constructors are invoked without arguments, and fields are set
/// explicitly after the object is created, which also means that for objects
/// that have getters or setters, the setters will be invoked during
/// marshalling. Use with caution.
///
/// Other behaviors:
///
/// - Object constructors only apply to types that will be marshalled as
///   `object`s. Defining an object constructor for a primitive or a unit-only
///   enum has no effect.
/// - Object constructors are ignored for the inner field of
///   `#[facet(transparent)]` types.
/// - Object constructors are ignored for fields that have `#[facet(flatten)]`.`
#[derive(Default)]
pub struct Constructors<'scope, 'env> {
    constructors: HashMap<ConstTypeId, Constructor<'scope, 'env>>,
}

impl<'s, 'env> Constructors<'s, 'env> {
    fn register_constructor<'shape, T: Facet<'shape>>(
        &mut self,
        constructor: Constructor<'s, 'env>,
    ) -> &mut Self {
        if !super::will_marshal_as_object(T::SHAPE) {
            panic!(
                "cannot register a constructor for a type that will not serialize as an object: {}",
                T::SHAPE.type_identifier
            );
        }

        let type_id = T::SHAPE.id;
        self.constructors.insert(type_id, constructor);
        self
    }

    /// Construct `T`s using `Object.create(prototype)`.
    pub fn with_prototype<'shape, T: Facet<'shape>>(
        &mut self,
        prototype: v8::Local<'s, v8::Value>,
    ) -> &mut Self {
        self.register_constructor::<T>(Constructor::Prototype(prototype))
    }

    /// Construct `T`s using `new Foo()`. When used to construct an array or
    /// tuple, the constructor is passed the length as the first argument.
    /// Otherwise, the constructor is invoked with no arguments.
    pub fn with_constructor<'shape, T: Facet<'shape>>(
        &mut self,
        constructor: v8::Local<'s, v8::Function>,
    ) -> &mut Self {
        self.register_constructor::<T>(Constructor::Function(constructor))
    }

    /// Construct `T`s using an internal object template.
    pub fn with_object_template<'shape, T: Facet<'shape>>(
        &mut self,
        object_template: v8::Local<'s, v8::ObjectTemplate>,
    ) -> &mut Self {
        self.register_constructor::<T>(Constructor::ObjectTemplate(object_template))
    }

    /// Construct `T`s using a custom constructor function defined in Rust code.
    ///
    /// The constructor should not populate the object with fields from the
    /// peeked value, but can infer from it how to construct the object.
    ///
    /// The `Field` argument is present when the value is the field of a struct,
    /// and the custom constructor may access its attributes or other
    /// information about the field.
    ///
    /// If the value is a list or tuple, the custom constructor should construct
    /// an array-like object.
    ///
    /// If the custom constructor returns `None`, it means that an exception was
    /// thrown.
    pub fn with_custom_constructor<'shape, T: Facet<'shape>>(
        &mut self,
        custom_constructor: impl FnMut(
            &mut v8::HandleScope<'s>,
            Peek,
            Option<&Field>,
        ) -> Option<v8::Local<'s, v8::Object>>
        + 'env,
    ) -> &mut Self {
        self.register_constructor::<T>(Constructor::Custom(Box::new(custom_constructor)))
    }
}

type CustomConstructorFn<'scope, 'env> = dyn FnMut(
        &mut v8::HandleScope<'scope>,
        Peek,
        Option<&Field>,
    ) -> Option<v8::Local<'scope, v8::Object>>
    + 'env;

enum Constructor<'scope, 'env> {
    /// `Object.create(prototype)`
    Prototype(v8::Local<'scope, v8::Value>),
    /// `new Foo()`
    Function(v8::Local<'scope, v8::Function>),
    /// Internal object template for creating objects.
    ObjectTemplate(v8::Local<'scope, v8::ObjectTemplate>),
    /// Custom constructor defined in Rust code.
    Custom(Box<CustomConstructorFn<'scope, 'env>>),
}

impl<'scope, 'env> Constructor<'scope, 'env> {
    pub fn construct<'shape>(
        &mut self,
        scope: &mut v8::HandleScope<'scope>,
        peek: Peek<'_, '_, 'shape>,
        field: Option<&Field>,
        len: Option<usize>,
    ) -> Result<v8::Local<'scope, v8::Object>, Error<'shape>> {
        match self {
            Constructor::Prototype(prototype) => Ok(v8::Object::with_prototype_and_properties(
                scope,
                *prototype,
                &[],
                &[],
            )),
            Constructor::Function(func) => {
                let len = len.map(|l| v8::Integer::new_from_unsigned(scope, l as u32).into());
                func.new_instance(scope, len.as_slice())
                    .ok_or(Error::Exception)
            }
            Constructor::ObjectTemplate(template) => {
                if len.is_some() {
                    panic!(
                        "object templates cannot be used to create arrays or tuples; use a constructor instead"
                    );
                }
                template.new_instance(scope).ok_or(Error::Exception)
            }
            Constructor::Custom(custom_constructor) => {
                custom_constructor(scope, peek, field).ok_or(Error::Exception)
            }
        }
    }
}

pub fn create_object_for_shape<'mem, 'facet, 'shape, 'scope>(
    peek: Peek<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    state: &mut MarshalState<'_, 'scope, '_, '_>,
    field: Option<&facet_core::Field>,
) -> Result<v8::Local<'scope, v8::Object>, Error<'shape>> {
    let shape = peek.shape();

    if let Ok(tuple) = peek.into_tuple() {
        // If the shape is a tuple, we create an array with the length of the tuple.
        let len = tuple.len();
        return Ok(v8::Array::new(scope, len as i32).into());
    }

    // See if this is an array and get the length if it is.
    let list_len_t = if let Ok(list_like) = peek.into_list_like() {
        Some((list_like.len(), list_like.def().t()))
    } else {
        None
    };

    let constructed = if let Some(constructor) = state.constructors.constructors.get_mut(&shape.id)
    {
        constructor.construct(scope, peek, field, list_len_t.map(|(len, _)| len))?
    } else {
        // If this is a list, create an array or array-like object.
        if let Some((len, def_t)) = list_len_t {
            return super::array::create_array_for_shape(scope, len, def_t, field);
        }

        match shape.def {
            Def::Map(_) => v8::Map::new(scope).into(),
            Def::Set(_) => v8::Set::new(scope).into(),
            Def::List(_) | Def::Array(_) | Def::Slice(_) => {
                unreachable!("list-like objects should have been handled earlier")
            }
            _ => {
                // No special handling, just create an empty plain object.
                v8::Object::new(scope)
            }
        }
    };

    Ok(constructed)
}

pub fn marshal_struct<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    peek: PeekStruct<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    obj: v8::Local<'scope, v8::Object>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<(), Error<'shape>> {
    let fields = peek.fields_for_serialize();
    for (field, field_value) in fields {
        let field_name = v8::String::new_from_utf8(
            scope,
            field.name.as_bytes(),
            v8::NewStringType::Internalized,
        )
        .ok_or(Error::Exception)?;

        let field_value = super::marshal_value(field_value, scope, state, Some(&field))?;
        obj.set(scope, field_name.into(), field_value)
            .ok_or(Error::Exception)?;
    }
    Ok(())
}
