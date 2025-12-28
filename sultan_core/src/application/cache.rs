use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::domain::DomainResult;

/// Trait for cache operations with generic key type
#[async_trait]
pub trait CacheService<K>: Send + Sync
where
    K: Send + Sync + std::hash::Hash + Eq + Clone,
{
    async fn get<T: Clone + Send + Sync + 'static>(&self, key: &K) -> Option<T>;
    async fn set<T: Clone + Send + Sync + 'static>(
        &self,
        key: &K,
        value: T,
        ttl: Duration,
    ) -> DomainResult<()>;
    async fn delete(&self, key: &K) -> DomainResult<()>;
    async fn clear(&self) -> DomainResult<()>;
}

#[derive(Clone)]
struct CacheEntry {
    value: Arc<dyn std::any::Any + Send + Sync>,
    expires_at: Instant,
}

/// In-memory cache implementation with generic key type
pub struct InMemoryCache<K>
where
    K: Send + Sync + std::hash::Hash + Eq + Clone,
{
    store: Arc<RwLock<HashMap<K, CacheEntry>>>,
}

impl<K> InMemoryCache<K>
where
    K: Send + Sync + std::hash::Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<K> Default for InMemoryCache<K>
where
    K: Send + Sync + std::hash::Hash + Eq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<K> CacheService<K> for InMemoryCache<K>
where
    K: Send + Sync + std::hash::Hash + Eq + Clone + 'static,
{
    async fn get<T: Clone + Send + Sync + 'static>(&self, key: &K) -> Option<T> {
        let store = self.store.read().await;
        let entry = store.get(key)?;

        // Check if expired
        if entry.expires_at <= Instant::now() {
            drop(store);
            let _ = self.delete(key).await;
            return None;
        }

        // Try to downcast and clone
        entry.value.downcast_ref::<T>().cloned()
    }

    async fn set<T: Clone + Send + Sync + 'static>(
        &self,
        key: &K,
        value: T,
        ttl: Duration,
    ) -> DomainResult<()> {
        let mut store = self.store.write().await;
        let entry = CacheEntry {
            value: Arc::new(value),
            expires_at: Instant::now() + ttl,
        };
        store.insert(key.clone(), entry);
        Ok(())
    }

    async fn delete(&self, key: &K) -> DomainResult<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }

    async fn clear(&self) -> DomainResult<()> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache = InMemoryCache::<String>::new();
        let value = vec![1, 2, 3, 4, 5];

        cache
            .set(
                &"test_key".to_string(),
                value.clone(),
                Duration::from_secs(60),
            )
            .await
            .unwrap();

        let retrieved: Option<Vec<i32>> = cache.get(&"test_key".to_string()).await;
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_cache_get_nonexistent() {
        let cache = InMemoryCache::<String>::new();
        let retrieved: Option<String> = cache.get(&"nonexistent".to_string()).await;
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = InMemoryCache::<String>::new();
        cache
            .set(
                &"test_key".to_string(),
                "value".to_string(),
                Duration::from_millis(100),
            )
            .await
            .unwrap();

        // Should exist immediately
        let retrieved: Option<String> = cache.get(&"test_key".to_string()).await;
        assert_eq!(retrieved, Some("value".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        let retrieved: Option<String> = cache.get(&"test_key".to_string()).await;
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache = InMemoryCache::<String>::new();
        cache
            .set(
                &"test_key".to_string(),
                "value".to_string(),
                Duration::from_secs(60),
            )
            .await
            .unwrap();

        cache.delete(&"test_key".to_string()).await.unwrap();

        let retrieved: Option<String> = cache.get(&"test_key".to_string()).await;
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = InMemoryCache::<String>::new();
        cache
            .set(
                &"key1".to_string(),
                "value1".to_string(),
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        cache
            .set(
                &"key2".to_string(),
                "value2".to_string(),
                Duration::from_secs(60),
            )
            .await
            .unwrap();

        cache.clear().await.unwrap();

        let retrieved1: Option<String> = cache.get(&"key1".to_string()).await;
        let retrieved2: Option<String> = cache.get(&"key2".to_string()).await;
        assert_eq!(retrieved1, None);
        assert_eq!(retrieved2, None);
    }

    #[tokio::test]
    async fn test_cache_different_types() {
        let cache = InMemoryCache::<String>::new();

        cache
            .set(
                &"string_key".to_string(),
                "text".to_string(),
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        cache
            .set(&"int_key".to_string(), 42i64, Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set(
                &"vec_key".to_string(),
                vec![1, 2, 3],
                Duration::from_secs(60),
            )
            .await
            .unwrap();

        let string_val: Option<String> = cache.get(&"string_key".to_string()).await;
        let int_val: Option<i64> = cache.get(&"int_key".to_string()).await;
        let vec_val: Option<Vec<i32>> = cache.get(&"vec_key".to_string()).await;

        assert_eq!(string_val, Some("text".to_string()));
        assert_eq!(int_val, Some(42));
        assert_eq!(vec_val, Some(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_cache_overwrite() {
        let cache = InMemoryCache::<String>::new();

        cache
            .set(
                &"key".to_string(),
                "first".to_string(),
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        cache
            .set(
                &"key".to_string(),
                "second".to_string(),
                Duration::from_secs(60),
            )
            .await
            .unwrap();

        let retrieved: Option<String> = cache.get(&"key".to_string()).await;
        assert_eq!(retrieved, Some("second".to_string()));
    }

    #[tokio::test]
    async fn test_cache_type_safety() {
        let cache = InMemoryCache::<String>::new();

        cache
            .set(&"key".to_string(), 42i64, Duration::from_secs(60))
            .await
            .unwrap();

        // Try to get as wrong type
        let retrieved: Option<String> = cache.get(&"key".to_string()).await;
        assert_eq!(retrieved, None);

        // Correct type should work
        let retrieved: Option<i64> = cache.get(&"key".to_string()).await;
        assert_eq!(retrieved, Some(42));
    }

    #[tokio::test]
    async fn test_cache_with_i64_key() {
        let cache = InMemoryCache::<i64>::new();

        // Use i64 as key directly
        cache
            .set(&123i64, "user_data".to_string(), Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set(&456i64, vec![1, 2, 3], Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved1: Option<String> = cache.get(&123i64).await;
        let retrieved2: Option<Vec<i32>> = cache.get(&456i64).await;

        assert_eq!(retrieved1, Some("user_data".to_string()));
        assert_eq!(retrieved2, Some(vec![1, 2, 3]));

        // Delete one key
        cache.delete(&123i64).await.unwrap();
        let retrieved: Option<String> = cache.get(&123i64).await;
        assert_eq!(retrieved, None);

        // Other key should still exist
        let retrieved: Option<Vec<i32>> = cache.get(&456i64).await;
        assert_eq!(retrieved, Some(vec![1, 2, 3]));
    }
}
