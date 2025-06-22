use std::borrow::Cow;

use crate::marshal::UnmarshalState;

use super::{Error, MarshalState};
use facet_core::Shape;
use facet_reflect::{Partial, Peek, ReflectError, ScalarType};

pub fn scalar_to_v8<'mem, 'facet, 'shape, 'scope>(
    peek: Peek<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    state: &MarshalState<'mem, 'scope, '_, '_>,
) -> Result<v8::Local<'scope, v8::Value>, Error<'shape>> {
    let peek = peek.innermost_peek();
    // TODO: Pray that this optimizes decently.
    match peek
        .scalar_type()
        .expect("def indicated a scalar, but innermost value is not a scalar type")
    {
        ScalarType::Unit => Ok(state.null.into()),
        ScalarType::Bool => Ok(v8::Boolean::new(scope, *peek.get().unwrap()).into()),
        ScalarType::Char => {
            let c = *peek.get::<char>().unwrap();
            let mut buf = [0; 4];
            let s = c.encode_utf8(&mut buf);
            let s = v8::String::new_from_utf8(scope, s.as_bytes(), v8::NewStringType::Normal)
                .expect("failed to create string from char");
            Ok(s.into())
        }
        ScalarType::Str | ScalarType::String | ScalarType::CowStr => {
            let s = v8::String::new(
                scope,
                peek.as_str()
                    .expect("ScalarType was string-like, but Peek::as_str() returned `None`"),
            )
            .expect("string too long");
            Ok(s.into())
        }
        ScalarType::F32 => {
            let f = *peek.get::<f32>().unwrap();
            Ok(v8::Number::new(scope, f as f64).into())
        }
        ScalarType::F64 => {
            let f = *peek.get::<f64>().unwrap();
            Ok(v8::Number::new(scope, f).into())
        }
        ScalarType::U8 => {
            let u = *peek.get::<u8>().unwrap();
            Ok(v8::Integer::new(scope, u as i32).into())
        }
        ScalarType::U16 => {
            let u = *peek.get::<u16>().unwrap();
            Ok(v8::Integer::new(scope, u as i32).into())
        }
        ScalarType::U32 => {
            let u = *peek.get::<u32>().unwrap();
            Ok(v8::Integer::new_from_unsigned(scope, u).into())
        }
        ScalarType::U64 => {
            let u = *peek.get::<u64>().unwrap();
            Ok(v8::BigInt::new_from_u64(scope, u).into())
        }
        ScalarType::U128 => {
            let u = *peek.get::<u128>().unwrap();
            Ok(u128_to_bigint(scope, u).into())
        }
        ScalarType::USize => {
            let u = *peek.get::<usize>().unwrap();
            Ok(v8::BigInt::new_from_u64(scope, u as u64).into())
        }
        ScalarType::I8 => {
            let i = *peek.get::<i8>().unwrap();
            Ok(v8::Integer::new(scope, i as i32).into())
        }
        ScalarType::I16 => {
            let i = *peek.get::<i16>().unwrap();
            Ok(v8::Integer::new(scope, i as i32).into())
        }
        ScalarType::I32 => {
            let i = *peek.get::<i32>().unwrap();
            Ok(v8::Integer::new(scope, i).into())
        }
        ScalarType::I64 => {
            let i = *peek.get::<i64>().unwrap();
            Ok(v8::BigInt::new_from_i64(scope, i).into())
        }
        ScalarType::I128 => {
            let i = *peek.get::<i128>().unwrap();
            Ok(i128_to_bigint(scope, i).into())
        }
        ScalarType::ISize => {
            let i = *peek.get::<isize>().unwrap();
            Ok(v8::BigInt::new_from_i64(scope, i as i64).into())
        }
        ScalarType::SocketAddr => {
            let addr = peek.get::<core::net::SocketAddr>().unwrap().to_string();
            let s = v8::String::new_from_utf8(scope, addr.as_bytes(), v8::NewStringType::Normal)
                .expect("failed to create string from SocketAddr");
            Ok(s.into())
        }
        ScalarType::IpAddr => {
            let ip = peek.get::<core::net::IpAddr>().unwrap().to_string();
            let s = v8::String::new_from_utf8(scope, ip.as_bytes(), v8::NewStringType::Normal)
                .expect("failed to create string from IpAddr");
            Ok(s.into())
        }
        ScalarType::Ipv4Addr => {
            let ip = peek.get::<core::net::Ipv4Addr>().unwrap().to_string();
            let s = v8::String::new_from_utf8(scope, ip.as_bytes(), v8::NewStringType::Normal)
                .expect("failed to create string from Ipv4Addr");
            Ok(s.into())
        }
        ScalarType::Ipv6Addr => {
            let ip = peek.get::<core::net::Ipv6Addr>().unwrap().to_string();
            let s = v8::String::new_from_utf8(scope, ip.as_bytes(), v8::NewStringType::Normal)
                .expect("failed to create string from Ipv6Addr");
            Ok(s.into())
        }
        _ => Err(ReflectError::OperationFailed {
            shape: peek.shape(),
            operation: "unsupported scalar type for serialization",
        }
        .into()),
    }
}

