# My DI / Dependency Injection Library

## Brief Description and Main Features

A Rust Dependency Injection (DI) library focused on simplicity and composability. Key features include:

* Simple design using macros
* Support for cyclic dependencies
* Support for arbitrary initialization order
* Working with dyn traits
* Ability to use multiple structures with the same types through tagging
* Usage of default traits and arbitrary functions for default arguments

This library enables efficient management of complex projects with numerous nested structures by assembling 
various application components, such as configurations, database connections, payment service clients, 
Kafka connections, and more. My DI keeps dependency management organized, easy to read, and expandable,
providing a solid foundation for the growth of your project.

## How to connect the library?
Simply add the dependency to your Cargo.toml:

```toml
[dependencies]
mydi = "0.1.0"
```

## So, what's the problem? Why do I need this?

Approaches using separate mechanisms for DI are common in other languages like Java and Scala, but not as widespread in Rust.
To understand the need for this library, let's look at an example without My DI and one with it. 
Let's build several structures (Rust programs sometimes consist of hundreds of nested structures) in plain Rust.

### The Problem!

```rust
struct A {
    x: u32
}

impl A {
    pub fn new(x: u32) -> Self {
        Self { x }
    }
}

struct B {
    x: u64
}

impl A {
    pub fn new(x: u64) -> Self {
        Self { x }
    }
}

struct C {
    x: f32
}

impl A {
    pub fn new(x: f32) -> Self {
        Self { x }
    }
}

struct D {
    a: A,
    b: B,
    c: C
}

impl D {
    pub fn new(a: A,
               b: B,
               c: C) -> Self {
        Self { a, b, c }
    }
    pub fn run(self) {
        todo!()
    }
}

fn main() {
    let a = A::new(1);
    let b = B::new(2);
    let c = C::new(3f64);
    let d = D::new(a, b, c);
    d.run()
}
```

As you can see, we write each argument in at least 4 places:

* in the struct declaration,
* in the constructor arguments,
* in the structure fields in the constructor,
* and then also substitute the arguments in the constructor.
And as the project grows, all of this will become more complex and confusing.

### The Solution!

Now let's try to simplify all this with My DI:

```rust
use mydi_macros::Component;
use mydi::InjectionBinder;

#[derive(Component, Clone)]
struct A {
    x: u32
}

#[derive(Component, Clone)]
struct B {
    x: u64
}

#[derive(Component, Clone)]
struct C {
    x: f32
}

#[derive(Component, Clone)]
struct D {
    a: A,
    b: B,
    c: C
}

impl D {
    pub fn run(self) {
        todo!()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let injector = InjectionBinder::new()
        .instance(1u32)
        .instance(2u64)
        .instance(3f32)
        .inject::<A>()
        .inject::<B>()
        .inject::<C>()
        .inject::<D>()
        .build()?;
    let d: D = injector.get()?;
    d.run()
}
```

As a result, we reduced the amount of code, removed unnecessary duplication, and left only the essential code. We also
opened ourselves up to further code refactoring (which we will discuss in the following sections):

1. We can now separate the structure building across different files and not drag them into a single one; for example, 
we can separately assemble configurations, database work, payment service clients, Kafka connections, etc.
2. We can assemble them in any order, not just in the order of initialization, 
which means we don't have to keep track of what was initialized first.
3. We can work with cyclic dependencies.

# Testing Dependencies

The library resolves dependencies at runtime, as otherwise, it would be impossible to implement features like cyclic dependencies and arbitrary initialization order. 
This means that dependency resolution needs to be checked somehow, and for this purpose, a test should be added.
This is done very simply. To do this, you just need to call the verify method. 
In general, it's enough to call it after the final assembly of dependencies.
For example, like this:

```rust
fn build_dependencies(config: MyConfig) -> InjectionBinder<()> {
    todo!()
}

#[cfg(test)]
mod test_dependencies_building {
    use std::any::TypeId;
    use sea_orm::DatabaseConnection;
    use crate::{build_dependencies, config};
    use std::collections::HashSet;

    #[test]
    fn test_dependencies() {
        let cfg_path = "./app_config.yml";
        let app_config = config::parse_config(&cfg_path).unwrap();
        let modules = build_dependencies(app_config);
        let initial_types = HashSet::from([  // types that will be resolved somewhere separately, but for the purposes of the test, we add them additionally
            TypeId::of::<DatabaseConnection>(),
            TypeId::of::<reqwest::Client>()
        ]);
        // the argument true means that in the errors, we will display not the full names of the structures, but only the final ones
        // if you are interested in the full ones, you should pass false instead
        modules.verify(initial_types, true).unwrap();
    }
}
```

# Modular Architecture and Composition

