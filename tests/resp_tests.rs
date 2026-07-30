use redis_clone::resp::{ParseFrame, Resp};

#[test]
fn parses_ping_array() {
    // redis-cli ping sends this RESP array.
    let input = b"*1\r\n$4\r\nPING\r\n";

    // Parse raw RESP bytes into our Rust enum.
    let parsed = Resp::parse(input);

    assert_eq!(
        parsed,
        Ok(Resp::Array(vec![Resp::BulkString(b"PING".to_vec())]))
    );
}

#[test]
fn parses_set_array() {
    // redis-cli set name shady sends this RESP array.
    let input = b"*3\r\n$3\r\nSET\r\n$4\r\nname\r\n$5\r\nshady\r\n";

    // Parse raw bytes.
    let parsed = Resp::parse(input);

    assert_eq!(
        parsed,
        Ok(Resp::Array(vec![
            Resp::BulkString(b"SET".to_vec()),
            Resp::BulkString(b"name".to_vec()),
            Resp::BulkString(b"shady".to_vec()),
        ]))
    );
}

#[test]
fn parses_null_bulk_string() {
    // Redis null bulk string.
    let input = b"$-1\r\n";

    let parsed = Resp::parse(input);

    assert_eq!(parsed, Ok(Resp::Null));
}

#[test]
fn encodes_simple_string() {
    // PING response.
    let resp = Resp::SimpleString("PONG".to_string());

    assert_eq!(resp.encode(), b"+PONG\r\n".to_vec());
}

#[test]
fn encodes_error() {
    // Redis-style error response.
    let resp = Resp::Error("ERR unknown command".to_string());

    assert_eq!(resp.encode(), b"-ERR unknown command\r\n".to_vec());
}

#[test]
fn encodes_integer() {
    // Integer response used by DEL, EXISTS, INCR, DECR.
    let resp = Resp::Integer(1);

    assert_eq!(resp.encode(), b":1\r\n".to_vec());
}

#[test]
fn encodes_bulk_string() {
    // GET response for a string value.
    let resp = Resp::BulkString(b"shady".to_vec());

    assert_eq!(resp.encode(), b"$5\r\nshady\r\n".to_vec());
}

#[test]
fn encodes_null() {
    // GET response for a missing key.
    let resp = Resp::Null;

    assert_eq!(resp.encode(), b"$-1\r\n".to_vec());
}

#[test]
fn parse_frame_reports_consumed_bytes() {
    // Two PING commands are in the same input buffer.
    let input = b"*1\r\n$4\r\nPING\r\n*1\r\n$4\r\nPING\r\n";

    let parsed = Resp::parse_frame(input);

    assert_eq!(
        parsed,
        Ok(ParseFrame::Complete {
            resp: Resp::Array(vec![Resp::BulkString(b"PING".to_vec())]),
            consumed: 14,
        })
    );
}

#[test]
fn parse_frame_reports_incomplete_input() {
    // This is only the first part of a PING command.
    let input = b"*1\r\n$4\r\nPI";

    let parsed = Resp::parse_frame(input);

    assert_eq!(parsed, Ok(ParseFrame::Incomplete));
}

#[test]
fn parse_rejects_extra_bytes() {
    // Resp::parse expects exactly one complete RESP value.
    let input = b"*1\r\n$4\r\nPING\r\n*1\r\n$4\r\nPING\r\n";

    let parsed = Resp::parse(input);

    assert_eq!(parsed, Err("extra bytes after RESP value".to_string()));
}
