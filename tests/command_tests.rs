use redis_clone::command::Command;
use redis_clone::resp::Resp;
use redis_clone::store::Store;

#[test]
fn parses_ping_command() {
    // A Redis PING command is an array with one bulk string.
    let resp = Resp::Array(vec![Resp::BulkString(b"PING".to_vec())]);

    // Convert RESP into our typed command enum.
    let command = Command::from_resp(resp);

    assert_eq!(command, Command::Ping);
}

#[test]
fn parses_echo_command() {
    // ECHO has one argument.
    let resp = Resp::Array(vec![
        Resp::BulkString(b"ECHO".to_vec()),
        Resp::BulkString(b"hello".to_vec()),
    ]);

    let command = Command::from_resp(resp);

    assert_eq!(command, Command::Echo("hello".to_string()));
}

#[test]
fn parses_set_command() {
    // SET has key and value arguments.
    let resp = Resp::Array(vec![
        Resp::BulkString(b"SET".to_vec()),
        Resp::BulkString(b"name".to_vec()),
        Resp::BulkString(b"shady".to_vec()),
    ]);

    let command = Command::from_resp(resp);

    assert_eq!(
        command,
        Command::Set {
            key: "name".to_string(),
            value: "shady".to_string()
        }
    );
}

#[test]
fn unknown_command_returns_unknown() {
    // Unknown command names should not crash the parser.
    let resp = Resp::Array(vec![Resp::BulkString(b"NOPE".to_vec())]);

    let command = Command::from_resp(resp);

    assert_eq!(
        command,
        Command::Unknown("unknown command: NOPE".to_string())
    );
}

#[tokio::test]
async fn execute_ping_returns_pong() {
    // Create a store even though PING does not use it.
    let store = Store::new();

    let response = Command::Ping.execute(store).await;

    assert_eq!(response, Resp::SimpleString("PONG".to_string()));
}

#[tokio::test]
async fn execute_set_then_get_returns_value() {
    // Commands should share the same store.
    let store = Store::new();

    let set_response = Command::Set {
        key: "name".to_string(),
        value: "shady".to_string(),
    }
    .execute(store.clone())
    .await;

    assert_eq!(set_response, Resp::SimpleString("OK".to_string()));

    let get_response = Command::Get {
        key: "name".to_string(),
    }
    .execute(store)
    .await;

    assert_eq!(get_response, Resp::BulkString(b"shady".to_vec()));
}

#[tokio::test]
async fn execute_incr_returns_incremented_integer() {
    // INCR should mutate the store and return the new number.
    let store = Store::new();

    let first = Command::Incr {
        key: "count".to_string(),
    }
    .execute(store.clone())
    .await;

    let second = Command::Incr {
        key: "count".to_string(),
    }
    .execute(store)
    .await;

    assert_eq!(first, Resp::Integer(1));
    assert_eq!(second, Resp::Integer(2));
}


#[test]
fn parses_expire_command() {
    let resp = Resp::Array(vec![
        Resp::BulkString(b"EXPIRE".to_vec()),
        Resp::BulkString(b"session".to_vec()),
        Resp::BulkString(b"3".to_vec()),
    ]);

    let command = Command::from_resp(resp);

    assert_eq!(
        command,
        Command::Expire {
            key: "session".to_string(),
            seconds: 3,
        }
    );
}

#[test]
fn parses_ttl_command() {
    let resp = Resp::Array(vec![
        Resp::BulkString(b"TTL".to_vec()),
        Resp::BulkString(b"session".to_vec()),
    ]);

    let command = Command::from_resp(resp);

    assert_eq!(
        command,
        Command::Ttl {
            key: "session".to_string(),
        }
    );
}