use std::mem::MaybeUninit;

use facet_core::{ConstTypeId, Field, FieldAttribute, Shape};
use facet_reflect::{Partial, Peek, PeekTuple};

use crate::marshal::UnmarshalState;

use super::{Error, MarshalState};

/// Populate an array-like JS object from an array-like Rust type.
///
/// When `object` is a plain JS array, each element of the Rust array is
/// marhalled normally.
///
/// However, when `object` is a typed array (`Uint8Array`, `Float64Array`,
/// etc.), it is populated with raw data from the Rust type. When the Rust type
/// is a matching `Vec<T>` or `&[T]`, the data is copied directly into the JS
/// `ArrayBuffer` backing the array (effectively a `memcpy()`).
pub fn marshal_list_object<'mem, 'facet: 'mem, 'shape: 'mem + 'facet, 'scope>(
    peek: Peek<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<(), Error<'shape>> {
    if object.is_array() {
        // The target object is a plain old array; marshal objects the old
        // fashioned way, no shenanigans.
        if let Ok(peek_list) = peek.into_list() {
            marshal_array_object(peek_list.iter(), scope, object, state)
        } else if let Ok(peek_list_like) = peek.into_list_like() {
            marshal_array_object(peek_list_like.iter(), scope, object, state)
        } else {
            panic!("expected a list or list-like object to populate an array");
        }
    } else if object.is_typed_array() {
        // Fast paths for typed arrays.
        let peek_list_like = peek
            .into_list_like()
            .expect("expected a list-like object for typed array");
        let t = peek_list_like.def().t();

        if let Ok(array) = v8::Local::<v8::Uint8Array>::try_from(object) {
            u8::marshal(scope, array, peek)
        } else if let Ok(array) = v8::Local::<v8::Int8Array>::try_from(object) {
            i8::marshal(scope, array, peek)
        } else if let Ok(array) = v8::Local::<v8::Uint16Array>::try_from(object) {
            u16::marshal(scope, array, peek)
        } else if let Ok(array) = v8::Local::<v8::Int16Array>::try_from(object) {
            i16::marshal(scope, array, peek)
        } else if let Ok(array) = v8::Local::<v8::Uint32Array>::try_from(object) {
            u32::marshal(scope, array, peek)
        } else if let Ok(array) = v8::Local::<v8::Int32Array>::try_from(object) {
            i32::marshal(scope, array, peek)
        } else if let Ok(array) = v8::Local::<v8::Float32Array>::try_from(object) {
            f32::marshal(scope, array, peek)
        } else if let Ok(array) = v8::Local::<v8::Float64Array>::try_from(object) {
            f64::marshal(scope, array, peek)
        } else {
            panic!("array buffer type mismatch: {t}");
        }
    } else {
        panic!("object constructor did not create an array or typed array");
    }
}

pub fn unmarshal_list_object<'scope, 'partial, 'facet, 'shape: 'facet>(
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    partial: &'partial mut Partial<'facet, 'shape>,
    state: &mut UnmarshalState<'_, 'scope>,
) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>> {
    if let Ok(array) = object.try_into() {
        unmarshal_array_object(scope, array, partial, state)
    } else if object.is_typed_array() {
        // Fast paths for typed arrays.
        if let Ok(array) = v8::Local::<v8::Uint8Array>::try_from(object) {
            u8::unmarshal(scope, array, partial)?;
        } else if let Ok(array) = v8::Local::<v8::Int8Array>::try_from(object) {
            i8::unmarshal(scope, array, partial)?;
        } else if let Ok(array) = v8::Local::<v8::Uint16Array>::try_from(object) {
            u16::unmarshal(scope, array, partial)?;
        } else if let Ok(array) = v8::Local::<v8::Int16Array>::try_from(object) {
            i16::unmarshal(scope, array, partial)?;
        } else if let Ok(array) = v8::Local::<v8::Uint32Array>::try_from(object) {
            u32::unmarshal(scope, array, partial)?;
        } else if let Ok(array) = v8::Local::<v8::Int32Array>::try_from(object) {
            i32::unmarshal(scope, array, partial)?;
        } else if let Ok(array) = v8::Local::<v8::Float32Array>::try_from(object) {
            f32::unmarshal(scope, array, partial)?;
        } else if let Ok(array) = v8::Local::<v8::Float64Array>::try_from(object) {
            f64::unmarshal(scope, array, partial)?;
        } else {
            unreachable!("unhandled TypedArray type (did JS gain new ones?)");
        }

        Ok(partial)
    } else {
        Err(Error::UnexpectedValue {
            shape: partial.shape(),
            unexpected: object.type_repr(),
        })
    }
}

