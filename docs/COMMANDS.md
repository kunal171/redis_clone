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

Deletes one or more keys.

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

Multi-key example:

```bash
redis-cli -p 9000 set a 1
redis-cli -p 9000 set b 2
redis-cli -p 9000 del a b c
```

Expected:

```text
OK
OK
(integer) 2
```

The return value is a count. It does not say which keys were deleted.

## EXISTS

Counts how many keys exist.

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

Multi-key example:

```bash
redis-cli -p 9000 set a 1
redis-cli -p 9000 set b 2
redis-cli -p 9000 exists a b c
```

Expected:

```text
OK
OK
(integer) 2
```

The return value is a count. It does not say which keys exist.

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

If incrementing would overflow an `i64`, the server returns:

```text
(error) ERR increment or decrement would overflow
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

If decrementing would overflow an `i64`, the server returns:

```text
(error) ERR increment or decrement would overflow
```

## EXPIRE

Sets a timeout on a key.

Command:

```bash
redis-cli -p 9000 expire session 3
```

If the key exists and the timeout is set:

```text
(integer) 1
```

If the key does not exist:

```text
(integer) 0
```

After the timeout passes, the key is treated as missing. Expired keys are removed
lazily when they are accessed.

Example:

```bash
redis-cli -p 9000 set session abc
redis-cli -p 9000 expire session 3
sleep 3
redis-cli -p 9000 get session
```

Expected:

```text
OK
(integer) 1
(nil)
```

## TTL

Returns the remaining time-to-live for a key.

Command:

```bash
redis-cli -p 9000 ttl session
```

Possible responses:

```text
(integer) <seconds>  key exists and has an expiry
(integer) -1         key exists but has no expiry
(integer) -2         key does not exist
```

Example:

```bash
redis-cli -p 9000 set session abc
redis-cli -p 9000 ttl session
redis-cli -p 9000 expire session 3
redis-cli -p 9000 ttl session
```

Expected:

```text
OK
(integer) -1
(integer) 1
(integer) 2 or 3
```

## Quick Manual Test

```bash
redis-cli -p 9000 ping
redis-cli -p 9000 set name shady
redis-cli -p 9000 get name
redis-cli -p 9000 exists name
redis-cli -p 9000 incr count
redis-cli -p 9000 decr count
redis-cli -p 9000 expire name 3
redis-cli -p 9000 ttl name
redis-cli -p 9000 del name count missing
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
(integer) 2 or 3
(integer) 2
(nil)
```
