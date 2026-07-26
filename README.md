# redis_clone

A small Redis-like server written in Rust.

This project is currently at the first networking/protocol milestone: it starts a TCP
server, accepts Redis-style RESP input, and responds to `PING` with `PONG`.

## What Works Now

- Starts a TCP server on `127.0.0.1:9000`.
- Accepts multiple client connections with Tokio.
- Encodes RESP responses.
- Responds to Redis-style `PING` commands.
- Contains an initial in-memory store wrapper using `Arc<RwLock<HashMap<...>>>`.

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
```

Expected output:

```text
PONG
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
  server.rs  # Accepts TCP clients and responds to basic commands.
  resp.rs    # Defines RESP values and encodes them into Redis-compatible bytes.
  store.rs   # Holds the future shared in-memory key-value store.
```

## Next Steps

1. Replace the temporary `contains("PING")` check with a real RESP parser.
2. Convert parsed RESP arrays into commands such as `PING`, `SET`, and `GET`.
3. Wire `Store` into the server so clients can save and read keys.
4. Add support for `DEL`, `EXISTS`, `INCR`, and key expiration.
5. Add tests for RESP encoding, parsing, command handling, and store behavior.