## Организация проекта

How to organize a project with many dependencies? It may depend on your preferences, 
but I prefer the following folder structure:

```
- main.rs
- modules
-- mod.rs
-- dao.rs
-- clients.rs
-- configs.rs
-- controllers.rs
-- services.rs
-- ...
```

This means that there is a separate folder with files for assembling dependencies, each responsible for its own 
set of services in terms of functionality. 
Alternatively, if you prefer, you can divide the services not by functional purpose, but by domain areas:

```
- main.rs
- modules
  -- mod.rs
  -- users.rs
  -- payments.rs
  -- metrics.rs
  -- ...
```

Both options are correct and will work, and which one to use is more a matter of taste. 
In each module, its own `InjectionBinder` will be assembled, and in main.rs, there will be something like:

```rust

#[derive(mydi::Component, Clone)]
struct MyApp {}

impl MyApp {
    fn run(&self) {
        todo!()
    }
}

fn merge_dependencies() -> Result<InjectionBinder<()>> {
    let result = modules::dao::build_dependencies()
        .merge(modules::configs::build_dependencies()?) // Of course, during dependency assembly, something might fail
        .merge(modules::clients::build_dependencies())
        .merge(modules::services::build_dependencies())
        .merge(modules::controllers::build_dependencies())
        .inject::<MyApp>();
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let injector = merge_dependencies().build()?;
    let app: MyApp = injector.get()?;
    app.run();
    Ok(())
}

```

## Organizing a separate module
So, how will the modules themselves look? This may also depend on personal preferences. I prefer to
use configurations as specific instances.
```rust
pub fn build_dependencies(app_config_path: &str,
                          kafka_config_path: &str) -> Result<InjectionBinder<()>> {
    let app_config = AppConfig::parse(app_config_path)?;
    let kafka_config = KafkaConfig::parse(kafka_config_path)?;
    let result = InjectionBinder::new()
        .instance(app_config)
        .instance(kafka_config)
        // ...
        .void();
    
    Ok(result)
}
```
Meanwhile, the module for controllers might be assembled like this:
```rust
pub fn build_dependencies() -> InjectionBinder<()> {
    InjectionBinder::new()
        .inject::<UsersController>()
        .inject::<PaymentsController>()
        .inject::<OrdersController>()
        // ...
        .void()
}
```

Note the `.void()` at the end. After each component is added to the `InjectionBinder`, it changes its internal
type to the one that was passed. Therefore, to simplify working with types, it makes sense to convert to the type `()`,
and that's what the `.void()` method is used for.

# Adding Dependencies Using Macros

To add dependencies, the best way is to use the derive macro Component:

```rust
#[derive(Component, Clone)]
struct A {
    x: u32,
    y: u16,
    z: u8,
}
```

It will generate the necessary `ComponentMeta` macro, and after that, you can add dependencies through the inject method:

```rust
fn main() {
    let injector = InjectionBinder::new()
        .instance(1u32)
        .instance(2u16)
        .instance(3u8)
        .inject::<A>()
        .build();
    todo!()
}
```


# Adding Dependencies Using Functions

In some cases, using macros may be inconvenient, so it makes sense to use functions instead. 
For this, use the `inject_fn` method:
```rust
 #[derive(Component, Clone)]
    struct A {
        x: u32,
    }
    #[derive(Component, Clone)]
    struct B {
        a: A,
        x: u32,
    }

    #[derive(Clone)]
    struct C {
        b: B,
        a: A,
        x: u64,
    }

fn main() {
    let inject = InjectionBinder::new()
        .inject_fn(|(b, a, x)| C { b, a, x })
        .inject::<B>()
        .inject::<A>()
        .instance(1u32)
        .instance(2u64)
        .build()
        .unwrap();

    let x = inject.get::<C>().unwrap(); 
}
```
Take note of the parentheses in the arguments. The argument here accepts a tuple. Therefore, for 0 arguments, you need to write
the arguments like this `|()|`, and for a single argument, you need to write the tuple in this form `|(x, )|`.

# Default Arguments

To add a default value, you can use the directive `#[component(...)]`.
Currently, there are only 2 available options: `#[component(default)]` and `#[component(default = my_func)]`,
where my_func is a function in the scope. `#[component(default)]` will substitute the value as
`Default::default()`
For example, like this:

```rust
#[derive(Component, Clone)]
struct A {
    #[component(default)]
    x: u32,
    #[component(default = custom_default)]
    y: u16,
    z: u8,
}

fn custom_default() -> u16 {
    todo!()
}
```

Note that custom_default is called without parentheses `()`. Also, at the moment, calls from nested modules
are not supported, meaning `foo::bar::custom_default` will not work. To work around this limitation,
simply use `use` to bring the function call into scope.