pub fn scalar_from_v8<'scope, 'partial, 'facet, 'shape>(
    scope: &mut v8::HandleScope<'scope>,
    value: v8::Local<'scope, v8::Value>,
    partial: &'partial mut Partial<'facet, 'shape>,
    state: &mut UnmarshalState<'_, 'scope>,
) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>> {
    let shape = partial.shape();
    match ScalarType::try_from_shape(shape)
        .expect("def indicated a scalar, but shape is not a scalar type")
    {
        ScalarType::Unit => {
            if value.is_null_or_undefined() {
                partial.set_default().unwrap();
                Ok(partial)
            } else {
                Err(Error::unexpected(shape, value.type_repr()))
            }
        }
        ScalarType::Bool => {
            if value.is_true() {
                partial.set(true).map_err(Into::into)
            } else if value.is_false() {
                partial.set(false).map_err(Into::into)
            } else {
                Err(Error::unexpected(shape, value.type_repr()))
            }
        }
        ScalarType::Char => {
            let s = string_from_v8(scope, value, partial.shape(), state)?;
            let mut chars = s.chars();
            let first_char = chars.next().ok_or(ReflectError::OperationFailed {
                shape,
                operation: "expected a single character for char type",
            })?;
            if chars.next().is_some() {
                return Err(ReflectError::OperationFailed {
                    shape,
                    operation: "expected a single character for char type",
                }
                .into());
            };
            partial.set(first_char).map_err(Into::into)
        }
        ScalarType::Str => Err(ReflectError::OperationFailed {
            shape,
            operation: "cannot unmarshal string slices directly; use `String` or `Cow<str>` instead",
        }.into()),
        ScalarType::String => {
            let s = string_from_v8(scope, value, partial.shape(), state)?;
            partial.set(s.into_owned()).map_err(Into::into)
        }
        ScalarType::CowStr => {
            let s = string_from_v8(scope, value, partial.shape(), state)?;
            partial.set(Cow::Owned(s.into_owned())).map_err(Into::into)
        }
        ScalarType::F32 => {
            let number = value
                .to_number(scope)
                .ok_or(Error::unexpected(shape, value.type_repr()))?;
            partial.set(number.value() as f32).map_err(Into::into)
        }
        ScalarType::F64 => {
            let number = value
                .to_number(scope)
                .ok_or(Error::unexpected(shape, value.type_repr()))?;
            partial.set(number.value()).map_err(Into::into)
        }
        ScalarType::U8 => unmarshal_via::<u8, u64>(value, partial),
        ScalarType::U16 => unmarshal_via::<u16, u64>(value, partial),
        ScalarType::U32 => unmarshal_via::<u32, u64>(value, partial),
        ScalarType::U64 => unmarshal_via::<u64, u64>(value, partial),
        ScalarType::U128 => unmarshal_via::<u128, u128>(value, partial),
        ScalarType::USize => unmarshal_via::<usize, u64>(value, partial),
        ScalarType::I8 => unmarshal_via::<i8, i64>(value, partial),
        ScalarType::I16 => unmarshal_via::<i16, i64>(value, partial),
        ScalarType::I32 => unmarshal_via::<i32, i64>(value, partial),
        ScalarType::I64 => unmarshal_via::<i64, i64>(value, partial),
        ScalarType::I128 => unmarshal_via::<i128, i128>(value, partial),
        ScalarType::ISize => unmarshal_via::<isize, i64>(value, partial),
        ScalarType::SocketAddr
        | ScalarType::IpAddr
        | ScalarType::Ipv4Addr
        | ScalarType::Ipv6Addr => {
            let s = string_from_v8(scope, value, partial.shape(), state)?;
            partial.parse_from_str(s.as_ref()).map_err(Into::into)
        }
        _ => Err(Error::unexpected(partial.shape(), value.type_repr())),
    }
}

