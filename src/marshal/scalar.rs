use super::{Error, MarshalState};
use facet_reflect::{Peek, ReflectError, ScalarType};

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
            let s = v8::String::new(scope, peek.as_str().unwrap()).expect("string too long");
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
            let lo = (u & 0xffff_ffff_ffff_ffff) as u64;
            let hi = (u >> 64) as u64;
            v8::BigInt::new_from_words(scope, false, &[lo, hi])
                .ok_or(Error::Exception)
                .map(Into::into)
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
            let u;
            let sign_bit;
            if i < 0 {
                if i == i128::MIN {
                    // Special case for i128::MIN, which cannot be
                    // represented as a u128, so this requires 3 words to
                    // represent.
                    return v8::BigInt::new_from_words(scope, true, &[0, 0, 1])
                        .ok_or(Error::Exception)
                        .map(Into::into);
                }

                sign_bit = true;
                u = -i as u128;
            } else {
                sign_bit = false;
                u = i as u128;
            }
            let lo = (u & 0xffff_ffff_ffff_ffff) as u64;
            let hi = (u >> 64) as u64;
            v8::BigInt::new_from_words(scope, sign_bit, &[lo, hi])
                .ok_or(Error::Exception)
                .map(Into::into)
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
