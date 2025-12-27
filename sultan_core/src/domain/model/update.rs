use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Represents an update action for a field in PATCH requests.
///
/// This enum provides explicit semantics for partial updates, distinguishing between:
/// - Field not present in request (don't change)
/// - Field explicitly set to null (clear the value)
/// - Field has a new value (update to new value)
///
/// # JSON Mapping
/// - Missing field → `Unchanged` (field not in JSON)
/// - `null` value → `Clear` (field explicitly set to null)
/// - Value present → `Set(value)` (field has new value)
///
/// # Example
/// ```ignore
/// use sultan_core::domain::model::Update;
///
/// #[derive(Default, serde::Deserialize)]
/// struct MyUpdate {
///     #[serde(default)]
///     name: Update<String>,
///     #[serde(default)]
///     address: Update<String>,
/// }
///
/// // JSON: {}
/// // Result: name = Unchanged, address = Unchanged
///
/// // JSON: { "address": null }
/// // Result: name = Unchanged, address = Clear
///
/// // JSON: { "address": "123 Main St" }
/// // Result: name = Unchanged, address = Set("123 Main St")
/// ```
#[derive(Debug, Clone, PartialEq, Default, ToSchema)]
pub enum Update<T> {
    /// Don't change the field (field missing from request)
    #[default]
    Unchanged,
    /// Clear the field (set to NULL)
    Clear,
    /// Set the field to a new value
    Set(T),
}

impl<T> Update<T> {
    /// Returns `true` if this update should modify the database field.
    ///
    /// Returns `false` only for `Unchanged` variant.
    pub fn should_update(&self) -> bool {
        !matches!(self, Update::Unchanged)
    }

    /// Returns `true` if the variant is `Unchanged`.
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Update::Unchanged)
    }

    /// Returns `true` if the variant is `Clear`.
    pub fn is_clear(&self) -> bool {
        matches!(self, Update::Clear)
    }

    /// Returns `true` if the variant is `Set`.
    pub fn is_set(&self) -> bool {
        matches!(self, Update::Set(_))
    }

    /// Returns the contained value if `Set`, or `None` otherwise.
    pub fn as_value(&self) -> Option<&T> {
        match self {
            Update::Set(v) => Some(v),
            _ => None,
        }
    }

    /// Converts to `Option<Option<T>>` for compatibility with existing code.
    ///
    /// - `Unchanged` → `None`
    /// - `Clear` → `Some(None)`
    /// - `Set(v)` → `Some(Some(v))`
    pub fn into_option(self) -> Option<Option<T>> {
        match self {
            Update::Unchanged => None,
            Update::Clear => Some(None),
            Update::Set(v) => Some(Some(v)),
        }
    }

    /// Converts to `Option<T>` for binding to SQL.
    ///
    /// - `Unchanged` → panics (should check `should_update()` first)
    /// - `Clear` → `None`
    /// - `Set(v)` → `Some(v)`
    ///
    /// # Panics
    /// Panics if called on `Unchanged` variant.
    pub fn into_bind_value(self) -> Option<T> {
        match self {
            Update::Unchanged => {
                panic!("Cannot bind Unchanged value - check should_update() first")
            }
            Update::Clear => None,
            Update::Set(v) => Some(v),
        }
    }

    /// Maps the inner value using the provided function.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Update<U> {
        match self {
            Update::Unchanged => Update::Unchanged,
            Update::Clear => Update::Clear,
            Update::Set(v) => Update::Set(f(v)),
        }
    }
}

impl<T: Clone> Update<T> {
    /// Returns the bind value as `Option<T>` by cloning.
    ///
    /// - `Unchanged` → panics
    /// - `Clear` → `None`
    /// - `Set(v)` → `Some(v.clone())`
    ///
    /// # Panics
    /// Panics if called on `Unchanged` variant.
    pub fn to_bind_value(&self) -> Option<T> {
        match self {
            Update::Unchanged => {
                panic!("Cannot bind Unchanged value - check should_update() first")
            }
            Update::Clear => None,
            Update::Set(v) => Some(v.clone()),
        }
    }
}

impl<T: Validate> Validate for Update<T> {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            Update::Set(value) => value.validate(),
            Update::Clear | Update::Unchanged => Ok(()),
        }
    }
}