/// Marshal each item from an iterator and set its value in the array-like
/// object. `array` can be any object that supports `set_index()`, including
/// `v8::Array` or any of the typed arrays.
fn marshal_array_object<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    iter: impl Iterator<Item = Peek<'mem, 'facet, 'shape>> + 'mem,
    scope: &mut v8::HandleScope<'scope>,
    array: v8::Local<'scope, v8::Object>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<(), Error<'shape>> {
    for (i, item) in iter.enumerate() {
        let item_value = super::marshal_value(item, scope, state, None)?;
        array
            .set_index(scope, i as u32, item_value)
            .ok_or(Error::Exception)?;
    }
    Ok(())
}

fn unmarshal_array_object<'scope, 'partial, 'facet, 'shape: 'facet>(
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Array>,
    partial: &'partial mut Partial<'facet, 'shape>,
    state: &mut UnmarshalState<'_, 'scope>,
) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>> {
    let len = object.length();
    let has_default = partial.shape().has_default_attr();
    partial.begin_list()?;
    for i in 0..len {
        let item = object.get_index(scope, i).ok_or(Error::Exception)?;
        super::unmarshal_value(scope, item, partial.begin_list_item()?, state)?.end()?;
    }
    if has_default {
        partial.fill_unset_fields_from_default()?;
    }
    // Note: `begin_list()` does not push a frame.
    Ok(partial)
}

pub fn marshal_tuple_object<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    peek: PeekTuple<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<(), Error<'shape>> {
    for (i, (field, field_value)) in peek.fields().enumerate() {
        let item = super::marshal_value(field_value, scope, state, Some(&field))?;
        object
            .set_index(scope, i as u32, item)
            .ok_or(Error::Exception)?;
    }
    Ok(())
}

pub fn unmarshal_tuple<'scope, 'partial, 'facet, 'shape: 'facet>(
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    partial: &'partial mut Partial<'facet, 'shape>,
    state: &mut UnmarshalState<'_, 'scope>,
) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>> {
    if let Ok(array) = object.try_into() {
        unmarshal_array_object(scope, array, partial, state)
    } else {
        Err(Error::UnexpectedValue {
            shape: partial.shape(),
            unexpected: object.type_repr(),
        })
    }
}

/// Create an array for the given shape.
///
/// If the field has the `array_buffer` attribute, a typed array is created.
/// Otherwise, a plain JS array is created with the specified length.
pub fn create_array_for_shape<'shape, 'scope>(
    scope: &mut v8::HandleScope<'scope>,
    len: usize,
    t: &'shape Shape<'shape>,
    field: Option<&Field>,
) -> Result<v8::Local<'scope, v8::Object>, Error<'shape>> {
    if let Some(field) = field {
        if field
            .attributes
            .contains(&FieldAttribute::Arbitrary("typed_array"))
        {
            return create_arraybuffer_for_shape(scope, len, t);
        }
    }

    Ok(v8::Array::new(scope, len.try_into().expect("array too large")).into())
}

