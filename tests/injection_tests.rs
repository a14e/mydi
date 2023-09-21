use std::marker::PhantomData;
use std::sync::Arc;
use dyn_clone::DynClone;
use mydi::{erase, Component, InjectionBinder};
use mydi_macros::ExpandComponent;

#[test]
fn resolve_simple_values() {
    #[derive(Component, Clone)]
    struct A {
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        a: A,
    }
    #[derive(Component, Clone)]
    struct C {
        b: B,
    }

    let inject = InjectionBinder::new()
        .instance(1u32)
        .inject::<A>()
        .inject::<B>()
        .inject::<C>()
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap().b.a.x;
    assert_eq!(x, 1)
}

#[test]
fn default_value() {
    #[derive(Component, Clone)]
    struct A {
        #[component(default)]
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        a: A,
    }
    #[derive(Component, Clone)]
    struct C {
        b: B,
    }

    let inject = InjectionBinder::new()
        .inject::<A>()
        .inject::<B>()
        .inject::<C>()
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap().b.a.x;
    assert_eq!(x, 0)
}

#[test]
fn default_value_func() {
    fn func() -> u32 {
        2
    }

    #[derive(Component, Clone)]
    struct A {
        #[component(default = func)]
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        a: A,
    }
    #[derive(Component, Clone)]
    struct C {
        b: B,
    }

    let inject = InjectionBinder::new()
        .inject::<A>()
        .inject::<B>()
        .inject::<C>()
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap().b.a.x;
    assert_eq!(x, 2)
}


#[test]
fn resolve_empty_structs() {
    #[derive(Component, Clone)]
    struct A {
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {}
    #[derive(Component, Clone)]
    struct C;

    let inject = InjectionBinder::new()
        .instance(1u32)
        .inject::<A>()
        .inject::<B>()
        .inject::<C>()
        .build()
        .unwrap();

    let x = inject.get::<A>().unwrap().x;
    assert_eq!(x, 1);

    inject.get::<B>().unwrap();
    inject.get::<C>().unwrap();
}


#[test]
fn resolve_simple_generic_values() {
    #[derive(Component, Clone)]
    struct A<T: Clone + 'static> {
        x: T,
    }

    let inject = InjectionBinder::new()
        .instance(1u32)
        .instance(2u64)
        .inject::<A<u32>>()
        .inject::<A<u64>>()
        .build()
        .unwrap();

    let x = inject.get::<A<u32>>().unwrap().x;
    assert_eq!(x, 1);
    let x = inject.get::<A<u64>>().unwrap().x;
    assert_eq!(x, 2);
}

#[test]
fn resolve_simple_value_reverse_order() {
    #[derive(Component, Clone)]
    struct A {
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        a: A,
    }
    #[derive(Component, Clone)]
    struct C {
        b: B,
    }

    let inject = InjectionBinder::new()
        .inject::<C>()
        .inject::<B>()
        .inject::<A>()
        .instance(1u32)
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap().b.a.x;
    assert_eq!(x, 1)
}


#[test]
fn resolve_values_with_multiple_deps() {
    #[derive(Component, Clone)]
    struct A {
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        _a: A,
        x: u32,
    }
    #[derive(Component, Clone)]
    struct C {
        _b: B,
        _a: A,
        x: u64,
    }

    let inject = InjectionBinder::new()
        .inject::<C>()
        .inject::<B>()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64)
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap();
    assert_eq!(x._b._a.x, 1);
    assert_eq!(x._b.x, 1);
    assert_eq!(x.x, 2);
}

#[test]
fn resolve_auto_boxing() {
    #[derive(Component, Clone)]
    struct A {
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        a: Box<A>,
    }
    #[derive(Component, Clone)]
    struct C {
        b: B,
    }

    let inject = InjectionBinder::new()
        .instance(1u32)
        .inject::<A>().auto_box()
        .inject::<B>()
        .inject::<C>()
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap().b.a.x;
    assert_eq!(x, 1)
}

#[test]
fn resolve_cyclic_dependencies() {
    #[derive(Component, Clone)]
    struct A {
        x: u32,
        c: mydi::Lazy<C>,
    }
    #[derive(Component, Clone)]
    struct B {
        a: A,
        x: u32,
    }
    #[derive(Component, Clone)]
    struct C {
        b: B,
        _a: A,
        x: u64,
    }

    let binder1 = InjectionBinder::new()
        .inject::<C>()
        .inject::<mydi::Lazy<C>>()
        .inject::<B>();
    let binder2 = InjectionBinder::new()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64);

    let inject = (binder1.merge(binder2))
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap();
    assert_eq!(x.b.a.x, 1);
    assert_eq!(x.b.x, 1);
    assert_eq!(x.x, 2);
    assert_eq!(x.b.a.c.x, 2);
}

