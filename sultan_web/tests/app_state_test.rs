mod common;

use common::MockAppStateBuilder;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct TestConfig {
    name: String,
    value: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct AnotherTestType {
    data: String,
}

#[test]
fn test_get_existing_extension_returns_some() {
    let config = Arc::new(TestConfig {
        name: "test".to_string(),
        value: 42,
    });

    let app_state = MockAppStateBuilder::new()
        .add_extension(config.clone())
        .build();

    let retrieved = app_state.get::<TestConfig>();
    assert!(retrieved.is_some());
    let retrieved_val = retrieved.unwrap();
    assert_eq!(retrieved_val.name, "test");
    assert_eq!(retrieved_val.value, 42);
}

#[test]
fn test_get_nonexistent_extension_returns_none() {
    let app_state = MockAppStateBuilder::new().build();

    let retrieved = app_state.get::<TestConfig>();
    assert!(retrieved.is_none());
}

#[test]
fn test_get_wrong_type_returns_none() {
    let config = Arc::new(TestConfig {
        name: "test".to_string(),
        value: 42,
    });

    let app_state = MockAppStateBuilder::new().add_extension(config).build();

    // Try to get a different type
    let retrieved = app_state.get::<AnotherTestType>();
    assert!(retrieved.is_none());
}

#[test]
fn test_get_multiple_different_types() {
    let config = Arc::new(TestConfig {
        name: "config".to_string(),
        value: 100,
    });

    let other = Arc::new(AnotherTestType {
        data: "some data".to_string(),
    });

    let app_state = MockAppStateBuilder::new()
        .add_extension(config.clone())
        .add_extension(other.clone())
        .build();

    // Retrieve first type
    let retrieved_config = app_state.get::<TestConfig>();
    assert!(retrieved_config.is_some());
    assert_eq!(retrieved_config.unwrap().name, "config");

    // Retrieve second type
    let retrieved_other = app_state.get::<AnotherTestType>();
    assert!(retrieved_other.is_some());
    assert_eq!(retrieved_other.unwrap().data, "some data");
}

#[test]
fn test_get_returns_arc_with_shared_ownership() {
    let config = Arc::new(TestConfig {
        name: "shared".to_string(),
        value: 999,
    });

    let app_state = MockAppStateBuilder::new()
        .add_extension(config.clone())
        .build();

    // Get the extension multiple times
    let retrieved1 = app_state.get::<TestConfig>().unwrap();
    let retrieved2 = app_state.get::<TestConfig>().unwrap();

    // Both should point to the same data
    assert_eq!(retrieved1.name, "shared");
    assert_eq!(retrieved2.name, "shared");

    // Arc should have multiple strong references
    assert!(Arc::strong_count(&retrieved1) >= 2);
}

#[test]
fn test_get_with_primitive_types() {
    let string_val = Arc::new("Hello, World!".to_string());
    let int_val = Arc::new(42i64);
    let bool_val = Arc::new(true);

    let app_state = MockAppStateBuilder::new()
        .add_extension(string_val)
        .add_extension(int_val)
        .add_extension(bool_val)
        .build();

    assert_eq!(*app_state.get::<String>().unwrap(), "Hello, World!");
    assert_eq!(*app_state.get::<i64>().unwrap(), 42);
    assert!(*app_state.get::<bool>().unwrap());
}

#[test]
fn test_get_after_clone() {
    let config = Arc::new(TestConfig {
        name: "original".to_string(),
        value: 123,
    });

    let app_state = MockAppStateBuilder::new().add_extension(config).build();

    // Clone the app state
    let cloned_state = app_state.clone();

    // Both should be able to get the extension
    assert!(app_state.get::<TestConfig>().is_some());
    assert!(cloned_state.get::<TestConfig>().is_some());

    assert_eq!(app_state.get::<TestConfig>().unwrap().name, "original");
    assert_eq!(cloned_state.get::<TestConfig>().unwrap().name, "original");
}

#[test]
fn test_get_with_empty_extensions() {
    let app_state = MockAppStateBuilder::new().build();

    // Try to get various types from empty extensions
    assert!(app_state.get::<String>().is_none());
    assert!(app_state.get::<i64>().is_none());
    assert!(app_state.get::<TestConfig>().is_none());
}

#[test]
fn test_get_preserves_data_integrity() {
    let config = Arc::new(TestConfig {
        name: "integrity_test".to_string(),
        value: 777,
    });

    let app_state = MockAppStateBuilder::new()
        .add_extension(config.clone())
        .build();

    // Get the same extension multiple times
    for _ in 0..10 {
        let retrieved = app_state.get::<TestConfig>().unwrap();
        assert_eq!(retrieved.name, "integrity_test");
        assert_eq!(retrieved.value, 777);
    }
}

#[test]
fn test_get_with_complex_types() {
    #[derive(Clone, Debug)]
    struct ComplexType {
        vec_data: Vec<String>,
        nested: Option<String>,
    }

    let complex = Arc::new(ComplexType {
        vec_data: vec!["one".to_string(), "two".to_string(), "three".to_string()],
        nested: Some("nested value".to_string()),
    });

    let app_state = MockAppStateBuilder::new().add_extension(complex).build();

    let retrieved = app_state.get::<ComplexType>().unwrap();
    assert_eq!(retrieved.vec_data.len(), 3);
    assert_eq!(retrieved.vec_data[0], "one");
    assert!(retrieved.nested.is_some());
}

#[test]
fn test_get_type_safety() {
    // Create two different types with same field names
    #[derive(Clone, Debug)]
    struct TypeA {
        value: i32,
    }

    #[derive(Clone, Debug)]
    struct TypeB {
        value: i32,
    }

    let type_a = Arc::new(TypeA { value: 100 });
    let type_b = Arc::new(TypeB { value: 200 });

    let app_state = MockAppStateBuilder::new()
        .add_extension(type_a)
        .add_extension(type_b)
        .build();

    // Should retrieve correct types
    let retrieved_a = app_state.get::<TypeA>().unwrap();
    let retrieved_b = app_state.get::<TypeB>().unwrap();

    assert_eq!(retrieved_a.value, 100);
    assert_eq!(retrieved_b.value, 200);
}