/// Create a typed array with the appropriate type for the given shape.
fn create_arraybuffer_for_shape<'shape, 'scope>(
    scope: &mut v8::HandleScope<'scope>,
    len: usize,
    t: &'shape Shape<'shape>,
) -> Result<v8::Local<'scope, v8::Object>, Error<'shape>> {
    let buffer = if t.id == ConstTypeId::of::<u8>() {
        u8::create_typed_array_for_len(scope, len).into()
    } else if t.id == ConstTypeId::of::<u16>() {
        u16::create_typed_array_for_len(scope, len).into()
    } else if t.id == ConstTypeId::of::<u32>() {
        u32::create_typed_array_for_len(scope, len).into()
    } else if t.id == ConstTypeId::of::<i8>() {
        i8::create_typed_array_for_len(scope, len).into()
    } else if t.id == ConstTypeId::of::<i16>() {
        i16::create_typed_array_for_len(scope, len).into()
    } else if t.id == ConstTypeId::of::<i32>() {
        i32::create_typed_array_for_len(scope, len).into()
    } else if t.id == ConstTypeId::of::<f32>() {
        f32::create_typed_array_for_len(scope, len).into()
    } else if t.id == ConstTypeId::of::<f64>() {
        f64::create_typed_array_for_len(scope, len).into()
    } else {
        panic!("unsupported array buffer type: {t}");
    };

    Ok(buffer)
}

trait TypedArrayType: bytemuck::Pod + 'static {
    type TypedArray<'scope>: Into<v8::Local<'scope, v8::TypedArray>> + Copy;
    fn create_typed_array_for_len<'scope>(
        scope: &mut v8::HandleScope<'scope>,
        len: usize,
    ) -> Self::TypedArray<'scope>;
    fn wrap_buffer<'scope>(
        scope: &mut v8::HandleScope<'scope>,
        buffer: v8::Local<'scope, v8::ArrayBuffer>,
    ) -> Self::TypedArray<'scope>;

    /// Given a `TypedArray` handle and a `Peek` representing a sequence, copy
    /// the data from the container into the array in the fastest possible way.
    fn marshal<'scope, 'shape>(
        scope: &mut v8::HandleScope<'scope>,
        handle: Self::TypedArray<'scope>,
        peek: Peek<'_, '_, 'shape>,
    ) -> Result<(), Error<'shape>>;

    /// Given a `TypedArray` handle and a `Partial` container, copy the data
    /// from the array into the container in the fastest possible way.
    fn unmarshal<'scope, 'partial, 'facet, 'shape>(
        scope: &mut v8::HandleScope<'scope>,
        handle: Self::TypedArray<'scope>,
        container: &'partial mut Partial<'facet, 'shape>,
    ) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>>;

    fn copy_to_partial_list<'partial, 'facet, 'shape>(
        buffer: v8::Local<v8::ArrayBuffer>,
        partial: &'partial mut Partial<'facet, 'shape>,
    ) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>>
    where
        // TODO: Not sure why these are needed.
        Vec<Self>: facet_core::Facet<'facet>,
        Self: facet_core::Facet<'facet>,
    {
        let byte_len = buffer.byte_length();
        let buffer_bytes: &[u8] = unsafe {
            buffer
                .data()
                .map(|ptr| std::slice::from_raw_parts(ptr.as_ptr() as *mut u8, byte_len))
                .unwrap_or(&[])
        };

        // Fast path for Vec.
        if partial.shape().id == ConstTypeId::of::<Vec<Self>>() {
            let len = byte_len / size_of::<Self>();
            let mut vec = Vec::<MaybeUninit<Self>>::with_capacity(len);
            unsafe {
                vec.set_len(len);
                std::ptr::copy_nonoverlapping(
                    buffer_bytes.as_ptr() as *const Self,
                    vec.as_mut_ptr() as *mut Self,
                    len,
                );
                let vec: Vec<Self> = std::mem::transmute(vec);
                partial.set(vec)?;
                return Ok(partial);
            }
        }

        partial.begin_list()?;
        for chunk in buffer_bytes.chunks_exact(size_of::<Self>()) {
            let item: Self = bytemuck::pod_read_unaligned(chunk);
            partial.push(item)?;
        }
        // Note: `begin_list()` does not push a frame.
        Ok(partial)
    }

    fn set_data_slice<'scope>(buffer: v8::Local<'scope, v8::ArrayBuffer>, data: &[Self]) {
        let buffer_bytes: &mut [u8] = unsafe {
            buffer
                .data()
                .map(|ptr| {
                    std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, buffer.byte_length())
                })
                .unwrap_or(&mut [])
        };
        buffer_bytes.copy_from_slice(bytemuck::cast_slice(data));
    }

    fn set_data_iter<'scope>(
        buffer: v8::Local<'scope, v8::ArrayBuffer>,
        iter: impl Iterator<Item = Self>,
    ) {
        let mut buffer_bytes: &mut [u8] = unsafe {
            buffer
                .data()
                .map(|ptr| {
                    std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, buffer.byte_length())
                })
                .unwrap_or(&mut [])
        };
        for item in iter {
            let (item_bytes, rest) = buffer_bytes
                .split_at_mut_checked(std::mem::size_of::<Self>())
                .expect("buffer too small to hold all items from the sequence");
            buffer_bytes = rest;
            item_bytes[0..size_of::<Self>()].copy_from_slice(bytemuck::bytes_of(&item));
        }
    }
}

