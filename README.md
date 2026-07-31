# redis_clone

A small Redis-like in-memory key-value server written in Rust.

This project is a learning-focused Redis clone. It speaks the Redis Serialization
Protocol (RESP), accepts TCP clients with Tokio, and supports a growing set of
Redis-style commands backed by an in-memory store.

## What Works Now

- Starts a TCP server on `127.0.0.1:9000`.
- Accepts multiple client connections with Tokio.
- Parses RESP arrays and bulk strings.
- Keeps a per-client read buffer for partial reads and pipelined commands.
- Encodes RESP responses.
- Supports `PING`, `ECHO`, `SET`, `GET`, `DEL`, `EXISTS`, `INCR`, `DECR`,
  `EXPIRE`, and `TTL`.
- Supports multi-key `DEL` and `EXISTS`.
- Stores string keys and values in memory with optional expiration times.
- Shares one store across client connections.
- Has integration tests for the store, RESP parser/encoder, and command layer.

## Run

```bash
cargo run
```

The server should print:

```text
redis_clone listening on 127.0.0.1:9000
```

## Test With redis-cli

```bash
redis-cli -p 9000 ping
redis-cli -p 9000 echo hello
redis-cli -p 9000 set name shady
redis-cli -p 9000 get name
redis-cli -p 9000 incr count
redis-cli -p 9000 decr count
redis-cli -p 9000 exists name count missing
redis-cli -p 9000 expire name 3
redis-cli -p 9000 ttl name
redis-cli -p 9000 del name count missing
```

Expected output:

```text
PONG
"hello"
OK
"shady"
(integer) 1
(integer) 0
(integer) 2
(integer) 1
(integer) 2 or 3
(integer) 2
```

## Test Without redis-cli

Use `nc` to send a raw RESP command:

```bash
printf '*1\r\n$4\r\nPING\r\n' | nc 127.0.0.1 9000
```

Expected output:

```text
+PONG
```

## Current Architecture

```text
src/
  main.rs    # Starts the async Tokio runtime and launches the server.
  server.rs  # Accepts TCP clients, parses RESP input, and writes responses.
  resp.rs    # Defines RESP values and parses/encodes Redis-compatible bytes.
  command.rs # Converts RESP arrays into commands and executes them.
  store.rs   # Holds the shared in-memory key-value store.
```

## Learning Notes

- [Redis Protocol Notes](docs/RESP.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Supported Commands](docs/COMMANDS.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Roadmap](docs/ROADMAP.md)

## Next Steps

1. Add append-only persistence and startup replay.
2. Add `SET key value EX seconds` syntax.
3. Extract shared numeric mutation logic for `INCR` and `DECR`.
4. Add more command tests for `DEL`, `EXISTS`, `DECR`, `EXPIRE`, and `TTL`.
5. Add richer data types like lists, sets, and hashes.
