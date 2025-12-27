use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Context provides request-scoped state for operations.
///
/// It stores:
/// - User ID (optional)
/// - Permissions (resource + branch access)
/// - Arbitrary typed extensions via `set`/`get`
///
/// # Examples
///
/// ```rust
/// use sultan_core::domain::Context;
///
/// let mut ctx = Context::new().with_user_id(123);
///
/// // Store custom data
/// ctx.set("request-id".to_string());
/// ctx.set(42i64);
///
/// // Retrieve typed data
/// let request_id: &String = ctx.get::<String>().unwrap();
/// let count: &i64 = ctx.get::<i64>().unwrap();
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

    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_permission(&self, permission: HashMap<(i32, Option<i64>), i32>) -> Self {
        Self {
            user_id: self.user_id,
            permission,
            extensions: self.extensions.clone(),
        }
    }

    /// Set a value of any type in the context.
    /// The value must implement Clone + Send + Sync + 'static.
    pub fn set<T: Clone + Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.insert(TypeId::of::<T>(), Arc::new(value));
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

        let ctx = Context::new().with_permission(permissions);

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

        let ctx = Context::new().with_permission(permissions);

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

        let ctx = Context::new().with_permission(permissions);

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

        let ctx = Context::new().with_permission(permissions);

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

        let ctx = Context::new().with_permission(permissions);

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

        let ctx = Context::new().with_permission(permissions);

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
    fn test_user_id_preserved_with_permissions() {
        let mut ctx = Context::new();
        ctx.user_id = Some(123);

        let mut permissions = std::collections::HashMap::new();
        permissions.insert((1, None), 0b1111);

        let child = ctx.with_permission(permissions);
        assert_eq!(child.user_id(), Some(123));
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
    fn test_with_user_id_builder() {
        let ctx = Context::new().with_user_id(999);
        assert_eq!(ctx.user_id(), Some(999));
    }

    #[test]
    fn test_with_user_id_chain_with_permissions() {
        let ctx = Context::new().with_user_id(777);

        let mut permissions = std::collections::HashMap::new();
        permissions.insert((1, None), 0b0011);

        let child = ctx.with_permission(permissions);

        assert_eq!(child.user_id(), Some(777));
        assert!(child.has_access(None, 1, 0b0001));
    }

    #[test]
    fn test_with_user_id_multiple_contexts() {
        let ctx1 = Context::new().with_user_id(111);
        let ctx2 = Context::new().with_user_id(222);
        let ctx3 = Context::new(); // No user_id set

        assert_eq!(ctx1.user_id(), Some(111));
        assert_eq!(ctx2.user_id(), Some(222));
        assert_eq!(ctx3.user_id(), None);
    }

    #[test]
    fn test_set_and_get_string() {
        let mut ctx = Context::new();
        ctx.set("Hello, World!".to_string());

        let value = ctx.get::<String>();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_set_and_get_integer() {
        let mut ctx = Context::new();
        ctx.set(42i64);

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
    fn test_set_overwrites_existing_value() {
        let mut ctx = Context::new();
        ctx.set("First".to_string());
        ctx.set("Second".to_string());

        let value = ctx.get::<String>();
        assert_eq!(value.unwrap(), "Second");
    }

    #[test]
    fn test_set_different_types() {
        let mut ctx = Context::new();
        ctx.set("String value".to_string());
        ctx.set(123i32);
        ctx.set(true);

        assert_eq!(ctx.get::<String>().unwrap(), "String value");
        assert_eq!(*ctx.get::<i32>().unwrap(), 123);
        assert_eq!(*ctx.get::<bool>().unwrap(), true);
    }

    #[test]
    fn test_get_wrong_type_returns_none() {
        let mut ctx = Context::new();
        ctx.set(42i64);

        // Try to get as different type
        let value = ctx.get::<String>();
        assert!(value.is_none());
    }

    #[test]
    fn test_set_custom_struct() {
        #[derive(Clone, Debug, PartialEq)]
        struct CustomData {
            name: String,
            count: i32,
        }

        let mut ctx = Context::new();
        let data = CustomData {
            name: "Test".to_string(),
            count: 100,
        };
        ctx.set(data.clone());

        let retrieved = ctx.get::<CustomData>();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &data);
    }

    #[test]
    fn test_extensions_preserved_with_permission() {
        let mut ctx = Context::new();
        ctx.set("Extension value".to_string());

        let mut permissions = HashMap::new();
        permissions.insert((1, None), 0b1111);

        let child = ctx.with_permission(permissions);

        // Extension should be preserved
        assert_eq!(child.get::<String>().unwrap(), "Extension value");
    }

    #[test]
    fn test_extensions_with_user_id_and_permissions() {
        let mut ctx = Context::new().with_user_id(999);
        ctx.set("Test data".to_string());
        ctx.set(42i64);

        let mut permissions = HashMap::new();
        permissions.insert((1, None), 0b0011);

        let child = ctx.with_permission(permissions);

        assert_eq!(child.user_id(), Some(999));
        assert_eq!(child.get::<String>().unwrap(), "Test data");
        assert_eq!(*child.get::<i64>().unwrap(), 42);
        assert!(child.has_access(None, 1, 0b0001));
    }
}
