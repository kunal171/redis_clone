# Supported Commands

This document describes the Redis-style commands currently supported by the server.

All commands are sent by clients as RESP arrays of bulk strings. The easiest way to
test them is with `redis-cli`:

```bash
redis-cli -p 9000 <command>
```

## PING

Checks whether the server is alive.

Command:

```bash
redis-cli -p 9000 ping
```

Response:

```text
PONG
```

RESP response:

```text
+PONG\r\n
```

## ECHO

Returns the provided message.

Command:

```bash
redis-cli -p 9000 echo hello
```

Response:

```text
"hello"
```

RESP response:

```text
$5\r\nhello\r\n
```

## SET

Stores a string value under a key.

Command:

```bash
redis-cli -p 9000 set name shady
```

Response:

```text
OK
```

RESP response:

```text
+OK\r\n
```

Current behavior:

- only supports `SET key value`
- does not yet support options like `EX`, `PX`, `NX`, or `XX`

## GET

Reads a string value by key.

Command:

```bash
redis-cli -p 9000 get name
```

If the key exists:

```text
"shady"
```

If the key does not exist:

```text
(nil)
```

RESP responses:

```text
$5\r\nshady\r\n
$-1\r\n
```

## DEL

Deletes a key.

Command:

```bash
redis-cli -p 9000 del name
```

If the key existed:

```text
(integer) 1
```

If the key did not exist:

```text
(integer) 0
```

RESP responses:

```text
:1\r\n
:0\r\n
```

Current behavior:

- only supports one key
- real Redis supports one or more keys

## EXISTS

Checks whether a key exists.

Command:

```bash
redis-cli -p 9000 exists name
```

If the key exists:

```text
(integer) 1
```

If the key does not exist:

```text
(integer) 0
```

Current behavior:

- only supports one key
- real Redis supports one or more keys and returns the count of existing keys

## INCR

Increments a key as an integer.

Command:

```bash
redis-cli -p 9000 incr count
```

If the key does not exist, it is treated as `0`.

Example:

```bash
redis-cli -p 9000 incr count
redis-cli -p 9000 incr count
redis-cli -p 9000 get count
```

Response:

```text
(integer) 1
(integer) 2
"2"
```

If the value is not an integer:

```bash
redis-cli -p 9000 set name shady
redis-cli -p 9000 incr name
```

Response:

```text
(error) ERR value is not an integer or out of range
```

## DECR

Decrements a key as an integer.

Command:

```bash
redis-cli -p 9000 decr count
```

If the key does not exist, it is treated as `0`.

Example:

```bash
redis-cli -p 9000 decr count
redis-cli -p 9000 decr count
redis-cli -p 9000 get count
```

Response:

```text
(integer) -1
(integer) -2
"-2"
```

If the value is not an integer:

```bash
redis-cli -p 9000 set name shady
redis-cli -p 9000 decr name
```

Response:

```text
(error) ERR value is not an integer or out of range
```

## Quick Manual Test

```bash
redis-cli -p 9000 ping
redis-cli -p 9000 set name shady
redis-cli -p 9000 get name
redis-cli -p 9000 exists name
redis-cli -p 9000 incr count
redis-cli -p 9000 decr count
redis-cli -p 9000 del name
redis-cli -p 9000 get name
```

Expected:

```text
PONG
OK
"shady"
(integer) 1
(integer) 1
(integer) 0
(integer) 1
(nil)
```

