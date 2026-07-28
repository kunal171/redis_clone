# redis_clone

A small Redis-like in-memory key-value server written in Rust.

This project is a learning-focused Redis clone. It speaks the Redis Serialization
Protocol (RESP), accepts TCP clients with Tokio, and supports a growing set of
Redis-style commands backed by an in-memory store.

## What Works Now

- Starts a TCP server on `127.0.0.1:9000`.
- Accepts multiple client connections with Tokio.
- Parses RESP arrays and bulk strings.
- Encodes RESP responses.
- Supports `PING`, `ECHO`, `SET`, `GET`, `DEL`, and `EXISTS`.
- Stores string keys and values in memory using `Arc<RwLock<HashMap<...>>>`.
- Shares one store across client connections.

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
redis-cli -p 9000 exists name
redis-cli -p 9000 del name
redis-cli -p 9000 exists name
```

Expected output:

```text
PONG
"hello"
OK
"shady"
(integer) 1
(integer) 1
(integer) 0
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

## Next Steps

1. Add `INCR` and numeric string handling.
2. Support multiple-key variants like `DEL key [key ...]` and `EXISTS key [key ...]`.
3. Add key expiration with `EXPIRE` and `TTL`.
4. Add tests for RESP parsing, command handling, and store behavior.
5. Add append-only persistence and startup replay.
