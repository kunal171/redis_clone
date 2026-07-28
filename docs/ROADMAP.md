# Roadmap

This roadmap keeps the project moving in small, useful milestones.

## Milestone 1: Basic RESP Server

Status: done

- Start a TCP server with Tokio.
- Accept multiple clients.
- Encode RESP responses.
- Respond to `PING`.

## Milestone 2: Command Layer And Store

Status: done

- Parse RESP arrays and bulk strings.
- Convert parsed RESP into typed commands.
- Add shared in-memory store.
- Support `PING`, `ECHO`, `SET`, `GET`, `DEL`, and `EXISTS`.

## Milestone 3: Numeric Commands

Status: in progress

- Support `INCR`.
- Support `DECR`.
- Return Redis-style integer responses.
- Return Redis-style errors for non-integer values.

Next improvements:

- handle integer overflow with checked arithmetic
- extract shared increment/decrement logic to reduce duplication

## Milestone 4: Automated Tests

Status: not started

Add tests for:

- RESP encoding
- RESP parsing
- command parsing
- command execution
- store operations

Suggested files:

```text
tests/
  resp_tests.rs
  command_tests.rs
  store_tests.rs
```

## Milestone 5: Better Request Framing

Status: not started

The current server assumes one complete command arrives in one read.

Real TCP does not guarantee that.

Future parser/server work:

- keep a per-client read buffer
- parse one command at a time from the buffer
- leave incomplete commands in the buffer
- support command pipelining

Pipelining means a client can send multiple commands before reading responses.

Example:

```text
PING
PING
GET name
```

All sent together over one TCP connection.

## Milestone 6: Expiration

Status: not started

Add:

```text
EXPIRE key seconds
TTL key
SET key value EX seconds
```

Storage will need to change from:

```rust
HashMap<String, String>
```

To something like:

```rust
struct Entry {
    value: String,
    expires_at: Option<Instant>,
}
```

Then:

```rust
HashMap<String, Entry>
```

## Milestone 7: Persistence

Status: not started

Start with append-only persistence.

Idea:

- every write command is appended to a file
- on startup, replay the file
- after replay, the in-memory store is restored

Commands to persist:

- `SET`
- `DEL`
- `INCR`
- `DECR`
- expiration commands once added

## Milestone 8: More Data Structures

Status: not started

Redis supports many data types. Good next structures:

- lists
- sets
- hashes

Possible commands:

```text
LPUSH
RPOP
SADD
SISMEMBER
HSET
HGET
```

The store will need a richer value enum:

```rust
enum Value {
    String(String),
    List(VecDeque<String>),
    Set(HashSet<String>),
    Hash(HashMap<String, String>),
}
```