fn string_from_v8<'scope, 'shape, 'state>(
    scope: &mut v8::HandleScope<'scope>,
    value: v8::Local<'scope, v8::Value>,
    shape: &'shape Shape<'shape>,
    state: &'state mut UnmarshalState<'_, 'scope>,
) -> Result<Cow<'state, str>, Error<'shape>> {
    if let Ok(s) = v8::Local::<v8::String>::try_from(value) {
        Ok(s.to_rust_cow_lossy(scope, &mut state.string_conversion_buffer))
    } else {
        Err(Error::unexpected(shape, value.type_repr()))
    }
}

trait IntConversion: Sized {
    fn int_from_v8<'shape>(
        value: v8::Local<v8::Value>,
        shape: &'shape Shape<'shape>,
    ) -> Result<Self, Error<'shape>>;
}

impl IntConversion for i64 {
    fn int_from_v8<'shape>(
        value: v8::Local<v8::Value>,
        shape: &'shape Shape<'shape>,
    ) -> Result<i64, Error<'shape>> {
        if let Ok(number) = v8::Local::<v8::Integer>::try_from(value) {
            Ok(number.value())
        } else if let Ok(bigint) = v8::Local::<v8::BigInt>::try_from(value) {
            let (value, truncated) = bigint.i64_value();
            if truncated {
                Err(Error::IntOverflow(shape))
            } else {
                Ok(value)
            }
        } else {
            Err(Error::UnexpectedValue {
                shape,
                unexpected: value.type_repr(),
            })
        }
    }
}

impl IntConversion for u64 {
    fn int_from_v8<'shape>(
        value: v8::Local<v8::Value>,
        shape: &'shape Shape<'shape>,
    ) -> Result<u64, Error<'shape>> {
        if let Ok(number) = v8::Local::<v8::Integer>::try_from(value) {
            number
                .value()
                .try_into()
                .map_err(|_| Error::IntOverflow(shape))
        } else if let Ok(bigint) = v8::Local::<v8::BigInt>::try_from(value) {
            let (value, lossless) = bigint.u64_value();
            if lossless {
                Ok(value)
            } else {
                Err(Error::IntOverflow(shape))
            }
        } else {
            Err(Error::UnexpectedValue {
                shape,
                unexpected: value.type_repr(),
            })
        }
    }
}

impl IntConversion for i128 {
    fn int_from_v8<'shape>(
        value: v8::Local<v8::Value>,
        shape: &'shape Shape<'shape>,
    ) -> Result<i128, Error<'shape>> {
        if let Ok(number) = v8::Local::<v8::Integer>::try_from(value) {
            Ok(number.value() as i128)
        } else if let Ok(bigint) = v8::Local::<v8::BigInt>::try_from(value) {
            bigint_to_i128(bigint).ok_or(Error::IntOverflow(shape))
        } else {
            Err(Error::UnexpectedValue {
                shape,
                unexpected: value.type_repr(),
            })
        }
    }
}

