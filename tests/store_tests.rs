use redis_clone::store::Store;

#[tokio::test]
async fn set_then_get_returns_value() {
    //create a fresh store instance
    let store = Store::new();

    //Store a key value pair
    store.set("name".to_string(), "shady".to_string()).await;

    //Read the value back
    let value = store.get("name").await;

    //The stored value should be retuirned
    assert_eq!(value.unwrap(), "shady".to_string());
}

#[tokio::test]
async fn get_missing_key_returns_none() {
    // Create a fresh empty store.
    let store = Store::new();

    // Missing keys should return None.
    let value = store.get("missing").await;

    assert_eq!(value, None);
}

#[tokio::test]
async fn del_existing_key_returns_true() {
    // Create a fresh store.
    let store = Store::new();

    // Insert a key first.
    store.set("name".to_string(), "shady".to_string()).await;

    // Delete should return true because the key existed.
    let deleted = store.del("name").await;

    assert_eq!(deleted, true);
    assert_eq!(store.get("name").await, None);
}

#[tokio::test]
async fn del_missing_key_returns_false() {
    // Create a fresh empty store.
    let store = Store::new();

    // Delete should return false because the key did not exist.
    let deleted = store.del("missing").await;

    assert_eq!(deleted, false);
}

#[tokio::test]
async fn exists_returns_true_for_existing_key() {
    // Create a fresh store.
    let store = Store::new();

    // Insert a key.
    store.set("name".to_string(), "shady".to_string()).await;

    // The key should exist.
    assert_eq!(store.exists("name").await, true);
}

#[tokio::test]
async fn incr_missing_key_starts_at_one() {
    // Create a fresh store.
    let store = Store::new();

    // Redis treats a missing key as 0, then increments to 1.
    let value = store.incr("count").await;

    assert_eq!(value, Ok(1));
    assert_eq!(store.get("count").await, Some("1".to_string()));
}

#[tokio::test]
async fn decr_missing_key_starts_at_negative_one() {
    // Create a fresh store.
    let store = Store::new();

    // Redis treats a missing key as 0, then decrements to -1.
    let value = store.decr("count").await;

    assert_eq!(value, Ok(-1));
    assert_eq!(store.get("count").await, Some("-1".to_string()));
}

#[tokio::test]
async fn incr_non_integer_returns_error() {
    // Create a fresh store.
    let store = Store::new();

    // Store a non-integer value.
    store.set("name".to_string(), "shady".to_string()).await;

    // INCR should fail because "shady" is not an integer.
    let result = store.incr("name").await;

    assert_eq!(
        result,
        Err("value is not an integer or out of range".to_string())
    );
}

#[tokio::test]
async fn del_many_returns_removed_count() {
    // Create a fresh store.
    let store = Store::new();

    // Insert two keys.
    store.set("a".to_string(), "1".to_string()).await;
    store.set("b".to_string(), "2".to_string()).await;

    let keys = vec!["a".to_string(), "b".to_string(), "c".to_string()];

    // Only a and b exist, so removed count should be 2.
    let removed = store.del_many(&keys).await;

    assert_eq!(removed, 2);
}

#[tokio::test]
async fn exists_many_returns_existing_count() {
    // Create a fresh store.
    let store = Store::new();

    // Insert two keys.
    store.set("a".to_string(), "1".to_string()).await;
    store.set("b".to_string(), "2".to_string()).await;

    let keys = vec!["a".to_string(), "b".to_string(), "c".to_string()];

    // Only a and b exist, so count should be 2.
    let count = store.exists_many(&keys).await;

    assert_eq!(count, 2);
}

#[tokio::test]
async fn incr_overflow_returns_error() {
    // Create a fresh store.
    let store = Store::new();

    // Store the maximum i64 value.
    store
        .set("count".to_string(), i64::MAX.to_string())
        .await;

    // Incrementing i64::MAX would overflow.
    let result = store.incr("count").await;

    assert_eq!(
        result,
        Err("increment or decrement would overflow".to_string())
    );
}

#[tokio::test]
async fn decr_overflow_returns_error() {
    // Create a fresh store.
    let store = Store::new();

    // Store the minimum i64 value.
    store
        .set("count".to_string(), i64::MIN.to_string())
        .await;

    // Decrementing i64::MIN would overflow.
    let result = store.decr("count").await;

    assert_eq!(
        result,
        Err("increment or decrement would overflow".to_string())
    );
}