# How to read values?

As a result of dependency assembling, an injector is created, from which you can obtain the dependencies themselves.
Currently, there are 2 ways to get values: getting a single dependency and getting a tuple.

```rust
#[derive(Component, Clone)]
struct A {}

#[derive(Component, Clone)]
struct B {}

fn main() {
    let injector = InjectionBinder::new()
        .inject::<A>()
        .inject::<B>()
        .build();

    let a: A = injector.get()?; // getting a single value
    let (a, b): (A, B) = injector.get_tuple()?; // getting a tuple
    todo!()
}
```

Currently, tuples up to dimension 18 are supported.

# Generics

Generics in macros are also supported, but with the limitation that they must implement 
the `Clone` trait and have a `'static` lifetime:

```rust
 #[derive(Component, Clone)]
struct A<T: Clone + 'static> {
    x: u32
}

fn main() {
    let injector = InjectionBinder::new()
        .instance(1u32)
        .instance(2u64)
        .inject::<A<u32>>()
        .inject::<A<u64>>()
        .build();

    let a: A = injector.get::<A<u32>>()?;
    todo!()
}
```

# Circular Dependencies

In some complex situations, there is a need to assemble circular dependencies. In a typical situation, this leads to an exception and a build error. But for this situation, there is a special Lazy type.

It is applied simply by adding it to the inject method:

```rust
 #[derive(Component, Clone)]
struct A {
    x: Lazy<B>
}

struct B {
    x: A,
    y: u32
}

fn main() {
    let injector = InjectionBinder::new()
        .instance(1u32)
        .inject::<A>()
        .inject::<B>()
        .inject::<Lazy<B>>()
        .build();

    let a: A = injector.get::<A>()?;
    todo!()
}
```
Also, it's worth noting that nested lazy types are prohibited

# Working with dyn traits

In some cases, it makes sense to abstract from the type and work with Arc<dyn Trait> or Box<dyn Trait>. 
For these situations, there is a special auto trait and erase! macro.

For example, like this:

```rust
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

fn main() {
    let inject_res = InjectionBinder::new()
        .inject::<A>().auto(erase!(Arc<dyn Test>))
        .instance(1u32)
        .build()
        .unwrap();
    let dyn_type = inject_res.get::<Arc<dyn Test>>().unwrap();
}
```

What's happening here? `auto` is simply adding a new dependency based on the previous type without adding 
it to the InjectionBinder's type. In other words, you could achieve the same effect by writing `.inject_fn(|(x, )| -> Arc<dyn Test> { Arc::new(x) })`, 
but doing so would require writing a lot of boilerplate code, which you'd want to avoid.

Why might we need to work with `dyn traits`?
One reason is to abstract away from implementations and simplify the use of mocks, such as those from the (mockall)[https://github.com/asomers/mockall] library.

But if you need to use something like `Box` instead of `Arc`, you need to use the library (
dyn-clone)[https://github.com/dtolnay/dyn-clone]

```rust
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

fn main() {
    let inject_res = InjectionBinder::new()
        .inject::<A>().auto(erase!(Box<dyn Test>))
        .instance(1u32)
        .build()
        .unwrap();
    let dyn_type = inject_res.get::<Arc<dyn Test>>().unwrap();
}
```

# Duplicate Dependencies and Tagging


# Limitations
Current implementation limitations:
* All types must be 'static and must implement Clone
* Heap is heavily used, so no_std usage is not yet possible
* It is worth noting that there can be multiple copies made at the moment of building dependencies, which should not be critical for most long-lived applications,
and based on basic tests, it is performed 1-2 orders of magnitude faster than simple config parsing.

# Licensing
Licensed under either of Apache License, Version 2.0 or MIT license at your option.

# Contribution
Any contribution is welcome. Just write tests and submit merge requests on GitLab. 
All others merge requests will be closed with a request to add a merge request on GitLab.

# Special thanks to

* Chat GPT-4, which helped me write all this documentation and correct a huge number of errors in the code
* Christine, who was my inspiration
* Numerous libraries in Java, Scala, and Rust that I used as references
* Library authors, you are the best

# Related projects

## rust

* [inject](https://docs.rs/inject/latest/inject/)
* [teloc]( https://github.com/p0lunin/teloc)
* [shaku]( https://github.com/AzureMarker/shaku )
* [waiter]( https://github.com/dmitryb-dev/waiter)

## java

* [guice](https://github.com/google/guice)
* [spring](https://github.com/spring-projects/spring-framework)

## scala

* [macwire](https://github.com/softwaremill/macwire)
* [izumi](https://github.com/7mind/izumi)