use crate::component_meta::ComponentMeta;
use crate::injector::Injector;
use std::any::TypeId;
use std::marker::PhantomData;

pub struct Tagged<T, Tag> {
    x: T,
    _phantom: PhantomData<Tag>,
}

impl<T> Tagged<T, ()> {
    pub fn pure(x: T) -> Self {
        Tagged::new(x)
    }
}

impl<T, Tag> Tagged<T, Tag> {
    pub fn new(x: T) -> Self {
        Self {
            x,
            _phantom: PhantomData,
        }
    }

    pub fn tagged<NewTag>(self) -> Tagged<T, NewTag> {
        Tagged::new(self.x)
    }

    pub fn untag(self) -> T {
        self.x
    }
}

impl<T: ComponentMeta, Tag> ComponentMeta for Tagged<T, Tag> {
    fn inject(injector: &Injector) -> anyhow::Result<Self> {
        let res = T::inject(injector)?;
        let res = Self::new(res);
        Ok(res)
    }

    fn debug_line() -> Option<String> {
        T::debug_line()
    }

    fn dependencies_names() -> Vec<(TypeId, &'static str)> {
        T::dependencies_names()
    }
}

impl<T: Clone, Tag> Clone for Tagged<T, Tag> {
    fn clone(&self) -> Self {
        Tagged::new(self.x.clone())
    }
}

impl<T, Tag> std::ops::Deref for Tagged<T, Tag> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.x
    }
}
