use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Entry{
    pub value:String,
    pub expires_at: Option<Instant>
}

#[derive(Clone)]
pub struct Store {
    inner: Arc<RwLock<HashMap<String, Entry>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set(&self, key: String, value: String) {
        let mut db = self.inner.write().await;
        let entry = Entry {
            value,
            expires_at: None
        };
        db.insert(key, entry);
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let mut db = self.inner.write().await;
        //look up the entry
        let entry = db.get(key)?;
        
        // If the key has an expiry and that time has passed, remove it.
        if let Some(expires_at) = entry.expires_at {
            if Instant::now() >= expires_at {
                db.remove(key);
                return None
            }
        }

        // Return a clone copy of the stored string.
        db.get(key).map(|entry| entry.value.clone())

    }

    pub async fn del(&self, key: &str) -> bool {
        let mut db = self.inner.write().await;
        db.remove(key).is_some()
    }

    pub async fn del_many(&self, keys: &[String]) -> i64 {
        // Write lock because we are removing keys.
        let mut db = self.inner.write().await;

        // Count how many keys were actually removed.
        let mut removed = 0;

        for key in keys {
            if db.remove(key).is_some() {
                removed += 1;
            }
        }

        removed
    }

    pub async fn exists_many(&self, keys: &[String]) -> i64 {
        // Write lock because checking may remove expired keys.
        let mut db = self.inner.write().await;

        let mut count = 0;

        for key in keys {
            let Some(entry) = db.get(key) else {
                continue;
            };

            if Self::is_expired(entry) {
                db.remove(key);
            } else {
                count += 1;
            }
        }

        count
    }

    pub async fn exists(&self, key: &str) -> bool {
        // Write lock because we may remove an expired key.
        let mut db = self.inner.write().await;

        // Check whether the key exists.
        let Some(entry) = db.get(key) else {
            return false;
        };

        // If it exists but is expired, remove it and report false.
        if Self::is_expired(entry) {
            db.remove(key);
            return false;
        }

        true
    }

    pub async fn incr(&self, key: &str) -> Result<i64, String> {
        //we need a write lock because INCR may insert or update
        let mut db = self.inner.write().await;

        // If key exists but expired, remove it first.
        if let Some(entry) = db.get(key) {
            if Self::is_expired(entry) {
                db.remove(key);
            }
        }

        // If the key exists, parse it as an integer.
        // If it does not exist, Redis treats it as 0.
        let current = match db.get(key) {
            Some(entry) => entry
                .value
                .parse::<i64>()
                .map_err(|_| "value is not an integer or out of range".to_string())?,

            None => 0,
        };

        //Add one to the current value.
        let next = current
            .checked_add(1)
            .ok_or_else(|| "increment or decrement would overflow".to_string())?;

        // Store the new number as a string.
        // INCR preserves expiry in real Redis if the key already exists.
        let expires_at = db.get(key).and_then(|entry| entry.expires_at);

        db.insert(
            key.to_string(),
            Entry { value: next.to_string(), expires_at });

        Ok(next)
    }

    pub async fn decr(&self, key: &str) -> Result<i64, String> {
        // We need a write lock because DECR may insert or update the key.
        let mut db = self.inner.write().await;

        // If key exists but expired, remove it first.
        if let Some(entry) = db.get(key) {
            if Self::is_expired(entry) {
                db.remove(key);
            }
        }

        // If the key exists, parse it as an integer.
        // If it does not exist, Redis treats it as 0.
        let current = match db.get(key) {
            Some(entry) => entry
                .value
                .parse::<i64>()
                .map_err(|_| "value is not an integer or out of range".to_string())?,

            None => 0,
        };

        // Subtract one and store the result as a string.
        let next = current
            .checked_sub(1)
            .ok_or_else(|| "increment or decrement would overflow".to_string())?;

        // DECR preserves expiry in real Redis if the key already exists.
        let expires_at = db.get(key).and_then(|entry| entry.expires_at);
        
        db.insert(
        key.to_string(),
        Entry { 
                value: next.to_string(),
                expires_at 
            },
        );

        Ok(next)
    }

    // Returns true if the entry has expired.
    fn is_expired(entry: &Entry) -> bool {
        // If expires_at is Some(time), compare it with now.
        match entry.expires_at {
            Some(expires_at) => Instant::now() >= expires_at,
            None => false,
        }
    }
}
