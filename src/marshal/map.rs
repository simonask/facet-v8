use facet_reflect::PeekMap;

use super::{Error, MarshalState};

pub fn serialize_map_into<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
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