macro_rules! impl_typed_array_type {
    ($type:ty, $array_type:ident) => {
        impl TypedArrayType for $type {
            type TypedArray<'scope> = v8::Local<'scope, v8::$array_type>;

            fn create_typed_array_for_len<'scope>(
                scope: &mut v8::HandleScope<'scope>,
                len: usize,
            ) -> v8::Local<'scope, v8::$array_type> {
                let buffer = v8::ArrayBuffer::new(scope, len * std::mem::size_of::<$type>());
                Self::wrap_buffer(scope, buffer)
            }

            fn wrap_buffer<'scope>(
                scope: &mut v8::HandleScope<'scope>,
                buffer: v8::Local<'scope, v8::ArrayBuffer>,
            ) -> Self::TypedArray<'scope> {
                v8::$array_type::new(
                    scope,
                    buffer,
                    0,
                    buffer.byte_length() / std::mem::size_of::<$type>(),
                )
                .unwrap()
            }

            fn marshal<'scope, 'shape>(
                scope: &mut v8::HandleScope<'scope>,
                handle: Self::TypedArray<'scope>,
                peek: Peek<'_, '_, 'shape>,
            ) -> Result<(), Error<'shape>> {
                let buffer = handle
                    .buffer(scope)
                    .expect("typed array does not have a backing array buffer");
                if let Ok(vec) = peek.get::<Vec<$type>>() {
                    // Fast path for Vec.
                    Self::set_data_slice(buffer, &*vec);
                // TODO: Boxed slices when `facet` supports it.
                // } else if let Ok(boxed) = peek.get::<Box<[$type]>>() {
                //     // Fast path for boxed slices.
                //     Self::set_data_slice(buffer, &*boxed);
                } else if let Ok(slice) = peek.get::<&[$type]>() {
                    // Fast path for slices.
                    Self::set_data_slice(buffer, slice);
                // TODO: VecDeque when `facet` supports it.
                // } else if let Ok(vec_deque) = peek.get::<std::collections::VecDeque<$type>>() {
                //     // Fast path for VecDeque.
                //     Self::set_data_from_iter(buffer, vec_deque.iter().copied());
                } else {
                    // Otherwise, we assume it's a list-like object. This is
                    // also checked further up the call stack, so just unwrap.
                    let peek_list_like = peek.into_list_like().unwrap();
                    Self::set_data_iter(
                        buffer,
                        peek_list_like
                            .iter()
                            .map(|item| *item.get::<$type>().expect("array buffer type mismatch")),
                    );
                }
                Ok(())
            }

            fn unmarshal<'scope, 'partial, 'facet, 'shape>(
                scope: &mut v8::HandleScope<'scope>,
                handle: Self::TypedArray<'scope>,
                container: &'partial mut Partial<'facet, 'shape>,
            ) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>> {
                let buffer = handle
                    .buffer(scope)
                    .expect("typed array does not have a backing array buffer");
                Self::copy_to_partial_list(buffer, container)?;
                Ok(container)
            }
        }
    };
}

impl_typed_array_type!(u8, Uint8Array);
impl_typed_array_type!(i8, Int8Array);
impl_typed_array_type!(u16, Uint16Array);
impl_typed_array_type!(i16, Int16Array);
impl_typed_array_type!(u32, Uint32Array);
impl_typed_array_type!(i32, Int32Array);
impl_typed_array_type!(f32, Float32Array);
impl_typed_array_type!(f64, Float64Array);
