use std::any::{type_name, TypeId};
use crate::injector::Injector;

pub trait TupleInjectTypes where Self: Sized {
    fn read_from_injector(injector: &Injector) -> anyhow::Result<Self>;
    fn dependencies_names() -> Vec<(TypeId, &'static str)>;
}

impl TupleInjectTypes for () {
    fn read_from_injector(_: &Injector) -> anyhow::Result<Self> {
        Ok(())
    }

    fn dependencies_names() -> Vec<(TypeId, &'static str)> {
        vec![]
    }
}

impl<Arg1: std::clone::Clone + 'static> TupleInjectTypes for (Arg1, ) {
    fn read_from_injector(injector: &Injector) -> anyhow::Result<Self> {
        Ok((injector.get::<Arg1>()?, ))
    }

    fn dependencies_names() -> Vec<(TypeId, &'static str)> {
        vec![(TypeId::of::<Arg1>(), type_name::<Arg1>())]
    }
}

macro_rules! build_tuple_injector {
    ($($tuple_type:ident),*) => {
        impl<$($tuple_type: std::clone::Clone + 'static),*> TupleInjectTypes for ($($tuple_type),* ) {
           fn read_from_injector(injector: &Injector) -> anyhow::Result<Self> {
               Ok(($(injector.get::<$tuple_type>()?),* ))
           }

           fn dependencies_names() -> Vec<(TypeId, &'static str)> {
               vec![$( (TypeId::of::<$tuple_type>(), type_name::<$tuple_type>()) ),*]
           }
       }
    }
}

build_tuple_injector!(Arg1, Arg2);
build_tuple_injector!(Arg1, Arg2, Arg3);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Arg13);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Arg13, Arg14);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Arg13, Arg14, Arg15);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Arg13, Arg14, Arg15, Arg16);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Arg13, Arg14, Arg15, Arg16, Arg17);
build_tuple_injector!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6, Arg7, Arg8, Arg9, Arg10, Arg11, Arg12, Arg13, Arg14, Arg15, Arg16, Arg17, Arg18);

