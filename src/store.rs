use std::collections::HashMap;
use std::collections::btree_map::Values;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Store {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set(&self, key: String, value: String) {
        let mut db = self.inner.write().await;
        db.insert(key, value);
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let db = self.inner.read().await;
        db.get(key).cloned()
    }

    pub async fn del(&self, key: &str) -> bool {
        let mut db = self.inner.write().await;
        db.remove(key).is_some()
    }

    pub async fn exists(&self, key: &str) -> bool {
        //Read lock is enough because we are not modifying map
        let db = self.inner.read().await;
        db.contains_key(key)
    }

    pub async fn incr(&self, key: &str) -> Result<i64, String> {
        //we need a write lock because INCR may insert or update
        let mut db = self.inner.write().await;
        // If the key exists, parse it as an integer.
        // If it does not exist, Redis treats it as 0.

        let current = match db.get(key) {
            Some(value) => value
                .parse::<i64>()
                .map_err(|_| "value is not an integer or out of range".to_string())?,
            
            None => 0,
        };

        //Add one to the current value.
        let next = current + 1;
        db.insert(key.to_string(), next.to_string());

        Ok(next)
    }
}
