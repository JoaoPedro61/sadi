use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::error::SaDiError;

/// A simple, flexible dependency injection container
///
/// Supports both transient and singleton service registration
/// with a clean, type-safe API.
pub struct SaDi {
    /// Factories for transient services (new instance each time)
    factories: HashMap<TypeId, Box<dyn Fn(&SaDi) -> Box<dyn Any>>>,
    /// Factories for singleton services (cached instances)
    singletons: HashMap<TypeId, Box<dyn Fn(&SaDi) -> Box<dyn Any>>>,
    /// Cache for singleton instances
    singleton_cache: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
}

impl SaDi {
    /// Create a new DI container
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            singletons: HashMap::new(),
            singleton_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Register a transient factory
    ///
    /// Creates a new instance every time `get()` is called
    pub fn factory<T, F>(self, factory: F) -> Self
    where
        T: 'static + Any,
        F: Fn(&SaDi) -> T + 'static,
    {
        self.try_factory(factory)
            .unwrap_or_else(|err| panic!("{}", err))
    }

    /// Try to register a transient factory
    ///
    /// Returns Ok(Self) if successful, or Err if factory already exists
    pub fn try_factory<T, F>(mut self, factory: F) -> Result<Self, SaDiError>
    where
        T: 'static + Any,
        F: Fn(&SaDi) -> T + 'static,
    {
        let type_id = TypeId::of::<T>();

        if self.factories.contains_key(&type_id) {
            return Err(SaDiError::factory_already_registered(
                std::any::type_name::<T>(),
                "transient",
            ));
        }

        self.factories
            .insert(type_id, Box::new(move |di| Box::new(factory(di))));
        Ok(self)
    }

    /// Get a transient instance
    ///
    /// Returns a new instance every time
    pub fn get<T: 'static + Any>(&self) -> T {
        self.try_get().unwrap_or_else(|err| panic!("{}", err))
    }

    /// Try to get a transient instance
    ///
    /// Returns Ok(T) with a new instance if factory is registered, or Err with error message
    pub fn try_get<T: 'static + Any>(&self) -> Result<T, String> {
        let type_id = TypeId::of::<T>();

        if let Some(factory) = self.factories.get(&type_id) {
            let boxed_any = factory(self);
            match boxed_any.downcast::<T>() {
                Ok(instance) => Ok(*instance),
                Err(_) => Err(format!(
                    "Factory returned wrong type for: {}",
                    std::any::type_name::<T>()
                )),
            }
        } else {
            Err(format!(
                "No transient factory registered for type: {}",
                std::any::type_name::<T>()
            ))
        }
    }

    /// Register a singleton factory
    ///
    /// Creates the instance once and caches it for subsequent calls
    pub fn factory_singleton<T, F>(self, factory: F) -> Self
    where
        T: 'static + Any,
        F: Fn(&SaDi) -> T + 'static,
    {
        self.try_factory_singleton(factory)
            .unwrap_or_else(|err| panic!("{}", err))
    }

    /// Try to register a singleton factory
    ///
    /// Returns Ok(Self) if successful, or Err if factory already exists
    pub fn try_factory_singleton<T, F>(mut self, factory: F) -> Result<Self, SaDiError>
    where
        T: 'static + Any,
        F: Fn(&SaDi) -> T + 'static,
    {
        let type_id = TypeId::of::<T>();

        if self.singletons.contains_key(&type_id) {
            return Err(SaDiError::factory_already_registered(
                std::any::type_name::<T>(),
                "singleton",
            ));
        }

        self.singletons
            .insert(type_id, Box::new(move |di| Box::new(factory(di))));
        Ok(self)
    }

    /// Get a singleton instance
    ///
    /// Returns the same cached instance every time
    pub fn get_singleton<T: 'static + Any>(&self) -> Rc<T> {
        self.try_get_singleton()
            .unwrap_or_else(|err| panic!("{}", err))
    }

    /// Try to get a singleton instance
    ///
    /// Returns Ok(Rc<T>) with the cached instance if factory is registered, or Err with error message
    pub fn try_get_singleton<T: 'static + Any>(&self) -> Result<Rc<T>, String> {
        let type_id = TypeId::of::<T>();

        // Check cache first
        {
            let cache = self.singleton_cache.borrow();
            if let Some(cached) = cache.get(&type_id) {
                return cached.clone().downcast::<T>().map_err(|_| {
                    format!(
                        "Cached instance has wrong type for: {}",
                        std::any::type_name::<T>()
                    )
                });
            }
        }

        // Create new instance and cache it
        if let Some(factory) = self.singletons.get(&type_id) {
            let boxed_any = factory(self);
            match boxed_any.downcast::<T>() {
                Ok(boxed_t) => {
                    let rc_instance = Rc::new(*boxed_t);
                    let rc_any: Rc<dyn Any> = rc_instance.clone();
                    self.singleton_cache.borrow_mut().insert(type_id, rc_any);
                    Ok(rc_instance)
                }
                Err(_) => Err(format!(
                    "Factory returned wrong type for: {}",
                    std::any::type_name::<T>()
                )),
            }
        } else {
            Err(format!(
                "No singleton factory registered for type: {}",
                std::any::type_name::<T>()
            ))
        }
    }
}
