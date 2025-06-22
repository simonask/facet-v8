use facet_core::Def;
use facet_reflect::{Partial, Peek};

use super::{Error, MarshalState, UnmarshalState};

pub fn marshal_set_into<'mem, 'facet: 'mem, 'shape: 'facet, 'scope>(
    peek: Peek<'mem, 'facet, 'shape>,
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    state: &mut MarshalState<'mem, 'scope, '_, '_>,
) -> Result<(), Error<'shape>> {
    let shape = peek.shape();
    let Def::Set(def) = shape.def else {
        panic!("expected a SetDef");
    };
    let peek = PeekSet { value: peek, def };

    let set =
        v8::Local::<v8::Set>::try_from(object).expect("object constructor did not create a set");
    for item in peek.iter() {
        let item_value = super::marshal_value(item, scope, state, None)?;
        set.add(scope, item_value).ok_or(Error::Exception)?;
    }
    Ok(())
}

pub fn unmarshal_set<'scope, 'partial, 'facet, 'shape: 'facet>(
    scope: &mut v8::HandleScope<'scope>,
    object: v8::Local<'scope, v8::Object>,
    partial: &'partial mut Partial<'facet, 'shape>,
    state: &mut UnmarshalState<'_, 'scope>,
) -> Result<&'partial mut Partial<'facet, 'shape>, Error<'shape>> {
    let shape = partial.shape();
    let set = v8::Local::<v8::Set>::try_from(object).map_err(|_| Error::UnexpectedValue {
        shape,
        unexpected: object.type_repr(),
    })?;

    let array = set.as_array(scope);
    partial.begin_list()?;
    for i in 0..array.length() {
        let item = array.get_index(scope, i).ok_or(Error::Exception)?;
        super::unmarshal_value(scope, item, partial.begin_list_item()?, state)?.end()?;
    }
    // Note: `begin_list()` does not push a frame.
    Ok(partial)
}

// TODO: This is missing from `facet`.
#[derive(Clone, Copy)]
struct PeekSet<'mem, 'facet, 'shape> {
    value: Peek<'mem, 'facet, 'shape>,
    def: facet_core::SetDef<'shape>,
}

impl<'mem, 'facet, 'shape> PeekSet<'mem, 'facet, 'shape> {
    pub fn iter(self) -> PeekSetIter<'mem, 'facet, 'shape> {
        let iter_init_with_value_fn = self.def.vtable.iter_vtable.init_with_value.unwrap();
        let iter = unsafe { iter_init_with_value_fn(self.value.data().thin().unwrap()) };
        PeekSetIter { set: self, iter }
    }
}

struct PeekSetIter<'mem, 'facet, 'shape> {
    set: PeekSet<'mem, 'facet, 'shape>,
    iter: facet_core::PtrMut<'mem>,
}

impl<'mem, 'facet, 'shape> Iterator for PeekSetIter<'mem, 'facet, 'shape> {
    type Item = Peek<'mem, 'facet, 'shape>;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = (self.set.def.vtable.iter_vtable.next)(self.iter);
            let shape = (self.set.def.t)();
            next.map(|ptr| Peek::unchecked_new(ptr, shape))
        }
    }
}

impl<'mem, 'facet, 'shape> Drop for PeekSetIter<'mem, 'facet, 'shape> {
    fn drop(&mut self) {
        unsafe {
            (self.set.def.vtable.iter_vtable.dealloc)(self.iter);
        }
    }
}
