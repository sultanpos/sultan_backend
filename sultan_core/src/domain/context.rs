use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Context provides request-scoped state for operations.
///
/// It stores:
/// - User ID (optional)
/// - Permissions (resource + branch access)
/// - Arbitrary typed extensions via `get`
///
/// # Examples
///
/// ```rust
/// use sultan_core::domain::Context;
/// use std::sync::Arc;
/// use std::any::{Any, TypeId};
/// use std::collections::HashMap;
///
/// // Create context with user_id and extensions
/// let mut extensions = HashMap::new();
/// extensions.insert(
///     TypeId::of::<String>(),
///     Arc::new("request-123".to_string()) as Arc<dyn Any + Send + Sync>
/// );
/// let ctx = Context::new_with_all(Some(123), HashMap::new(), extensions);
///
/// // Retrieve typed data
/// let request_id: &String = ctx.get::<String>().unwrap();
/// ```
#[derive(Clone)]
pub struct Context {
    user_id: Option<i64>,
    // (resource, branch_id) -> permission
    permission: HashMap<(i32, Option<i64>), i32>,
    // Type-erased storage for arbitrary values using Arc for cheap cloning
    extensions: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            user_id: None,
            permission: HashMap::new(),
            extensions: HashMap::new(),
        }
    }

    pub fn new_with_all(
        user_id: Option<i64>,
        permission: HashMap<(i32, Option<i64>), i32>,
        extensions: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    ) -> Self {
        Self {
            user_id,
            permission,
            extensions,
        }
    }

    /// Get a reference to a value of type T from the context.
    /// Returns None if the value doesn't exist or has a different type.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.downcast_ref::<T>())
    }

    pub fn user_id(&self) -> Option<i64> {
        self.user_id
    }

    pub fn require_access(
        &self,
        branch_id: Option<i64>,
        resource: i32,
        action: i32,
    ) -> Result<(), crate::domain::Error> {
        if self.has_access(branch_id, resource, action) {
            Ok(())
        } else {
            Err(crate::domain::Error::Forbidden(format!(
                "Access denied for resource {} with action {}",
                resource, action
            )))
        }
    }

    pub fn has_access(&self, branch_id: Option<i64>, resource: i32, action: i32) -> bool {
        use crate::domain::model::permission::resource as res;

        // Check if user has ADMIN permission (global or for specific branch)
        if self.permission.contains_key(&(res::ADMIN, None)) {
            return true;
        }
        if let Some(bid) = branch_id
            && self.permission.contains_key(&(res::ADMIN, Some(bid)))
        {
            return true;
        }

        // Check global permission for the requested resource
        if let Some(&perm) = self.permission.get(&(resource, None))
            && (perm & action) == action
        {
            return true;
        }
        // Check branch-specific permission for the requested resource
        if let Some(bid) = branch_id
            && let Some(&perm) = self.permission.get(&(resource, Some(bid)))
            && (perm & action) == action
        {
            return true;
        }
        false
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_access_global_permission() {
        // User has global permission (branch_id = None) for resource 1
        let mut permissions = std::collections::HashMap::new();
        permissions.insert((1, None), 0b0011); // READ | CREATE

        let ctx = Context::new_with_all(None, permissions, HashMap::new());

        // Should have access for any branch
        assert!(ctx.has_access(Some(1), 1, 0b0001)); // CREATE on branch 1
        assert!(ctx.has_access(Some(2), 1, 0b0010)); // READ on branch 2
        assert!(ctx.has_access(Some(99), 1, 0b0011)); // READ | CREATE on branch 99
        assert!(ctx.has_access(None, 1, 0b0001)); // CREATE with no branch

        // Should NOT have access for actions not granted
        assert!(!ctx.has_access(Some(1), 1, 0b0100)); // UPDATE not granted
    }

    #[test]
    fn test_has_access_branch_specific_permission() {
        // User has permission only for branch 5
        let mut permissions = std::collections::HashMap::new();
        permissions.insert((1, Some(5)), 0b0011); // READ | CREATE for branch 5

        let ctx = Context::new_with_all(None, permissions, HashMap::new());

        // Should have access only for branch 5
        assert!(ctx.has_access(Some(5), 1, 0b0001)); // CREATE on branch 5
        assert!(ctx.has_access(Some(5), 1, 0b0010)); // READ on branch 5

        // Should NOT have access for other branches
        assert!(!ctx.has_access(Some(1), 1, 0b0001)); // CREATE on branch 1
        assert!(!ctx.has_access(Some(99), 1, 0b0010)); // READ on branch 99
        assert!(!ctx.has_access(None, 1, 0b0001)); // CREATE with no branch
    }

    #[test]
    fn test_has_access_mixed_permissions() {
        // User has global READ and branch-specific CREATE for branch 5
        let mut permissions = std::collections::HashMap::new();
        permissions.insert((1, None), 0b0010); // Global READ
        permissions.insert((1, Some(5)), 0b0001); // CREATE for branch 5

        let ctx = Context::new_with_all(None, permissions, HashMap::new());

        // Global READ should work for any branch
        assert!(ctx.has_access(Some(1), 1, 0b0010)); // READ on branch 1
        assert!(ctx.has_access(Some(5), 1, 0b0010)); // READ on branch 5
        assert!(ctx.has_access(Some(99), 1, 0b0010)); // READ on branch 99

        // CREATE should only work for branch 5
        assert!(ctx.has_access(Some(5), 1, 0b0001)); // CREATE on branch 5
        assert!(!ctx.has_access(Some(1), 1, 0b0001)); // CREATE on branch 1 - denied
    }

    #[test]
    fn test_has_access_no_permission() {
        let ctx = Context::new();

        assert!(!ctx.has_access(Some(1), 1, 0b0001));
        assert!(!ctx.has_access(None, 1, 0b0001));
    }

    #[test]
    fn test_has_access_requires_all_actions() {
        let mut permissions = std::collections::HashMap::new();
        permissions.insert((1, None), 0b0010); // Only READ

        let ctx = Context::new_with_all(None, permissions, HashMap::new());

        // Requesting READ | CREATE should fail because CREATE is missing
        assert!(!ctx.has_access(Some(1), 1, 0b0011)); // READ | CREATE

        // Requesting just READ should succeed
        assert!(ctx.has_access(Some(1), 1, 0b0010)); // READ
    }

    #[test]
    fn test_has_access_global_admin() {
        use crate::domain::model::permission::resource;

        // User has global ADMIN permission
        let mut permissions = std::collections::HashMap::new();
        permissions.insert((resource::ADMIN, None), 0b0001); // any value, just needs to exist

        let ctx = Context::new_with_all(None, permissions, HashMap::new());

        // Should have access to any resource, any action, any branch
        assert!(ctx.has_access(Some(1), resource::BRANCH, 0b0001)); // CREATE on branch 1
        assert!(ctx.has_access(Some(99), resource::USER, 0b1111)); // All actions on branch 99
        assert!(ctx.has_access(None, resource::BRANCH, 0b0100)); // UPDATE with no branch
    }

    #[test]
    fn test_has_access_branch_admin() {
        use crate::domain::model::permission::resource;

        // User has ADMIN permission only for branch 5
        let mut permissions = std::collections::HashMap::new();
        permissions.insert((resource::ADMIN, Some(5)), 0b0001); // ADMIN for branch 5

        let ctx = Context::new_with_all(None, permissions, HashMap::new());

        // Should have access to any resource, any action, but only for branch 5
        assert!(ctx.has_access(Some(5), resource::BRANCH, 0b0001)); // CREATE on branch 5
        assert!(ctx.has_access(Some(5), resource::USER, 0b1111)); // All actions on branch 5

        // Should NOT have access for other branches
        assert!(!ctx.has_access(Some(1), resource::BRANCH, 0b0001)); // CREATE on branch 1
        assert!(!ctx.has_access(None, resource::BRANCH, 0b0001)); // CREATE with no branch
    }

    #[test]
    fn test_user_id_default_is_none() {
        let ctx = Context::new();
        assert_eq!(ctx.user_id(), None);
    }

    #[test]
    fn test_user_id_preserved_when_cloned() {
        let ctx = Context::new_with_all(Some(123), HashMap::new(), HashMap::new());
        let cloned = ctx.clone();
        assert_eq!(cloned.user_id(), Some(123));
    }

    #[test]
    fn test_user_id_different_values() {
        let mut ctx1 = Context::new();
        ctx1.user_id = Some(100);

        let mut ctx2 = Context::new();
        ctx2.user_id = Some(200);

        assert_eq!(ctx1.user_id(), Some(100));
        assert_eq!(ctx2.user_id(), Some(200));
        assert_ne!(ctx1.user_id(), ctx2.user_id());
    }

    #[test]
    fn test_new_with_user_id() {
        let ctx = Context::new_with_all(Some(999), HashMap::new(), HashMap::new());
        assert_eq!(ctx.user_id(), Some(999));
    }

    #[test]
    fn test_context_with_user_id_and_permissions() {
        let mut permissions = std::collections::HashMap::new();
        permissions.insert((1, None), 0b0011);

        let ctx = Context::new_with_all(Some(777), permissions, HashMap::new());

        assert_eq!(ctx.user_id(), Some(777));
        assert!(ctx.has_access(None, 1, 0b0001));
    }

    #[test]
    fn test_multiple_contexts_with_different_user_ids() {
        let ctx1 = Context::new_with_all(Some(111), HashMap::new(), HashMap::new());
        let ctx2 = Context::new_with_all(Some(222), HashMap::new(), HashMap::new());
        let ctx3 = Context::new(); // No user_id set

        assert_eq!(ctx1.user_id(), Some(111));
        assert_eq!(ctx2.user_id(), Some(222));
        assert_eq!(ctx3.user_id(), None);
    }

    #[test]
    fn test_set_and_get_string() {
        let mut extensions = HashMap::new();
        extensions.insert(
            TypeId::of::<String>(),
            Arc::new("Hello, World!".to_string()) as Arc<dyn Any + Send + Sync>,
        );
        let ctx = Context::new_with_all(None, HashMap::new(), extensions);

        let value = ctx.get::<String>();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_set_and_get_integer() {
        let mut extensions = HashMap::new();
        extensions.insert(
            TypeId::of::<i64>(),
            Arc::new(42i64) as Arc<dyn Any + Send + Sync>,
        );
        let ctx = Context::new_with_all(None, HashMap::new(), extensions);

        let value = ctx.get::<i64>();
        assert!(value.is_some());
        assert_eq!(*value.unwrap(), 42);
    }

    #[test]
    fn test_get_nonexistent_type_returns_none() {
        let ctx = Context::new();
        let value = ctx.get::<String>();
        assert!(value.is_none());
    }

    #[test]
    fn test_extensions_are_immutable() {
        let mut extensions = HashMap::new();
        extensions.insert(
            TypeId::of::<String>(),
            Arc::new("Original".to_string()) as Arc<dyn Any + Send + Sync>,
        );
        let ctx = Context::new_with_all(None, HashMap::new(), extensions);

        let value = ctx.get::<String>();
        assert_eq!(value.unwrap(), "Original");

        // Extensions are immutable - cannot be modified after creation
        // This is by design to prevent accidental mutations
    }

    #[test]
    fn test_get_different_types() {
        let mut extensions = HashMap::new();
        extensions.insert(
            TypeId::of::<String>(),
            Arc::new("String value".to_string()) as Arc<dyn Any + Send + Sync>,
        );
        extensions.insert(
            TypeId::of::<i32>(),
            Arc::new(123i32) as Arc<dyn Any + Send + Sync>,
        );
        extensions.insert(
            TypeId::of::<bool>(),
            Arc::new(true) as Arc<dyn Any + Send + Sync>,
        );
        let ctx = Context::new_with_all(None, HashMap::new(), extensions);

        assert_eq!(ctx.get::<String>().unwrap(), "String value");
        assert_eq!(*ctx.get::<i32>().unwrap(), 123);
        assert!(*ctx.get::<bool>().unwrap());
    }

    #[test]
    fn test_get_wrong_type_returns_none() {
        let mut extensions = HashMap::new();
        extensions.insert(
            TypeId::of::<i64>(),
            Arc::new(42i64) as Arc<dyn Any + Send + Sync>,
        );
        let ctx = Context::new_with_all(None, HashMap::new(), extensions);

        // Try to get as different type
        let value = ctx.get::<String>();
        assert!(value.is_none());
    }

    #[test]
    fn test_get_custom_struct() {
        #[derive(Clone, Debug, PartialEq)]
        struct CustomData {
            name: String,
            count: i32,
        }

        let data = CustomData {
            name: "Test".to_string(),
            count: 100,
        };

        let mut extensions = HashMap::new();
        extensions.insert(
            TypeId::of::<CustomData>(),
            Arc::new(data.clone()) as Arc<dyn Any + Send + Sync>,
        );
        let ctx = Context::new_with_all(None, HashMap::new(), extensions);

        let retrieved = ctx.get::<CustomData>();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &data);
    }

    #[test]
    fn test_extensions_preserved_when_cloned() {
        let mut extensions = HashMap::new();
        extensions.insert(
            TypeId::of::<String>(),
            Arc::new("Extension value".to_string()) as Arc<dyn Any + Send + Sync>,
        );
        let ctx = Context::new_with_all(None, HashMap::new(), extensions);

        let cloned = ctx.clone();

        // Extension should be preserved
        assert_eq!(cloned.get::<String>().unwrap(), "Extension value");
    }

    #[test]
    fn test_complete_context_with_all_fields() {
        let mut extensions = HashMap::new();
        extensions.insert(
            TypeId::of::<String>(),
            Arc::new("Test data".to_string()) as Arc<dyn Any + Send + Sync>,
        );
        extensions.insert(
            TypeId::of::<i64>(),
            Arc::new(42i64) as Arc<dyn Any + Send + Sync>,
        );

        let mut permissions = HashMap::new();
        permissions.insert((1, None), 0b0011);

        let ctx = Context::new_with_all(Some(999), permissions, extensions);

        assert_eq!(ctx.user_id(), Some(999));
        assert_eq!(ctx.get::<String>().unwrap(), "Test data");
        assert_eq!(*ctx.get::<i64>().unwrap(), 42);
        assert!(ctx.has_access(None, 1, 0b0001));
    }
}
