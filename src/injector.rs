use crate::tuples::TupleInjectTypes;
use anyhow::anyhow;
use parking_lot::RwLock;
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct Injector {
    values: Arc<RwLock<HashMap<TypeId, Box<dyn Any>>>>,
}

impl Injector {
    pub(crate) fn new(values: HashMap<TypeId, Box<dyn Any + 'static>>) -> Self {
        let res = Self::default();
        *res.values.write() = values;
        res
    }

    pub(crate) fn insert(&self, type_id: TypeId, item: Box<dyn Any + 'static>) {
        self.values.write().insert(type_id, item);
    }

    pub fn get<X: Clone + 'static>(&self) -> anyhow::Result<X> {
        let type_id = TypeId::of::<X>();
        self.values
            .read()
            .get(&type_id)
            .and_then(|x| x.as_ref().downcast_ref::<X>())
            .cloned()
            .ok_or_else(|| {
                let type_name = type_name::<X>();
                anyhow!("Missing value of type {type_name}")
            })
    }

    pub fn get_tuple<Tuple: TupleInjectTypes>(&self) -> anyhow::Result<Tuple> {
        Tuple::read_from_injector(self)
    }
}
