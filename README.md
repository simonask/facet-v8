facet 💖 v8
===========

Facet bindings for the V8 JavaScript engine.

- Convert JavaScript values to/from Rust types with maximum fidelity.
- (TODO) Reflect on Rust types from JavaScript code.

This crate provides general marshalling/unmarshalling support, similar to
serialization/deserialization, but allows for more complex object graphs. For
example, this crate preserves object identity, so it is possible to represent
things like circular data structures.

Container Attributes
--------------------

`facet-v8` respects the following container (type) attributes from
[`#[derive(Facet)]`](https://docs.rs/facet/latest/facet/derive.Facet.html):

- `#[facet(transparent)]` - Erase the type and use the inner type instead.
- `#[facet(skip_serializing)]` and `#[facet(skip_serializing_if = "..")]` - Skip
  serializing this type.

`facet-v8` further introduces the following container attributes:

- `#[facet(js_enum_tag = "type")]`: Objects representing this enum will have a
  `"type"` property indicating the enum variant. This attribute only has an
  effect on enum types with data carrying variants (enums with only unit
  variants are marshalled as their string/number representation). The default is
  `"type"`.
- `#[facet(js_enum_repr = "string" | "number")]`: The enum tag will be either a
  string (the variant name) or a number (the discriminant value). The default is
  `"string"`.

Field Attributes
----------------

`facet-v8` understands the following field attributes from
[`#[derive(Facet)]`](https://docs.rs/facet/latest/facet/derive.Facet.html):

- `#[facet(skip_serializing)]` and `#[facet(skip_serializing_if = "..")]` - Skip
  a field when marshalling/unmarshalling.
- `#[facet(flatten)]` - Flatten this field into the parent object.
- `#[facet(default)]` - Use the default value for this field when it is missing
  during unmarshalling.

`facet-v8` further introduces the following field attributes:

- `#[facet(typed_array)]`: For fields that are sequence types (e.g., `Vec<T>`,
  `&[T]`, `Box<[T]>`, etc.), this attribute indicates that the field should be
  serialized as a JavaScript `TypedArray` containing the plain values of the
  sequence, rather than as a plain JavaScript `Array`. For example, a `Vec<u8>`
  will be serialized as a `Uint8Array` in JavaScript.

Custom constructors
-------------------

When embedding V8 in a Rust/C++ application, it is sometimes necessary to
construct JavaScript objects from native code in custom ways. For example,
objects in V8 may contain external pointers to Rust data, or pointers into V8's
`cppgc` heap.

To support this, `facet-v8` provides a facility to register custom constructors
per type, which will be called when objects of that type are encountered during
object marshalling.

Conversion table
----------------

| Rust Type                      | JavaScript Type | V8 Type        | Notes |
|--------------------------------|-----------------|----------------|-------|
| `()`, `None`                   | `null`          | `v8::Primitive` |       |
| `bool`                         | `boolean`       | `v8::Boolean`   |       |
| Integers up to 32 bits         | `number`        | `v8::Integer`   |       |
| `u64`, `i64`, `u128`, `i128`, `usize`, `isize`   | `bigint`        | `v8::BigInt`   |       |
| `f32`, `f64`                   | `number`        | `v8::Number`    |       |
| `String`, `&str`, `Cow<str>`, `Box<str>` | `string`        | `v8::String`   |       |
| Enums with only unit variants | `string` or `number` | `v8::String` or `v8::Integer` | Based on `#[facet(js_enum_repr = "...")]` |
| Enums with any data-carrying variants | `object`        | `v8::Object`    | Discriminant in `"type"` (or the field indicated by `#[facet(js_enum_tag)]` |
| Tuples `(A, B, ..)`            | `array`         | `v8::Array`     |       |
| Structs                        | `object`        | `v8::Object`    | Except transparent structs where the inner type is a primitive |
| `Vec<T>`, `Box<[T]>`, `&[T]`   | `Array`         | `v8::Array`     | If `T` is a supported primitive, it will be marshalled as a `TypedArray` (`Uint8Array`, etc.) if `#[facet(typed_array)]` is present on the field |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `Map`        | `v8::Map`    | *Caution:* Key comparison is different in JS  |
| `HashSet<T>`, `BTreeSet<T>`     | `Set`           | `v8::Set`       | *Caution:* Element comparison is different in JS |

Semantics and Fidelity
----------------------

One of the key choices of `facet-v8` is to preserve the semantic structure of
exchanged types with high fidelity, but not necessarily the exact behavior.
Language and standard library semantics are wildly different between Rust and
JavaScript, so matching the behavior of types is often impossible. Instead,
`facet-v8` focuses on the semantics of the data structure, which works in the
majority of cases, as long as important caveats are kept in mind.

1. In particular, `Map` and `Set` with non-primitive keys being passed back and
   forth may not behave as expected, because JavaScript `object` comparison is by
   object identity, while Rust has deep equality semantics, so it may appear as if
   maps and sets are losing elements when unmarshalling.
2. Another wart is that floating point numbers (including `NaN`) can be used as map
   keys in JavaScript, but not in Rust (unless using something like
   `OrderedFloat`), so this is a source of errors when unmarshalling maps coming
   from JavaScript.
3. `Option` flattening: JavaScript does not have a concept of `None`, so these
   are represented as `null`. This affects nested options, where `Some(None)`
   will be represented as `null` in JavaScript, and will turn into `None` when
   converted back. Use a struct with a field containing the option if the
   nesting carries information in your use case.
