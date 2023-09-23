use std::sync::Arc;
use crate::InjectionBinder;

pub trait ComponentExpander {
    fn expand<T: Clone + 'static>(self, injector: InjectionBinder<T>) -> InjectionBinder<T>;
}


impl<INNER: ComponentExpander> ComponentExpander for Box<INNER> {
    fn expand<T: Clone + 'static>(self, injector: InjectionBinder<T>) -> InjectionBinder<T> {
        let unboxed =  *self;
        unboxed.expand(injector)
    }
}

impl<INNER: ComponentExpander + Clone> ComponentExpander for Arc<INNER> {
    fn expand<T: Clone + 'static>(self, injector: InjectionBinder<T>) -> InjectionBinder<T> {
        let unboxed =  (*self).clone();
        unboxed.expand(injector)
    }
}