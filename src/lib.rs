
pub mod component_meta;
mod tuples;
pub mod injector;
pub mod tags;
pub mod injection_binder;

pub type Injector = injector::Injector;
pub type InjectionBinder<T> = injection_binder::InjectionBinder<T>;

pub type Lazy<T> = std::sync::Arc<once_cell::sync::Lazy<T, Box<dyn FnOnce() -> T>>>;

impl<T: Clone + 'static> crate::component_meta::ComponentMeta for Lazy<T> {
    fn inject(injector: &crate::injector::Injector) -> anyhow::Result<Self> {
        use std::sync::Arc;
        let injector = injector.clone();
        let func: Box<dyn FnOnce() -> T + 'static> = Box::new(move || -> T {
            match injector.get::<T>() {
                Ok(x) => x,
                _ => unreachable!()
            }
        });
        let result = once_cell::sync::Lazy::new(func);
        Ok(Arc::new(result))
    }

    fn debug_line() -> Option<String> {
        None
    }

    fn dependencies_names() -> Vec<(std::any::TypeId, &'static str)> {
        vec![(std::any::TypeId::of::<T>(), std::any::type_name::<T>())]
    }

    fn lazy() -> bool {
        true
    }
}

#[macro_export]
macro_rules! erase {
   ( $pointer:ident < $dyn_type:ty > ) => ({
        |x| ->  $pointer< $dyn_type  > {
            $pointer::new(x)
        }
    })
}
