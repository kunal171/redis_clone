# Redis Protocol Notes

Redis clients and Redis servers communicate over TCP using a protocol called RESP.

RESP means:

```text
Redis Serialization Protocol
```

It is a set of rules for turning commands and responses into bytes. Your Rust
server does not know whether the client is `redis-cli`, `nc`, another Rust program,
or a real application. It only receives bytes over TCP. If those bytes follow RESP
rules, your server can parse them.

The connection looks like this:

```text
redis-cli  <--- RESP bytes over TCP --->  your Rust server
```

So we are not coding special support for `redis-cli`. We are coding support for
RESP. `redis-cli` works because it also speaks RESP.

## CRLF

RESP uses `\r\n` to mark the end of protocol lines.

```text
\r = carriage return
\n = newline
```

Together they are called CRLF.

Example:

```text
+PONG\r\n
```

This means:

```text
+       simple string response
PONG    response value
\r\n    end of response
```

Redis expects `\r\n`, not only `\n`.

## RESP Data Types

Every RESP value starts with a prefix byte.

```text
+ simple string
- error
: integer
$ bulk string
* array
```

Your Rust enum models these values:

```rust
// Represents values in the Redis Serialization Protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum Resp {
    // Example: +OK\r\n
    SimpleString(String),

    // Example: -ERR unknown command\r\n
    Error(String),

    // Example: :1\r\n
    Integer(i64),

    // Example: $5\r\nhello\r\n
    BulkString(Vec<u8>),

    // Example: *1\r\n$4\r\nPING\r\n
    Array(Vec<Resp>),

    // Example: $-1\r\n
    Null,
}
```

## Simple Strings

Simple strings are used for short successful responses.

Format:

```text
+<text>\r\n
```

Example:

```text
+OK\r\n
```

`redis-cli` displays:

```text
OK
```

Your server uses this for commands like:

```text
PING -> +PONG\r\n
SET  -> +OK\r\n
```

In Rust:

```rust
// Send a simple PONG response.
Resp::SimpleString("PONG".to_string())
```

Encoded:

```text
+PONG\r\n
```

## Errors

Errors start with `-`.

Format:

```text
-<message>\r\n
```

Example:

```text
-ERR unknown command\r\n
```

`redis-cli` displays:

```text
(error) ERR unknown command
```

In Rust:

```rust
// Send a Redis-style error response.
Resp::Error("ERR unknown command".to_string())
```

The `ERR` prefix is part of Redis convention. Your server should usually include it
in command errors:

```rust
// Good Redis-style error response.
Resp::Error(format!("ERR {message}"))
```

## Integers

Integers start with `:`.

Format:

```text
:<number>\r\n
```

Example:

```text
:1\r\n
```

`redis-cli` displays:

```text
(integer) 1
```

Your server uses integers for:

```text
DEL
EXISTS
INCR
DECR
```

Examples:

```text
DEL name       -> :1\r\n if deleted
DEL missing    -> :0\r\n if not found
EXISTS name    -> :1\r\n if exists
INCR count     -> :1\r\n, :2\r\n, :3\r\n, ...
DECR count     -> :-1\r\n, :-2\r\n, :-3\r\n, ...
```

In Rust:

```rust
// Send integer 1.
Resp::Integer(1)
```

## Bulk Strings

Bulk strings are binary-safe strings. Redis uses them for command names, command
arguments, and stored values.

Format:

```text
$<length>\r\n<data>\r\n
```

Example:

```text
$5\r\nshady\r\n
```

This means:

```text
$5      the next string has 5 bytes
shady   the actual string data
\r\n    end of string
```

`GET name` returns a bulk string when the key exists.

In Rust:

```rust
// Send the value "shady" as a bulk string.
Resp::BulkString(b"shady".to_vec())
```

Encoded:

```text
$5\r\nshady\r\n
```

### Null Bulk String

If a key does not exist, Redis returns null:

```text
$-1\r\n
```

`redis-cli` displays:

```text
(nil)
```

In Rust:

```rust
// Send a null value for missing keys.
Resp::Null
```

Your `GET` command uses this:

```rust
Command::Get { key } => {
    // Return the value if it exists, otherwise return Redis null.
    match store.get(&key).await {
        Some(value) => Resp::BulkString(value.into_bytes()),
        None => Resp::Null,
    }
}
```

## Arrays

Arrays start with `*`.

Format:

```text
*<number-of-items>\r\n<item1><item2>...
```

Redis commands from clients are usually arrays of bulk strings.

Example:

```text
*1\r\n$4\r\nPING\r\n
```

This means:

```text
Array with 1 item:
1. bulk string "PING"
```

In Rust:

```rust
Resp::Array(vec![
    Resp::BulkString(b"PING".to_vec()),
])
```

Another example:

```text
*3\r\n$3\r\nSET\r\n$4\r\nname\r\n$5\r\nshady\r\n
```

This means:

```text
Array with 3 items:
1. bulk string "SET"
2. bulk string "name"
3. bulk string "shady"
```

In Rust:

```rust
Resp::Array(vec![
    Resp::BulkString(b"SET".to_vec()),
    Resp::BulkString(b"name".to_vec()),
    Resp::BulkString(b"shady".to_vec()),
])
```

Your command layer then converts that RESP array into:

```rust
Command::Set {
    key: "name".to_string(),
    value: "shady".to_string(),
}
```

## Full Command Examples

### PING

Client sends:

```text
*1\r\n$4\r\nPING\r\n
```

Meaning:

```text
["PING"]
```

Server responds:

```text
+PONG\r\n
```

`redis-cli` displays:

```text
PONG
```

### ECHO

Client sends:

```text
*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n
```

Meaning:

```text
["ECHO", "hello"]
```

Server responds:

```text
$5\r\nhello\r\n
```

`redis-cli` displays:

```text
"hello"
```

### SET

Client sends:

```text
*3\r\n$3\r\nSET\r\n$4\r\nname\r\n$5\r\nshady\r\n
```

Meaning:

```text
["SET", "name", "shady"]
```

Server stores:

```text
name = shady
```

Server responds:

```text
+OK\r\n
```

### GET

Client sends:

```text
*2\r\n$3\r\nGET\r\n$4\r\nname\r\n
```

Meaning:

```text
["GET", "name"]
```

If `name` exists, server responds:

```text
$5\r\nshady\r\n
```

If `name` does not exist, server responds:

```text
$-1\r\n
```

### DEL

Client sends:

```text
*2\r\n$3\r\nDEL\r\n$4\r\nname\r\n
```

Meaning:

```text
["DEL", "name"]
```

If the key was removed, server responds:

```text
:1\r\n
```

If the key did not exist, server responds:

```text
:0\r\n
```

### EXISTS

Client sends:

```text
*2\r\n$6\r\nEXISTS\r\n$4\r\nname\r\n
```

Meaning:

```text
["EXISTS", "name"]
```

If the key exists, server responds:

```text
:1\r\n
```

If the key does not exist, server responds:

```text
:0\r\n
```

### INCR

Client sends:

```text
*2\r\n$4\r\nINCR\r\n$5\r\ncount\r\n
```

Meaning:

```text
["INCR", "count"]
```

If `count` does not exist, Redis treats it as `0`, increments it, stores `1`, and
returns:

```text
:1\r\n
```

If `count` already stores `"1"`, it becomes `"2"` and returns:

```text
:2\r\n
```

If the value is not an integer, server responds:

```text
-ERR value is not an integer or out of range\r\n
```

If incrementing or decrementing would overflow, server responds:

```text
-ERR increment or decrement would overflow\r\n
```

### DECR

Client sends:

```text
*2\r\n$4\r\nDECR\r\n$5\r\ncount\r\n
```

Meaning:

```text
["DECR", "count"]
```

If `count` does not exist, Redis treats it as `0`, decrements it, stores `-1`, and
returns:

```text
:-1\r\n
```

If `count` already stores `"10"`, it becomes `"9"` and returns:

```text
:9\r\n
```

If the value is not an integer, server responds:

```text
-ERR value is not an integer or out of range\r\n
```

### EXPIRE

Client sends:

```text
*3\r\n$6\r\nEXPIRE\r\n$7\r\nsession\r\n$1\r\n3\r\n
```

Meaning:

```text
["EXPIRE", "session", "3"]
```

If the key exists and timeout is set, server responds:

```text
:1\r\n
```

If the key does not exist, server responds:

```text
:0\r\n
```

### TTL

Client sends:

```text
*2\r\n$3\r\nTTL\r\n$7\r\nsession\r\n
```

Meaning:

```text
["TTL", "session"]
```

Possible responses:

```text
:3\r\n   key exists and has about 3 seconds left
:-1\r\n  key exists but has no expiry
:-2\r\n  key does not exist
```

## How Your Server Handles A Request

The flow in your server is:

```text
TCP bytes
-> Resp::parse_frame(...)
-> Command::from_resp(...)
-> Command::execute(...)
-> Resp::encode(...)
-> TCP bytes back to client
```

In code, that is roughly:

```rust
// Append new bytes to this client's persistent read buffer.
read_buffer.extend_from_slice(&temp[..n]);

// Parse all complete frames currently available in the buffer.
loop {
    let frame = Resp::parse_frame(&read_buffer)?;

    match frame {
        ParseFrame::Complete { resp, consumed } => {
            read_buffer.drain(..consumed);

            let response = Command::from_resp(resp).execute(store.clone()).await;
            stream.write_all(&response.encode()).await?;
        }
        ParseFrame::Incomplete => break,
    }
}
```

This matters because TCP can split one command across multiple reads or put
multiple commands into one read.

## Pipelining

Redis pipelining means a client sends multiple commands before waiting for
responses.

Example input:

```text
*1\r\n$4\r\nPING\r\n*1\r\n$4\r\nPING\r\n
```

This is two `PING` commands in the same TCP buffer.

Server response:

```text
+PONG\r\n+PONG\r\n
```

## Why redis-cli Works

`redis-cli` works because it speaks RESP.

When you run:

```bash
redis-cli -p 9000 set name shady
```

It sends:

```text
*3\r\n$3\r\nSET\r\n$4\r\nname\r\n$5\r\nshady\r\n
```

Your Rust server parses that as:

```rust
Command::Set {
    key: "name".to_string(),
    value: "shady".to_string(),
}
```

Then your server responds:

```text
+OK\r\n
```

`redis-cli` understands that response and prints:

```text
OK
```

So the important idea is:

```text
We are not supporting redis-cli directly.
We are supporting RESP.
redis-cli works because it speaks RESP.
```

## Testing Without redis-cli

You can send RESP manually with `nc`.

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

Pipelined PING:

```bash
printf '*1\r\n$4\r\nPING\r\n*1\r\n$4\r\nPING\r\n' | nc 127.0.0.1 9000
```

Manual `nc` tests are useful because they show the exact bytes your server receives.
