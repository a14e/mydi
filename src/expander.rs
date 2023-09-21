use crate::InjectionBinder;

pub trait ComponentExpander {
    fn expand<T: Clone + 'static>(self, injector: InjectionBinder<T>) -> InjectionBinder<T>;
}
