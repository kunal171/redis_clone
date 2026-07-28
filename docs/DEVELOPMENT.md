# Development Guide

This document contains common development commands and testing notes.

## Run The Server

```bash
cargo run
```

Expected startup log:

```text
redis_clone listening on 127.0.0.1:9000
```

Keep this terminal open while testing.

## Test With redis-cli

In another terminal:

```bash
redis-cli -p 9000 ping
```

Expected:

```text
PONG
```

Run a fuller smoke test:

```bash
redis-cli -p 9000 set name shady
redis-cli -p 9000 get name
redis-cli -p 9000 exists name
redis-cli -p 9000 del name
redis-cli -p 9000 get name
```

Expected:

```text
OK
"shady"
(integer) 1
(integer) 1
(nil)
```

## Test Without redis-cli

You can use `nc` to send raw RESP bytes.

PING:

```bash
printf '*1\r\n$4\r\nPING\r\n' | nc 127.0.0.1 9000
```

SET:

```bash
printf '*3\r\n$3\r\nSET\r\n$4\r\nname\r\n$5\r\nshady\r\n' | nc 127.0.0.1 9000
```

GET:

```bash
printf '*2\r\n$3\r\nGET\r\n$4\r\nname\r\n' | nc 127.0.0.1 9000
```

## Format

```bash
cargo fmt
```

Check formatting without changing files:

```bash
cargo fmt --check
```

## Check Compilation

```bash
cargo check
```

## Build

```bash
cargo build
```

## Run Tests

There are no automated tests yet. When tests are added, run:

```bash
cargo test
```

Suggested first tests:

- RESP encoding
- RESP parsing
- command parsing
- store behavior

## Debugging Protocol Input

The server currently prints incoming bytes:

```rust
println!("Received bytes: {input:?}");
```

This is useful while learning RESP because it shows exactly what `redis-cli` sent.

Example:

```text
Received bytes: [42, 49, 13, 10, 36, 52, 13, 10, 80, 73, 78, 71, 13, 10]
```

Those bytes are:

```text
*1\r\n$4\r\nPING\r\n
```

## Common Issues

### Port Already In Use

If `cargo run` fails because port `9000` is already in use, another copy of the
server may still be running.

Find it:

```bash
lsof -i :9000
```

Or:

```bash
ss -ltnp | grep 9000
```

Stop the older process, then run again.

### redis-cli Cannot Connect

Check that the server is running:

```bash
nc -vz 127.0.0.1 9000
```

Expected:

```text
Connection to 127.0.0.1 9000 port [tcp/*] succeeded!
```

### ping Is Not A Redis Test

This checks whether the host is reachable:

```bash
ping 127.0.0.1
```

It does not check your Redis clone.

Use this instead:

```bash
redis-cli -p 9000 ping
```

Or:

```bash
printf '*1\r\n$4\r\nPING\r\n' | nc 127.0.0.1 9000
```

