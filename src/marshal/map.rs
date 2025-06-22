use facet_reflect::{Partial, PeekMap};

use super::{Error, MarshalState, UnmarshalState};

pub fn marshal_map_into<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    peek: PeekMap<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<(), Error<'shape>> {
    let map =
        v8::Local::<v8::Map>::try_from(object).expect("object constructor did not create a map");
    for (key, value) in peek.iter() {
        let key_value = super::marshal_value(key, scope, state, None)?;
        let value_value = super::marshal_value(value, scope, state, None)?;
        map.set(scope, key_value, value_value)
            .ok_or(Error::Exception)?;
    }
    Ok(())
}

pub fn unmarshal_map<'scope, 'partial, 'facet, 'shape: 'facet>(
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    partial: &'partial mut Partial<'facet, 'shape>,
    state: &mut UnmarshalState<'_, 'scope>,
) -> Result<&'partial mut facet_reflect::Partial<'facet, 'shape>, Error<'shape>> {
    let shape = partial.shape();
    let map = v8::Local::<v8::Map>::try_from(object).map_err(|_| Error::UnexpectedValue {
        shape,
        unexpected: object.type_repr(),
    })?;

    partial.begin_map()?;
    let array = map.as_array(scope);
    for i in 0..array.length() / 2 {
        let key = array.get_index(scope, i * 2).ok_or(Error::Exception)?;
        let value = array.get_index(scope, i * 2 + 1).ok_or(Error::Exception)?;
        super::unmarshal_value(scope, key, partial.begin_key()?, state)?.end()?;
        super::unmarshal_value(scope, value, partial.begin_value()?, state)?.end()?;
    }
    // Note: `begin_map()` does not push a frame.
    Ok(partial)
}