#[test]
fn resolve_values_with_multiple_binders() {
    #[derive(Component, Clone)]
    struct A {
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        _a: A,
        x: u32,
    }
    #[derive(Component, Clone)]
    struct C {
        _b: B,
        _a: A,
        x: u64,
    }

    let binder1 = InjectionBinder::new()
        .inject::<C>()
        .inject::<B>();
    let binder2 = InjectionBinder::new()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64);

    let inject = (binder1.merge(binder2))
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap();
    assert_eq!(x._b._a.x, 1);
    assert_eq!(x._b.x, 1);
    assert_eq!(x.x, 2);
}

#[test]
fn read_tuples() {
    #[derive(Component, Clone)]
    struct A {
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        _a: A,
        x: u32,
    }
    #[derive(Component, Clone)]
    struct C {
        _b: B,
        _a: A,
        x: u64,
    }

    let binder1 = InjectionBinder::new()
        .inject::<C>()
        .inject::<B>();
    let binder2 = InjectionBinder::new()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64);

    let inject = (binder1.merge(binder2))
        .build()
        .unwrap();

    let (_, _, x) = inject.get_tuple::<(A, B, C)>().unwrap();
    assert_eq!(x._b._a.x, 1);
    assert_eq!(x._b.x, 1);
    assert_eq!(x.x, 2);
}

#[test]
fn resolve_values_with_multiple_deps_tuple_init() {
    #[derive(Component, Clone)]
    struct A {
        _x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        _a: A,
        _x: u32,
    }
    #[derive(Component, Clone)]
    struct C {
        _b: B,
        _a: A,
        _x: u64,
    }

    let inject = InjectionBinder::new()
        .inject_fn(|(_b, _a, _x)| C { _b, _a, _x })
        .inject::<B>()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64)
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap();
    assert_eq!(x._b._a._x, 1);
    assert_eq!(x._b._x, 1);
    assert_eq!(x._x, 2);
}

#[test]
fn fail_on_duplicate_values() {
    #[derive(Component, Clone)]
    struct A {
        _x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        _a: A,
        _x: u32,
    }
    #[derive(Component, Clone)]
    struct DuplicateStruct {
        _b: B,
        _a: A,
        _x: u64,
    }

    let inject_res = InjectionBinder::new()
        .inject::<DuplicateStruct>()
        .inject::<DuplicateStruct>()
        .inject::<B>()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64)
        .build();

    assert!(inject_res.is_err());
    let err_string = inject_res.err().unwrap().to_string();
    assert!(err_string.contains("Dependencies duplications found"));
    assert!(err_string.contains("DuplicateStruct"));
}

#[test]
fn fail_on_duplicate_missing_values() {
    #[derive(Component, Clone)]
    struct A {
        _x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        _a: A,
        _x: u32,
    }
    #[derive(Clone)]
    struct MissingDep {}
    #[derive(Component, Clone)]
    struct StructWithoutDep {
        _b: B,
        _a: A,
        _x: u64,
        _missing: MissingDep,
    }

    let inject_res = InjectionBinder::new()
        .inject::<StructWithoutDep>()
        .inject::<B>()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64)
        .build();

    assert!(inject_res.is_err());
    let err_string = inject_res.err().unwrap().to_string();
    assert!(err_string.contains("Missing injection values"));
    assert!(err_string.contains("StructWithoutDep"));
    assert!(err_string.contains("MissingDep"));
}


#[test]
fn fail_on_missing_lazy_values() {
    #[derive(Component, Clone)]
    struct MissingStruct {
        _x: u32,
    }

    let inject_res = InjectionBinder::new()
        .inject::<mydi::Lazy<MissingStruct>>()
        .instance(1u32)
        .build();

    assert!(inject_res.is_err());
    let err_string = inject_res.err().unwrap().to_string();
    assert!(err_string.contains("Missing injection values"));
    assert!(err_string.contains("MissingStruct"));
}

#[test]
fn fail_on_nested_lazy_values() {
    #[derive(Component, Clone)]
    struct MissingStruct {
        _x: u32,
    }

    let inject_res = InjectionBinder::new()
        .inject::<mydi::Lazy<MissingStruct>>()
        .inject::<mydi::Lazy<mydi::Lazy<MissingStruct>>>()
        .inject::<MissingStruct>()
        .instance(1u32)
        .build();

    assert!(inject_res.is_err());
    let err_string = inject_res.err().unwrap().to_string();
    assert!(err_string.contains("Nested lazy dependencies"));
    assert!(err_string.contains("MissingStruct"));
}

