use facet_core::{EnumType, Shape, StructKind};
use facet_reflect::{HasFields as _, PeekEnum};

use super::{Error, MarshalState};

/// The type of the enum tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum EnumTagRepr {
    /// Serialize the enum tag as a string.
    #[default]
    String,
    /// Serialize the enum tag as a number (the variant repr value).
    Number,
}

/// How to map Rust enums to JavaScript objects or values.
///
/// Customize this per type to match common patterns in TypeScript etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EnumBehavior<'shape> {
    /// The type of the enum tag (string or number).
    pub js_enum_repr: EnumTagRepr,
    /// The name of the tag field in the serialized object.
    pub js_enum_tag: &'shape str,
}

// I would love for this to be a const fn, but it can't because of the string
// comparisons, and there's no way to hook into facet's arbitrary attributes
// without dealing with strings.
fn enum_behavior_for_shape<'shape>(shape: &Shape<'shape>) -> EnumBehavior<'shape> {
    let mut behavior = EnumBehavior {
        js_enum_repr: EnumTagRepr::String,
        js_enum_tag: "type",
    };

    for attr in shape.attributes.iter() {
        let facet_core::ShapeAttribute::Arbitrary(attr) = attr else {
            continue;
        };
        let Some((k, v)) = attr.split_once('=') else {
            continue;
        };
        match k.trim_ascii() {
            "js_enum_tag" => {
                behavior.js_enum_tag = v.trim_ascii();
            }
            "js_enum_repr" => match v.trim_ascii() {
                "\"string\"" => behavior.js_enum_repr = EnumTagRepr::String,
                "\"number\"" => behavior.js_enum_repr = EnumTagRepr::Number,
                _ => panic!(
                    "invalid js_enum_repr value: {} (expected \"string\" or \"number\")",
                    v
                ),
            },
            _ => continue,
        }
    }

    behavior
}

pub const fn will_serialize_as_object(t: EnumType) -> bool {
    let mut i = 0;
    let len = t.variants.len();
    loop {
        if i >= len {
            return false;
        }
        let variant = &t.variants[i];
        if !matches!(variant.data.kind, StructKind::Unit) {
            return true;
        }
        i += 1;
    }
}

fn serialize_enum_tag<'scope>(
    repr: EnumTagRepr,
    variant: &facet_core::Variant,
    scope: &mut v8::HandleScope<'scope>,
) -> v8::Local<'scope, v8::Value> {
    match repr {
        EnumTagRepr::String => {
            let tag = v8::String::new_from_utf8(
                scope,
                variant.name.as_bytes(),
                v8::NewStringType::Internalized,
            )
            .expect("failed to create enum tag string");
            tag.into()
        }
        EnumTagRepr::Number => {
            let repr_value = variant.discriminant.unwrap_or(0);
            v8::Integer::new(scope, repr_value as i32).into()
        }
    }
}

/// Serialize a unit enum variant as a value.
///
/// Depending on the enum's attributes, this returns either a string (the
/// variant name) or a number (the discriminant value).
pub fn serialize_enum_unit<'mem, 'facet, 'shape, 'scope>(
    peek: PeekEnum<'mem, 'facet, 'shape>,
    enum_type: EnumType<'shape>,
    scope: &mut v8::HandleScope<'scope>,
) -> Result<v8::Local<'scope, v8::Value>, Error<'shape>> {
    let shape = peek.shape();
    debug_assert!(!will_serialize_as_object(enum_type));
    // TODO: Cache this.
    let enum_behavior = enum_behavior_for_shape(shape);
    let active_variant = peek.active_variant()?;
    Ok(serialize_enum_tag(
        enum_behavior.js_enum_repr,
        active_variant,
        scope,
    ))
}

pub fn serialize_enum_object_into<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    peek: PeekEnum<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<(), Error<'shape>> {
    let shape = peek.shape();
    // TODO: Cache this.
    let enum_behavior = enum_behavior_for_shape(shape);
    let active_variant = peek.active_variant()?;

    let tag = serialize_enum_tag(enum_behavior.js_enum_repr, active_variant, scope);

    // Setting the tag field up front to ensure that V8 uses the optimal
    // metaclass chain.
    let tag_field = v8::String::new_from_utf8(
        scope,
        enum_behavior.js_enum_tag.as_bytes(),
        v8::NewStringType::Internalized,
    )
    .ok_or(Error::Exception)?;
    object
        .set(scope, tag_field.into(), tag)
        .ok_or(Error::Exception)?;

    for (field, field_value) in peek.fields_for_serialize() {
        let field_name = field.name;
        if field_name == enum_behavior.js_enum_tag {
            return Err(Error::ClobberedTypeTag(peek.shape()));
        }

        let field_name = v8::String::new_from_utf8(
            scope,
            field_name.as_bytes(),
            v8::NewStringType::Internalized,
        )
        .ok_or(Error::Exception)?;
        let field_value = super::marshal_value(field_value, scope, state, Some(&field))?;
        object
            .set(scope, field_name.into(), field_value)
            .ok_or(Error::Exception)?;
    }

    Ok(())
}
