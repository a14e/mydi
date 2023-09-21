use crate::component_meta::ComponentMeta;
use crate::expander::ComponentExpander;
use crate::injector::Injector;
use crate::tuples::TupleInjectTypes;
use std::any::{type_name, Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

#[derive(Default)]
pub struct InjectionBinder<LastType> {
    static_values: HashMap<TypeId, Box<dyn Any + 'static>>,
    builders: Vec<(
        TypeId,
        Box<dyn Fn(&Injector) -> anyhow::Result<Box<dyn Any + 'static>>>,
    )>,

    requirements_graph: Vec<(TypeId, Vec<TypeId>)>,
    type_names: HashMap<TypeId, &'static str>,
    debug_lines: HashMap<TypeId, String>,

    lazy_types: HashSet<TypeId>,

    _phantom_data: PhantomData<LastType>,
}

impl InjectionBinder<()> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<LastType: Clone + 'static> InjectionBinder<LastType> {
    pub fn void(self) -> InjectionBinder<()> {
        self.change_type()
    }

    fn change_type<T>(self) -> InjectionBinder<T> {
        InjectionBinder {
            static_values: self.static_values,
            builders: self.builders,
            requirements_graph: self.requirements_graph,
            type_names: self.type_names,
            debug_lines: self.debug_lines,
            lazy_types: self.lazy_types,
            _phantom_data: PhantomData,
        }
    }

    pub fn expand<X: ComponentExpander>(self, expander: X) -> Self {
        expander.expand(self)
    }

    pub fn merge<OtherLast>(mut self, mut other: InjectionBinder<OtherLast>) -> Self {
        self.static_values
            .extend(mem::take(&mut other.static_values));
        self.builders.extend(mem::take(&mut other.builders));
        self.requirements_graph
            .extend(mem::take(&mut other.requirements_graph));
        self.type_names.extend(mem::take(&mut other.type_names));
        self.debug_lines.extend(mem::take(&mut other.debug_lines));
        self.lazy_types.extend(mem::take(&mut other.lazy_types));

        self
    }

    pub fn instance<X: Any + Clone + 'static>(mut self, x: X) -> Self {
        let type_id = TypeId::of::<X>();
        self.static_values.insert(type_id, Box::new(x));

        self.type_names.insert(type_id, type_name::<X>());
        self.requirements_graph.push((type_id.clone(), vec![]));

        self
    }

    pub fn inject<X: Any + ComponentMeta + Clone + 'static>(self) -> InjectionBinder<X> {
        self.inject_fn_raw::<X>(
            move |x: &Injector| -> anyhow::Result<X> {
                let result = X::inject(x)?;
                Ok(result)
            },
            X::dependencies_names(),
            X::debug_line(),
            X::lazy(),
        )
    }

    pub fn auto_box(self) -> Self {
        self.inject_fn(|(x,)| -> Box<LastType> { Box::new(x) })
            .change_type::<LastType>()
    }

    pub fn auto_arc(self) -> Self {
        self.inject_fn(|(x,)| -> Arc<LastType> { Arc::new(x) })
            .change_type::<LastType>()
    }

    pub fn auto<Din: Clone + 'static>(self, f: impl Fn(LastType) -> Din + 'static) -> Self {
        self.inject_fn(move |(x,)| -> Din { f(x) })
            .change_type::<LastType>()
    }

    pub fn inject_fn<R: Any + Clone + 'static, In: TupleInjectTypes, FN: Fn(In) -> R + 'static>(
        self,
        f: FN,
    ) -> InjectionBinder<R> {
        self.inject_fn_ok::<R, In, _>(move |tuple| Ok(f(tuple)))
    }

    pub fn inject_fn_ok<
        R: Any + Clone + 'static,
        In: TupleInjectTypes,
        FN: Fn(In) -> anyhow::Result<R> + 'static,
    >(
        self,
        f: FN,
    ) -> InjectionBinder<R> {
        let type_names = In::dependencies_names();
        self.inject_fn_raw(
            move |injector| -> anyhow::Result<R> {
                let tuple = In::read_from_injector(injector)?;
                f(tuple)
            },
            type_names,
            None,
            false,
        )
    }

    pub fn inject_fn_raw<X: Any + Clone + 'static>(
        mut self,
        f: impl Fn(&Injector) -> anyhow::Result<X> + 'static,
        dependencies_names: Vec<(TypeId, &'static str)>,
        debug_line: Option<String>,
        lazy: bool,
    ) -> InjectionBinder<X> {
        let func =
            Box::new(move |x: &Injector| -> anyhow::Result<Box<dyn Any>> { Ok(Box::new((f)(x)?)) });
        let type_id = TypeId::of::<X>();
        self.builders.push((type_id.clone(), func));

        let requirements = dependencies_names.iter().map(|(id, _)| *id).collect();
        self.requirements_graph.push((type_id, requirements));

        for (type_id, type_name) in dependencies_names {
            self.type_names.insert(type_id, type_name);
        }
        self.type_names.insert(type_id, type_name::<X>());

        if lazy {
            self.lazy_types.insert(type_id);
        }

        if let Some(debug_line) = debug_line {
            self.debug_lines.insert(type_id, debug_line);
        }

        self.change_type::<X>()
    }

    fn verify_missing_deps(
        &self,
        additional_types: &HashSet<TypeId>,
        short_types: bool,
    ) -> anyhow::Result<()> {
        let available_types: HashSet<_> = additional_types
            .iter()
            .chain(self.requirements_graph.iter().map(|(id, _)| id))
            .collect();
        let missing_local_types: Vec<(TypeId, &str, Vec<&str>)> = self
            .requirements_graph
            .iter()
            .flat_map(|(type_id, requirements)| {
                let missing_requirements: Vec<_> = requirements
                    .iter()
                    .filter(|x| !available_types.contains(x))
                    .flat_map(|x| self.type_names.get(x).cloned())
                    .collect();

                if missing_requirements.is_empty() {
                    None
                } else {
                    self.type_names
                        .get(type_id)
                        .map(|&name| (*type_id, name, missing_requirements))
                }
            })
            .collect();

        if missing_local_types.is_empty() {
            return Ok(());
        }
        // TODO move to func
        let mut message = String::new();
        message.push_str("Missing injection values:\n");
        for (type_id, value, requirements) in missing_local_types {
            message.push_str("for type ");
            message.push_str(make_name_shorter(value, short_types));
            if let Some(x) = self.debug_lines.get(&type_id) {
                message.push_str("\n at ");
                message.push_str(x.as_str());
            }
            message.push_str("\n");
            message.push_str("missing dependencies: ");
            message.push_str(make_name_shorter(&requirements[0], short_types));
            for r in requirements.iter().skip(1) {
                message.push_str(", ");
                message.push_str(make_name_shorter(r.trim(), short_types));
            }
            message.push_str("\n\n");
        }

        let err = anyhow::Error::msg(message);
        Err(err)
    }

    fn verify_nested_lazy_deps(&self, short_types: bool) -> anyhow::Result<()> {
        let invalid_lazy_types: Vec<_> = self
            .requirements_graph
            .iter()
            .filter(|(type_id, _)| self.lazy_types.contains(type_id))
            .filter(|(_, requirements)| requirements.iter().any(|x| self.lazy_types.contains(x)))
            .map(|(type_id, _)| *type_id)
            .collect();

        if invalid_lazy_types.is_empty() {
            return Ok(());
        }

        let recursive_names = invalid_lazy_types
            .into_iter()
            .flat_map(|x| self.type_names.get(&x))
            .map(|name| make_name_shorter(name, short_types));
        let recursive_names = join(recursive_names, ", ");
        let message = format!("Nested lazy dependencies: {recursive_names}");
        let err = anyhow::Error::msg(message);
        Err(err)
    }

    fn verify_recursive_deps(
        &self,
        additional_types: &HashSet<TypeId>,
        short_types: bool,
    ) -> anyhow::Result<()> {
        self.traverse_dependencies_and_verify_recursion(
            additional_types,
            short_types,
            |_| Ok(()),
            "Dependencies cycle (one or more) found :",
        )
    }

    fn traverse_dependencies_and_verify_recursion(
        &self,
        additional_types: &HashSet<TypeId>,
        short_types: bool,
        mut on_resolve: impl FnMut(TypeId) -> anyhow::Result<()>,
        err_message: &'static str,
    ) -> anyhow::Result<()> {
        let mut available_types: HashSet<_> = additional_types.iter().cloned().collect();

        let resolved_lazy_types: HashSet<_> = self
            .requirements_graph
            .iter()
            .map(|(k, v)| (*k, v))
            .filter(|(k, _)| self.lazy_types.contains(k))
            .map(|(type_id, _)| type_id)
            .collect();

        // Let's start by initializing lazy dependencies
        for lazy_type in resolved_lazy_types.iter().cloned() {
            available_types.insert(lazy_type);
            on_resolve(lazy_type)?;
        }

        // initializing lazy types at the very beginning
        let mut left_deps: HashMap<TypeId, &Vec<TypeId>> = self
            .requirements_graph
            .iter()
            .map(|(k, v)| (*k, v))
            .filter(|(k, _)| !resolved_lazy_types.contains(k))
            .collect();

        let mut current_len = left_deps.len();
        loop {
            // Resolving dependencies, at best works in O(n) time complexity, at worst in O(n^2)
            // In the most typical case, when DI represents an architecture
            // in the form of layers, it works roughly in O(n*k) time complexity, where k is the number of layers in the dependencies
            // and n is the number of dependencies.
            let resolved: Vec<TypeId> = left_deps
                .iter()
                .filter(|(_, deps)| deps.iter().all(|x| available_types.contains(x)))
                .map(|(type_id, _)| type_id.clone())
                .collect();

            for type_id in resolved {
                left_deps.remove(&type_id);
                available_types.insert(type_id);
                on_resolve(type_id)?;
            }

            // cycle found
            // if no changes in loop step there is a cycle
            if left_deps.len() == current_len {
                break;
            }
            current_len = left_deps.len();
            if left_deps.is_empty() {
                return Ok(());
            }
        }
        let cycle_type_names = left_deps
            .iter()
            .flat_map(|(type_id, _)| self.type_names.get(&type_id))
            .map(|name| make_name_shorter(name, short_types));
        let cycle_type_names = join(cycle_type_names, ", ");
        let err = anyhow::anyhow!("{err_message}{cycle_type_names}");
        Err(err)
    }

    fn verify_duplicates(
        &self,
        additional_types: &HashSet<TypeId>,
        short_types: bool,
    ) -> anyhow::Result<()> {
        let mut available: HashSet<_> = additional_types.clone();
        let mut duplicates: HashSet<TypeId> = Default::default();
        for (type_id, requirements) in &self.requirements_graph {
            if requirements.is_empty() {
                continue;
            }
            if !available.insert(*type_id) {
                duplicates.insert(*type_id);
            }
        }
        if duplicates.is_empty() {
            return Ok(());
        }

        let duplicates_names = duplicates
            .iter()
            .flat_map(|type_id| self.type_names.get(type_id))
            .map(|name| make_name_shorter(name, short_types));
        let duplicates_names = join(duplicates_names, ", ");
        let err = anyhow::anyhow!("Dependencies duplications found: {duplicates_names}");
        Err(err)
    }

    pub fn verify(
        &self,
        additional_types: HashSet<TypeId>,
        short_types: bool,
    ) -> anyhow::Result<()> {
        let additional_deps: HashSet<_> = additional_types
            .into_iter()
            .chain(self.static_values.iter().map(|(type_id, _)| *type_id))
            .collect();

        self.verify_duplicates(&additional_deps, short_types)?;
        self.verify_nested_lazy_deps(short_types)?;
        self.verify_missing_deps(&additional_deps, short_types)?;
        // Order is important, as the recursion check will also find fields where dependencies are missing
        self.verify_recursive_deps(&additional_deps, short_types)?;

        Ok(())
    }

    pub fn build(mut self) -> anyhow::Result<Injector> {
        let initial_known_deps: HashSet<_> = {
            self.static_values
                .iter()
                .map(|(type_id, _)| *type_id)
                .collect()
        };

        self.verify(initial_known_deps.clone(), false)?;

        let injector = Injector::new(mem::take(&mut self.static_values));

        let builders_map: HashMap<_, _> = mem::take(&mut self.builders).into_iter().collect();

        self.traverse_dependencies_and_verify_recursion(
            &initial_known_deps,
            false,
            |type_id| {
                if let Some(builder) = builders_map.get(&type_id) {
                    let item = (builder)(&injector)?;
                    injector.insert(type_id, item);
                }
                Ok(())
            },
            "Can't resolve dependencies of types :",
        )?;
        Ok(injector)
    }
}

fn join<T, IT>(mut iter: IT, separator: &str) -> String
where
    T: std::fmt::Display,
    IT: Iterator<Item = T>,
{
    use std::fmt::Write;

    let mut result = String::new();

    for x in iter.next().into_iter() {
        write!(&mut result, "{}", x).unwrap();
    }

    for x in iter {
        result.push_str(separator);
        write!(&mut result, "{}", x).unwrap();
    }
    result
}

fn make_name_shorter(name: &str, short_types: bool) -> &str {
    if short_types {
        let mut openned_generics = 0;
        let mut found_idx = None;

        for (idx, char) in name.chars().rev().enumerate() {
            match char {
                '>' => {
                    openned_generics += 1;
                }
                '<' => {
                    openned_generics -= 1;
                }
                ':' => {
                    if openned_generics == 0 {
                        found_idx = Some(idx);
                        break;
                    }
                }
                _ => {}
            }
        }

        match found_idx {
            Some(idx_from_right) => {
                let index = name.len() - idx_from_right;
                &name[index..]
            }
            None => name,
        }
    } else {
        name
    }
}
