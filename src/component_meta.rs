use std::rc::Rc;
use std::sync::Arc;
use crate::injector::Injector;

pub trait ComponentMeta where Self: Sized {
    fn inject(injector: &Injector) -> anyhow::Result<Self> ;

    // to simplify find pos of structures
    fn debug_line() -> Option<String>;

    fn dependencies_names() -> Vec<(std::any::TypeId, &'static str)>;

    // very unsafe
    // use carefully
    fn lazy() -> bool {
        false
    }
}

impl<Inner> ComponentMeta for Box<Inner>
    where Inner: ComponentMeta {

    fn inject(injector: &Injector) -> anyhow::Result<Self> {
        let result = Inner::inject(injector)?;
        let result = Box::new(result);
        Ok(result)
    }

    // to simplify find pos of structures
    fn debug_line() -> Option<String> {
        Inner::debug_line()
    }

    fn dependencies_names() -> Vec<(std::any::TypeId, &'static str)> {
        Inner::dependencies_names()
    }
}

impl <Inner> ComponentMeta for Rc<Inner>
    where Inner: ComponentMeta {

    fn inject(injector: &Injector) -> anyhow::Result<Self> {
        let result = Inner::inject(injector)?;
        let result = Rc::new(result);
        Ok(result)
    }

    // to simplify find pos of structures
    fn debug_line() -> Option<String> {
        Inner::debug_line()
    }

    fn dependencies_names() -> Vec<(std::any::TypeId, &'static str)> {
        Inner::dependencies_names()
    }
}

impl<Inner> ComponentMeta for Arc<Inner>
    where Inner: ComponentMeta {

    fn inject(injector: &Injector) -> anyhow::Result<Self>  {
        let result = Inner::inject(injector)?;
        let result = Arc::new(result);
        Ok(result)
    }

    // to simplify find pos of structures
    fn debug_line() -> Option<String> {
        Inner::debug_line()
    }

    fn dependencies_names() -> Vec<(std::any::TypeId, &'static str)> {
        Inner::dependencies_names()
    }
}