impl<T: Serialize> Serialize for Update<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Update::Unchanged => serializer.serialize_none(),
            Update::Clear => serializer.serialize_none(),
            Update::Set(v) => v.serialize(serializer),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Update<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // This is called ONLY when the field is present in JSON
        let opt = Option::<T>::deserialize(deserializer)?;
        Ok(match opt {
            Some(v) => Update::Set(v),
            None => Update::Clear, // Field was present but null
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_unchanged() {
        let update: Update<String> = Default::default();
        assert_eq!(update, Update::Unchanged);
        assert!(update.is_unchanged());
        assert!(!update.should_update());
    }

    #[test]
    fn test_clear_should_update() {
        let update: Update<String> = Update::Clear;
        assert!(update.is_clear());
        assert!(update.should_update());
        assert!(!update.is_unchanged());
    }

    #[test]
    fn test_set_should_update() {
        let update: Update<String> = Update::Set("value".to_string());
        assert!(update.is_set());
        assert!(update.should_update());
        assert!(!update.is_unchanged());
    }

    #[test]
    fn test_into_option() {
        let unchanged: Update<String> = Update::Unchanged;
        let clear: Update<String> = Update::Clear;
        let set: Update<String> = Update::Set("value".to_string());

        assert_eq!(unchanged.into_option(), None);
        assert_eq!(clear.into_option(), Some(None));
        assert_eq!(set.into_option(), Some(Some("value".to_string())));
    }

    #[test]
    fn test_to_bind_value() {
        let clear: Update<String> = Update::Clear;
        let set: Update<String> = Update::Set("value".to_string());

        assert_eq!(clear.to_bind_value(), None);
        assert_eq!(set.to_bind_value(), Some("value".to_string()));
    }

    #[test]
    #[should_panic(expected = "Cannot bind Unchanged value")]
    fn test_to_bind_value_unchanged_panics() {
        let unchanged: Update<String> = Update::Unchanged;
        let _ = unchanged.to_bind_value();
    }

    #[test]
    fn test_as_value() {
        let unchanged: Update<String> = Update::Unchanged;
        let clear: Update<String> = Update::Clear;
        let set: Update<String> = Update::Set("value".to_string());

        assert_eq!(unchanged.as_value(), None);
        assert_eq!(clear.as_value(), None);
        assert_eq!(set.as_value(), Some(&"value".to_string()));
    }

    #[test]
    fn test_map() {
        let set: Update<i32> = Update::Set(5);
        let mapped = set.map(|v| v * 2);
        assert_eq!(mapped, Update::Set(10));

        let clear: Update<i32> = Update::Clear;
        let mapped_clear = clear.map(|v| v * 2);
        assert_eq!(mapped_clear, Update::Clear);

        let unchanged: Update<i32> = Update::Unchanged;
        let mapped_unchanged = unchanged.map(|v| v * 2);
        assert_eq!(mapped_unchanged, Update::Unchanged);
    }

    #[test]
    fn test_into_bind_value() {
        let clear: Update<String> = Update::Clear;
        let set: Update<String> = Update::Set("value".to_string());

        assert_eq!(clear.into_bind_value(), None);
        assert_eq!(set.into_bind_value(), Some("value".to_string()));
    }

    #[test]
    #[should_panic(expected = "Cannot bind Unchanged value")]
    fn test_into_bind_value_unchanged_panics() {
        let unchanged: Update<String> = Update::Unchanged;
        let _ = unchanged.into_bind_value();
    }
}

#[cfg(test)]
mod serde_tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Default, PartialEq)]
    struct TestStruct {
        #[serde(default)]
        name: Update<String>,
        #[serde(default)]
        count: Update<i32>,
    }

    // ==================== Deserialization Tests ====================

    #[test]
    fn test_deserialize_missing_field_is_unchanged() {
        let json = r#"{}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, Update::Unchanged);
        assert_eq!(result.count, Update::Unchanged);
    }

    #[test]
    fn test_deserialize_null_is_clear() {
        let json = r#"{"name": null, "count": null}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, Update::Clear);
        assert_eq!(result.count, Update::Clear);
    }

    #[test]
    fn test_deserialize_value_is_set() {
        let json = r#"{"name": "Alice", "count": 42}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, Update::Set("Alice".to_string()));
        assert_eq!(result.count, Update::Set(42));
    }

    #[test]
    fn test_deserialize_mixed_states() {
        // name is missing (Unchanged), count is null (Clear)
        let json = r#"{"count": null}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, Update::Unchanged);
        assert_eq!(result.count, Update::Clear);
    }

    #[test]
    fn test_deserialize_set_and_unchanged() {
        // name has value (Set), count is missing (Unchanged)
        let json = r#"{"name": "Bob"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, Update::Set("Bob".to_string()));
        assert_eq!(result.count, Update::Unchanged);
    }

    #[test]
    fn test_deserialize_all_three_states() {
        #[derive(Debug, Deserialize, Default, PartialEq)]
        struct ThreeFieldStruct {
            #[serde(default)]
            unchanged_field: Update<String>,
            #[serde(default)]
            clear_field: Update<String>,
            #[serde(default)]
            set_field: Update<String>,
        }

        let json = r#"{"clear_field": null, "set_field": "value"}"#;
        let result: ThreeFieldStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.unchanged_field, Update::Unchanged);
        assert_eq!(result.clear_field, Update::Clear);
        assert_eq!(result.set_field, Update::Set("value".to_string()));
    }

    #[test]
    fn test_deserialize_empty_string_is_set() {
        // Empty string should be Set(""), not Clear
        let json = r#"{"name": ""}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, Update::Set("".to_string()));
    }

    #[test]
    fn test_deserialize_zero_is_set() {
        // Zero should be Set(0), not Clear
        let json = r#"{"count": 0}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.count, Update::Set(0));
    }

    #[test]
    fn test_deserialize_nested_object() {
        #[derive(Debug, Deserialize, PartialEq, Clone)]
        struct Inner {
            value: String,
        }

        #[derive(Debug, Deserialize, Default, PartialEq)]
        struct Outer {
            #[serde(default)]
            inner: Update<Inner>,
        }

        let json = r#"{"inner": {"value": "test"}}"#;
        let result: Outer = serde_json::from_str(json).unwrap();

        assert_eq!(
            result.inner,
            Update::Set(Inner {
                value: "test".to_string()
            })
        );
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_serialize_unchanged_is_null() {
        let update: Update<String> = Update::Unchanged;
        let json = serde_json::to_string(&update).unwrap();

        assert_eq!(json, "null");
    }

    #[test]
    fn test_serialize_clear_is_null() {
        let update: Update<String> = Update::Clear;
        let json = serde_json::to_string(&update).unwrap();

        assert_eq!(json, "null");
    }

    #[test]
    fn test_serialize_set_string() {
        let update: Update<String> = Update::Set("hello".to_string());
        let json = serde_json::to_string(&update).unwrap();

        assert_eq!(json, r#""hello""#);
    }

    #[test]
    fn test_serialize_set_integer() {
        let update: Update<i32> = Update::Set(42);
        let json = serde_json::to_string(&update).unwrap();

        assert_eq!(json, "42");
    }

    #[test]
    fn test_serialize_set_bool() {
        let update: Update<bool> = Update::Set(true);
        let json = serde_json::to_string(&update).unwrap();

        assert_eq!(json, "true");
    }

    #[test]
    fn test_serialize_struct_with_updates() {
        #[derive(Debug, Serialize)]
        struct Response {
            name: Update<String>,
            count: Update<i32>,
        }

        let response = Response {
            name: Update::Set("test".to_string()),
            count: Update::Clear,
        };

        let json = serde_json::to_string(&response).unwrap();

        // Both fields are serialized, Clear becomes null
        assert!(json.contains(r#""name":"test""#));
        assert!(json.contains(r#""count":null"#));
    }

    // ==================== Round-trip Tests ====================

    #[test]
    fn test_roundtrip_set_value() {
        let original: Update<String> = Update::Set("roundtrip".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Update<String> = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, Update::Set("roundtrip".to_string()));
    }

    #[test]
    fn test_serialization_loses_unchanged_vs_clear_distinction() {
        // Both Unchanged and Clear serialize to null
        // This is expected behavior - the distinction only matters on input
        let unchanged: Update<String> = Update::Unchanged;
        let clear: Update<String> = Update::Clear;

        let unchanged_json = serde_json::to_string(&unchanged).unwrap();
        let clear_json = serde_json::to_string(&clear).unwrap();

        assert_eq!(unchanged_json, clear_json);
        assert_eq!(unchanged_json, "null");
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use validator::Validate;

    #[derive(Debug, Clone, Validate)]
    struct ValidatedStruct {
        #[validate(length(min = 3, max = 10))]
        name: String,
        #[validate(range(min = 0, max = 100))]
        age: i32,
    }

    #[test]
    fn test_validate_set_valid() {
        let update: Update<ValidatedStruct> = Update::Set(ValidatedStruct {
            name: "Alice".to_string(),
            age: 25,
        });

        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_validate_set_invalid_name_too_short() {
        let update: Update<ValidatedStruct> = Update::Set(ValidatedStruct {
            name: "Al".to_string(),
            age: 25,
        });

        let result = update.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_validate_set_invalid_name_too_long() {
        let update: Update<ValidatedStruct> = Update::Set(ValidatedStruct {
            name: "VeryLongName".to_string(),
            age: 25,
        });

        let result = update.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_validate_set_invalid_age_out_of_range() {
        let update: Update<ValidatedStruct> = Update::Set(ValidatedStruct {
            name: "Alice".to_string(),
            age: 150,
        });

        let result = update.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("age"));
    }

    #[test]
    fn test_validate_clear_always_valid() {
        let update: Update<ValidatedStruct> = Update::Clear;
        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_validate_unchanged_always_valid() {
        let update: Update<ValidatedStruct> = Update::Unchanged;
        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_validate_string_with_validation() {
        #[derive(Debug, Clone, Validate)]
        struct Email {
            #[validate(email)]
            value: String,
        }

        let valid_email = Update::Set(Email {
            value: "test@example.com".to_string(),
        });
        assert!(valid_email.validate().is_ok());

        let invalid_email = Update::Set(Email {
            value: "not-an-email".to_string(),
        });
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_validate_nested_struct() {
        #[derive(Debug, Clone, Validate)]
        struct Address {
            #[validate(length(min = 5))]
            street: String,
        }

        #[derive(Debug, Clone, Validate)]
        struct Person {
            #[validate(length(min = 2))]
            name: String,
            #[validate(nested)]
            address: Address,
        }

        let valid = Update::Set(Person {
            name: "Bob".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
            },
        });
        assert!(valid.validate().is_ok());

        let invalid = Update::Set(Person {
            name: "Bob".to_string(),
            address: Address {
                street: "123".to_string(), // Too short
            },
        });
        assert!(invalid.validate().is_err());
    }
}
