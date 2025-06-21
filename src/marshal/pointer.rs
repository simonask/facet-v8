use std::collections::HashMap;

use facet_core::{Field, KnownSmartPointer, PointerType, PtrConst};
use facet_reflect::{Peek, PeekSmartPointer, ReflectError};

use super::{Error, MarshalState, will_marshal_as_object};

#[derive(Default)]
pub struct MarshalPointers<'mem, 'scope> {
    shared_pointers: HashMap<PtrConst<'mem>, v8::Local<'scope, v8::Object>>,
}

pub fn serialize_smart_pointer<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    peek: PeekSmartPointer<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
    field: Option<&Field>,
) -> Result<v8::Local<'scope, v8::Value>, Error<'shape>> {
    let (is_shared, is_weak) = match peek.def().known {
        Some(KnownSmartPointer::Arc | KnownSmartPointer::Rc) => (true, false),
        Some(KnownSmartPointer::ArcWeak | KnownSmartPointer::RcWeak) => (true, true),
        _ => (false, false),
    };

    if is_weak {
        unimplemented!("weak smart pointers are not supported (yet)");
    }

    let Some(pointee) = peek.borrow_inner() else {
        return Ok(state.null.into());
    };

    // TODO: Once we gain general support for references (immutable borrows),
    // all pointers are essentially shared pointers.

    if is_shared && will_marshal_as_object(pointee.shape()) {
        let ptr = pointee
            .data()
            .thin()
            .expect("DST shared pointers are not supported (yet)");

        if let Some(shared) = state.pointers.shared_pointers.get(&ptr) {
            // We already serialized this pointer, so just return the existing
            // object.
            return Ok((*shared).into());
        }

        // We didn't, let's create the object.
        let obj = super::object::create_object_for_shape(pointee, scope, state, field)?;
        // Insert the object into the shared pointers map before populating it,
        // in case there are circular references.
        state.pointers.shared_pointers.insert(ptr, obj);
        // Finally populate the object with the pointee's fields.
        super::marshal_into_object(pointee, scope, obj, state)?;
        Ok(obj.into())
    } else {
        // Not a shared pointer, or the pointee is not an object, so just
        // serialize it as a direct value.
        super::marshal_value(pointee, scope, state, field)
    }
}

pub fn serialize_pointer<'mem, 'facet, 'shape, 'scope>(
    peek: Peek<'mem, 'facet, 'shape>,
    pointer_type: PointerType<'shape>,
    scope: &mut v8::HandleScope<'scope>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<v8::Local<'scope, v8::Value>, Error<'shape>> {
    match pointer_type {
        PointerType::Reference(_) => {
            if let Ok(s) = peek.get::<&str>() {
                let s = v8::String::new(scope, s).expect("string too large");
                Ok(s.into())
            } else {
                // TODO: Need access to the pointee through facet. When that
                // lands, all pointers essentially become shared pointers.
                _ = state;
                Err(Error::Reflect(ReflectError::OperationFailed {
                    shape: peek.shape(),
                    operation: "cannot serialize reference to non-string type (yet)",
                }))
            }
        }
        PointerType::Raw(_) => Err(Error::Reflect(ReflectError::OperationFailed {
            shape: peek.shape(),
            operation: "cannot serialize raw pointers",
        })),
        PointerType::Function(_) => Err(Error::Reflect(ReflectError::OperationFailed {
            shape: peek.shape(),
            operation: "cannot serialize function pointers",
        })),
    }
}
