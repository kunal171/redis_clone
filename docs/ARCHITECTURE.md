# Architecture

This project is a small Redis-like TCP server written in Rust.

The main goal is to learn how Redis-like systems are built:

- TCP networking
- protocol parsing
- command routing
- shared in-memory state
- async concurrency
- Redis-compatible responses

## Request Flow

A request moves through the server like this:

```text
client
  |
  | RESP bytes over TCP
  v
server.rs
  |
  | Resp::parse(...)
  v
resp.rs
  |
  | Command::from_resp(...)
  v
command.rs
  |
  | command.execute(store)
  v
store.rs
  |
  | Resp response
  v
resp.rs
  |
  | response.encode()
  v
server.rs
  |
  | bytes over TCP
  v
client
```

In code, the core server loop looks like this:

```rust
// Read raw bytes from the TCP stream.
let input = &buffer[..n];

// Parse the bytes, convert the parsed RESP into a command, and execute it.
let response = match Resp::parse(input) {
    Ok(resp) => Command::from_resp(resp).execute(store.clone()).await,
    Err(err) => Resp::Error(format!("ERR {err}")),
};

// Convert the response back into RESP bytes and send it to the client.
stream.write_all(&response.encode()).await?;
```

## Modules

```text
src/
  main.rs
  server.rs
  resp.rs
  command.rs
  store.rs
```

## main.rs

`main.rs` starts the program.

Responsibilities:

- create the shared store
- start the Tokio async runtime
- launch the TCP server

Shape:

```rust
// Create one shared in-memory store.
let store = Store::new();

// Start listening for TCP clients.
server::run("127.0.0.1:9000", store).await
```

## server.rs

`server.rs` owns networking.

Responsibilities:

- bind a `TcpListener`
- accept client connections
- spawn one Tokio task per client
- read bytes from each TCP stream
- write RESP responses back to each TCP stream

The server clones `Store` for each client task:

```rust
// Clone the store handle, not the actual HashMap.
let store = store.clone();

tokio::spawn(async move {
    // This client gets access to the same shared database.
    handle_client(stream, store).await
});
```

This works because `Store` internally uses:

```rust
Arc<RwLock<HashMap<String, String>>>
```

## resp.rs

`resp.rs` owns the Redis Serialization Protocol.

Responsibilities:

- define RESP values as a Rust enum
- parse raw bytes into RESP values
- encode RESP values back into bytes

The important enum is:

```rust
pub enum Resp {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<Resp>),
    Null,
}
```

Examples:

```text
+PONG\r\n                 -> Resp::SimpleString("PONG")
:1\r\n                    -> Resp::Integer(1)
$5\r\nshady\r\n           -> Resp::BulkString(...)
*1\r\n$4\r\nPING\r\n      -> Resp::Array(...)
$-1\r\n                   -> Resp::Null
```

## command.rs

`command.rs` owns application behavior.

Responsibilities:

- convert parsed RESP arrays into typed commands
- validate command argument counts
- execute commands against the store
- return RESP responses

The command layer turns this:

```rust
Resp::Array(vec![
    Resp::BulkString(b"SET".to_vec()),
    Resp::BulkString(b"name".to_vec()),
    Resp::BulkString(b"shady".to_vec()),
])
```

Into this:

```rust
Command::Set {
    key: "name".to_string(),
    value: "shady".to_string(),
}
```

This keeps protocol parsing separate from command behavior.

## store.rs

`store.rs` owns the in-memory database.

Responsibilities:

- store key-value pairs
- read values
- delete keys
- check key existence
- mutate numeric strings for `INCR` and `DECR`

Current storage type:

```rust
HashMap<String, String>
```

Shared wrapper:

```rust
Arc<RwLock<HashMap<String, String>>>
```

Why `Arc`:

- many client tasks need access to the same store
- `Arc` gives shared ownership

Why `RwLock`:

- many readers can access the map at the same time
- writes still get exclusive access

Examples:

```text
GET     -> read lock
EXISTS  -> read lock
SET     -> write lock
DEL     -> write lock
INCR    -> write lock
DECR    -> write lock
```

## Current Limitations

The architecture is intentionally simple. Current limitations:

- request reading uses a fixed 1024-byte buffer
- parser assumes one complete command arrives in one read
- no command pipelining yet
- no key expiration yet
- no persistence yet
- no authentication
- no clustering or replication

These are good future learning milestones.

