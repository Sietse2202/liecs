#![no_std]
#![cfg(any(target_arch = "x86", target_arch = "arm"))]

extern crate alloc;

use core::time::Duration;
use heapless::Vec;
#[allow(deprecated)]
use time::Instant;

pub struct App<'a> {
    pub(crate) entities: Vec<Entity<'a>, 128>,
    count: u8,
    startup_systems: Vec<&'a dyn System<'a>, 4>,
    update_systems: Vec<&'a dyn System<'a>, 16>,
}

pub enum Moment {
    Startup,
    Update,
}

pub(crate) struct Entity<'a> {
    pub(crate) id: u8,
    pub(crate) components: Vec<dyn Component, 16>,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl App<'_> {
    pub const fn new() -> Self {
        Self {
            entities: Vec::new(),
            count: 0,
            startup_systems: Vec::new(),
            update_systems: Vec::new(),
        }
    }

    /// Spawn an entity with the given components
    pub fn spawn<'a>(&mut self, components: &[&'a dyn Component]) {
        let mut entity_components = Vec::new();
        for component in components {
            if entity_components.push(*component).is_err() {
                return;
            }
        }

        let entity = Entity {
            id: self.count + 1,
            components: entity_components,
        };

        self.count += 1;

        if self.entities.push(entity).is_err() {}
    }

    /// Add a system to the specified moment
    pub fn add_system<S: SystemFn<'_> + '_>(&mut self, moment: Moment, system: S) {
        match moment {
            Moment::Startup => {
                if self.startup_systems.push(&system as &dyn System).is_err() {
                    // Handle error if needed
                }
            }
            Moment::Update => {
                if self.update_systems.push(&system as &dyn System).is_err() {
                    // Handle error if needed
                }
            }
        }
    }

    /// Runs all systems in their respective moments
    pub fn run(&mut self) {
        for system in &self.startup_systems {
            system.run(self, Duration::from_secs(0));
        }
        
        #[allow(deprecated)]
        let mut start = Instant::now();
        loop {
            let delta = start.elapsed();
            for system in &self.update_systems {
                system.run(self, Duration::try_from(delta).unwrap());
            };
            #[allow(deprecated)]
            start = Instant::now();
        }
    }

    /// Query entities based on specific component constraints
    pub fn query<'b, Q: QueryFilter<'b>>(&'b self) -> Vec<&'b Entity<'b>, 128> {
        let mut results = Vec::new();
        for entity in &self.entities {
            if Q::matches(entity) {
                if results.push(entity).is_err() {
                    // Handle error if needed
                }
            }
        }
        results
    }
}

pub trait Component {}

pub trait QueryFilter<'a> {
    fn matches(entity: &Entity<'a>) -> bool;
}

impl App<'_> {
    pub fn query<'b, Q: QueryFilter<'b>>(&'b self) -> &[&'b Entity<'b>] {
        let mut results = Vec::new();
        for entity in &self.entities {
            if Q::matches(entity) {
                if results.push(entity).is_err() {
                    panic!("Query failed.");
                }
            }
        }
        results.as_slice()
    }
}

pub trait System<'a> {
    fn run(&self, app: &mut App<'a>, delta: Duration);
}

/// A trait alias for system functions
pub trait SystemFn<'a>: Fn(&mut App<'a>, Duration) {}

impl<'a, F> SystemFn<'a> for F where F: Fn(&mut App<'a>, Duration) {}

impl<'a, F> System<'a> for F
where
    F: SystemFn<'a>,
{
    fn run(&self, app: &mut App<'a>, delta: Duration) {
        self(app, delta);
    }
}

/// Query entities that contain a specific component type
pub struct Query<C: Component>(core::marker::PhantomData<C>);

impl<'a, C: Component + 'a> QueryFilter<'a> for Query<C> {
    fn matches(entity: &Entity<'a>) -> bool {
        entity
            .components
            .iter()
            .any(|comp| comp.as_ref().is::<C>())
    }
}

pub struct No<C: Component>(core::marker::PhantomData<C>);

impl<'a, C: Component + 'a> QueryFilter<'a> for No<C> {
    fn matches(entity: &Entity<'a>) -> bool {
        !entity
            .components
            .iter()
            .any(|comp| comp.as_ref().is::<C>())
    }
}