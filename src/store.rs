use std::collections::HashMap;
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
}