#[test]
fn work_with_arc_dyn_traits() {
    #[derive(Component, Clone)]
    pub struct A {
        x: u32,
    }

    trait Test {
        fn x(&self) -> u32;
    }
    impl Test for A {
        fn x(&self) -> u32 {
            self.x
        }
    }

    let inject_res = InjectionBinder::new()
        .inject::<A>().auto(erase!(Arc<dyn Test>))
        .instance(1u32)
        .build()
        .unwrap();
    let dyn_type = inject_res.get::<Arc<dyn Test>>().unwrap();
    assert_eq!(dyn_type.x(), 1u32)
}

#[test]
fn work_with_box_dyn_traits() {
    #[derive(Component, Clone)]
    pub struct A {
        x: u32,
    }

    trait Test: DynClone {
        fn x(&self) -> u32;
    }

    dyn_clone::clone_trait_object!(Test);

    impl Test for A {
        fn x(&self) -> u32 {
            self.x
        }
    }

    let inject_res = InjectionBinder::new()
        .inject::<A>().auto(erase!(Box<dyn Test>))
        .instance(1u32)
        .build()
        .unwrap();
    let dyn_type = inject_res.get::<Box<dyn Test>>().unwrap();
    assert_eq!(dyn_type.x(), 1u32)
}

#[test]
fn fail_on_duplicate_missing_values_with_tuple_init() {
    #[derive(Component, Clone)]
    struct A {
        _x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        _a: A,
        _x: u32,
    }
    #[derive(Clone)]
    struct MissingDep {}
    #[derive(Component, Clone)]
    struct StructWithoutDep {
        _b: B,
        _a: A,
        _x: u64,
        _missing: MissingDep,
    }

    let inject_res = InjectionBinder::new()
        .inject_fn(|(_b, _a, _x, _missing)| StructWithoutDep { _b, _a, _x, _missing })
        .inject::<B>()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64)
        .build();

    assert!(inject_res.is_err());
    let err_string = inject_res.err().unwrap().to_string();
    assert!(err_string.contains("Missing injection values"));
    assert!(err_string.contains("StructWithoutDep"));
    assert!(err_string.contains("MissingDep"));
}

#[test]
fn fail_on_cyclic_dep() {
    #[derive(Component, Clone)]
    struct A {
        _x: u32,
        _cycle: Box<CycleStart>,
    }
    #[derive(Component, Clone)]
    struct B {
        _a: A,
        _x: u32,
    }
    #[derive(Component, Clone)]
    struct ShouldNotBeInErr {
        _x: u32,
    }
    #[derive(Component, Clone)]
    struct CycleStart {
        _b: B,
        _a: A,
        _x: u64,
        _d: ShouldNotBeInErr,
    }

    let inject_res = InjectionBinder::new()
        .inject::<Box<CycleStart>>()
        .inject::<ShouldNotBeInErr>()
        .inject::<B>()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64)
        .build();

    assert!(inject_res.is_err());
    let err_string = inject_res.err().unwrap().to_string();
    assert!(err_string.contains("Dependencies cycle (one or more) found"));
    assert!(err_string.contains("CycleStart"));
    assert!(err_string.contains("A"));
    assert!(!err_string.contains("ShouldNotBeInErr"));
}


#[test]
fn should_work_with_expansion() {
    #[derive(ExpandComponent)]
    struct A {
        x: u32,
    }

    let inject = InjectionBinder::new()
        .expand(A { x: 1 })
        .build()
        .unwrap();

    let x = inject.get::<u32>().unwrap();
    assert_eq!(x, 1);
}


#[test]
fn should_work_with_expansion_with_multiple_fields() {
    #[derive(ExpandComponent)]
    struct A {
        x: u32,
        y: u64,
    }

    let inject = InjectionBinder::new()
        .expand(A { x: 1, y: 2 })
        .build()
        .unwrap();

    let x = inject.get::<u32>().unwrap();
    assert_eq!(x, 1);
    let y = inject.get::<u64>().unwrap();
    assert_eq!(y, 2);
}


#[test]
fn should_work_with_expansion_with_ignore() {
    #[derive(ExpandComponent)]
    struct A {
        x: u32,
        #[ignore_expansion]
        _y: u64,
    }

    let inject = InjectionBinder::new()
        .expand(A { x: 1, _y: 2 })
        .build()
        .unwrap();

    let x = inject.get::<u32>().unwrap();
    assert_eq!(x, 1);
    let y = inject.get::<u64>();
    assert_eq!(y.is_err(), true);
}


#[test]
fn should_work_with_generics() {
    #[derive(ExpandComponent)]
    struct A<T> {
        x: u32,
        #[ignore_expansion]
        _p: PhantomData<T>,
    }

    let inject = InjectionBinder::new()
        .expand(A::<()> { x: 1, _p: PhantomData })
        .build()
        .unwrap();

    let x = inject.get::<u32>().unwrap();
    assert_eq!(x, 1);
    let y = inject.get::<PhantomData<()>>();
    assert_eq!(y.is_err(), true);
}