impl IntConversion for u128 {
    fn int_from_v8<'shape>(
        value: v8::Local<v8::Value>,
        shape: &'shape Shape<'shape>,
    ) -> Result<u128, Error<'shape>> {
        if let Ok(number) = v8::Local::<v8::Integer>::try_from(value) {
            let u: u64 = number
                .value()
                .try_into()
                .map_err(|_| Error::IntOverflow(shape))?;
            Ok(u as u128)
        } else if let Ok(bigint) = v8::Local::<v8::BigInt>::try_from(value) {
            bigint_to_u128(bigint).ok_or(Error::IntOverflow(shape))
        } else {
            Err(Error::UnexpectedValue {
                shape,
                unexpected: value.type_repr(),
            })
        }
    }
}

fn unmarshal_via<
    'scope,
    'partial,
    'facet,
    'shape,
    T: facet_core::Facet<'facet> + TryFrom<U>,
    U: IntConversion,
>(
    value: v8::Local<'scope, v8::Value>,
    partial: &'partial mut Partial<'facet, 'shape>,
) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>> {
    let shape = partial.shape();
    let i: T = U::int_from_v8(value, shape)?
        .try_into()
        .map_err(|_| Error::IntOverflow(T::SHAPE))?;
    partial.set::<T>(i).map_err(Into::into)
}

fn u128_to_bigint<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    value: u128,
) -> v8::Local<'scope, v8::BigInt> {
    let lo = (value & 0xffff_ffff_ffff_ffff) as u64;
    let hi = (value >> 64) as u64;
    v8::BigInt::new_from_words(scope, false, &[lo, hi]).expect("failed to create bigint from u128")
}

fn i128_to_bigint<'scope>(
    scope: &mut v8::HandleScope<'scope>,
    value: i128,
) -> v8::Local<'scope, v8::BigInt> {
    if value == i128::MIN {
        // Special case for i128::MIN, which cannot be represented as a u128,
        // so this requires 3 words to represent.
        v8::BigInt::new_from_words(scope, true, &[0, 0, 1])
            .expect("failed to create bigint from i128::MIN")
    } else {
        let sign_bit = value < 0;
        let u = if sign_bit {
            -value as u128
        } else {
            value as u128
        };
        let lo = (u & 0xffff_ffff_ffff_ffff) as u64;
        let hi = (u >> 64) as u64;
        v8::BigInt::new_from_words(scope, sign_bit, &[lo, hi])
            .expect("failed to create bigint from i128")
    }
}

fn bigint_to_u128(value: v8::Local<v8::BigInt>) -> Option<u128> {
    let mut buf = [0; 3];
    let (sign, words) = value.to_words_array(&mut buf);
    if sign {
        return None; // Negative bigint cannot be converted to u128
    }
    match words.len() {
        1 => Some(words[0] as u128),
        2 => Some((words[1] as u128) << 64 | (words[0] as u128)),
        _ => None,
    }
}

fn bigint_to_i128(value: v8::Local<v8::BigInt>) -> Option<i128> {
    let mut buf = [0; 4];
    let (sign, words) = value.to_words_array(&mut buf);
    match words.len() {
        1 => Some(if sign {
            -(words[0] as i128)
        } else {
            words[0] as i128
        }),
        2 => Some(if sign {
            -((words[1] as i128) << 64 | (words[0] as i128))
        } else {
            (words[1] as i128) << 64 | (words[0] as i128)
        }),
        3 if sign && words == [0, 0, 1] => {
            Some(i128::MIN) // Special case for i128::MIN
        }
        _ => None, // Unsupported bigint size for i128
    }
}
