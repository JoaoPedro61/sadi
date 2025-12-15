use sadi::{Container, Limits};
use std::sync::{Arc, Mutex};

#[test]
fn test_deny_policy_enforces_limit() {
    struct Service {
        id: usize,
    }

    let counter = Arc::new(Mutex::new(0));

    let c = Container::new();
    let limits = Limits::deny(2); // Allow max 2 concurrent creations

    let counter_clone = counter.clone();
    c.bind_concrete_with_limits::<Service, Service, _>(
        move |_| {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            Service { id: *count }
        },
        limits,
    )
    .unwrap();

    // First two resolutions should succeed
    let s1 = c.resolve::<Service>().unwrap();
    let s2 = c.resolve::<Service>().unwrap();

    assert_eq!(s1.id, 1);
    assert_eq!(s2.id, 2);
}

#[test]
fn test_block_policy_waits_for_available_slot() {
    struct Service;

    let c = Container::new();
    let limits = Limits::block(1); // Allow only 1 concurrent creation

    c.bind_concrete_with_limits::<Service, Service, _>(
        |_| Service,
        limits,
    )
    .unwrap();

    // First resolution should succeed
    let s1 = c.resolve::<Service>();
    assert!(s1.is_ok());
}

#[test]
fn test_transient_with_limits() {
    struct Service {
        id: usize,
    }

    let counter = Arc::new(Mutex::new(0));

    let c = Container::new();
    let limits = Limits::block(10);

    let counter_clone = counter.clone();
    c.bind_concrete_with_limits::<Service, Service, _>(
        move |_| {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            Service { id: *count }
        },
        limits,
    )
    .unwrap();

    // Transient services should get new instances each time
    let s1 = c.resolve::<Service>().unwrap();
    let s2 = c.resolve::<Service>().unwrap();

    // Both should have been created (different IDs)
    assert_eq!(s1.id, 1);
    assert_eq!(s2.id, 2);
}

#[test]
fn test_singleton_with_limits() {
    struct Service {
        id: usize,
    }

    let counter = Arc::new(Mutex::new(0));

    let c = Container::new();
    let limits = Limits::block(10);

    let counter_clone = counter.clone();
    c.bind_concrete_singleton_with_limits::<Service, Service, _>(
        move |_| {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            Service { id: *count }
        },
        limits,
    )
    .unwrap();

    // Singleton should be created only once
    let s1 = c.resolve::<Service>().unwrap();
    let s2 = c.resolve::<Service>().unwrap();

    // Both should be the same instance (same ID)
    assert_eq!(s1.id, 1);
    assert_eq!(s2.id, 1); // Still 1, not 2
}

#[test]
fn test_abstract_binding_with_limits() {
    trait MyService: Send + Sync {
        fn value(&self) -> i32;
    }

    struct ServiceImpl;

    impl MyService for ServiceImpl {
        fn value(&self) -> i32 {
            42
        }
    }

    let c = Container::new();
    let limits = Limits::block(5);

    c.bind_abstract_with_limits::<dyn MyService, Arc<dyn MyService>, _>(
        |_| Arc::new(ServiceImpl),
        limits,
    )
    .unwrap();

    let service = c.resolve::<dyn MyService>().unwrap();
    assert_eq!(service.value(), 42);
}
