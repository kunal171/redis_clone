#[derive(Debug, Clone, PartialEq)]
pub enum Resp {
    // Example: +OK\r\n
    SimpleString(String),

    // Example: -ERR unknown command\r\n
    Error(String),

    // Example: :123\r\n
    Integer(i64),

    // Example: $5\r\nhello\r\n
    BulkString(Vec<u8>),

    // Example: *2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n
    Array(Vec<Resp>),

    // Example: $-1\r\n
    Null,
}

impl Resp {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            // Converts a RESP value into bytes that Redis clients understand.
            Resp::SimpleString(s) => format!("+{s}\r\n").into_bytes(),
            Resp::Error(s) => format!("-{s}\r\n").into_bytes(),
            Resp::Integer(n) => format!(":{n}\r\n").into_bytes(),
            Resp::BulkString(bytes) => {
                let mut out = format!("${}\r\n", bytes.len()).into_bytes();
                out.extend_from_slice(bytes);
                out.extend_from_slice(b"\r\n");
                out
            }
            Resp::Array(items) => {
                let mut out = format!("*{}\r\n", items.len()).into_bytes();
                for item in items {
                    out.extend_from_slice(&item.encode());
                }
                out
            }
            Resp::Null => b"$-1\r\n".to_vec(),
        }
    }

    // Parses raw TCP bytes into a RESP value.
    // For now, this parser supports arrays of bulk strings, which is enough for
    // Redis commands like PING, SET, and GET.
    pub fn parse(input: &[u8]) -> Result<Resp, String> {
        // Keep track of where we are while reading the byte slice.
        let mut pos = 0;

        // Parse one RESP value starting at position 0.
        parse_value(input, &mut pos)
    }
}

fn parse_value(input: &[u8], pos: &mut usize) -> Result<Resp, String> {
    // Make sure there is at least one byte to inspect.
    if *pos >= input.len() {
        return Err("unexpected end of input".to_string());
    }

    match input[*pos] {
        // '*' means RESP array.
        b'*' => parse_array(input, pos),

        // '$' means RESP bulk string.
        b'$' => parse_bulk_string(input, pos),

        // Anything else is unsupported for now.
        other => Err(format!("unsupported RESP type byte: {}", other as char)),
    }
}

// Parses an array like: *1\r\n$4\r\nPING\r\n

fn parse_array(input: &[u8], pos: &mut usize) -> Result<Resp, String> {
    // Skip the '*' byte.
    *pos += 1;

    //read the array len
    let len = read_number(input, pos)?;

    if len < 0 {
        return Err("negative array length is not supported".to_string());
    }
    
    let mut items = Vec::new();

    // Parse each item inside the array.
    for _ in 0..len {
        let item = parse_value(input, pos)?;
        items.push(item);
    }

    Ok(Resp::Array(items))
}

// Parses a bulk string like: $4\r\nPING\r\n
fn parse_bulk_string(input: &[u8], pos: &mut usize) -> Result<Resp, String> {
    // Skip the '$' byte.
    *pos += 1;

    // Read the number after '$', which is the byte length of the string.
    let len = read_number(input, pos)?;

    // Redis uses $-1\r\n for null.
    if len == -1 {
        return Ok(Resp::Null);
    }

    if len < 0 {
        return Err("invalid negative bulk string length".to_string());
    }

    let len = len as usize;

    // Make sure enough bytes exist for the string body plus trailing \r\n.
    if *pos + len + 2 > input.len() {
        return Err("bulk string is incomplete".to_string());
    }

    // Read the body bytes.
    let bytes = input[*pos..*pos + len].to_vec();

    // Move past the body.
    *pos += len;

    // Bulk string must end with \r\n.
    expect_crlf(input, pos)?;

    Ok(Resp::BulkString(bytes))
}

// Reads a signed number followed by \r\n.
// Used for array length and bulk string length.
fn read_number(input: &[u8], pos: &mut usize) -> Result<i64, String> {
    let start = *pos;

    //Move until we find \r\n.
    while *pos +1 < input.len() {
        if input[*pos] == b'\r' && input[*pos + 1] == b'\n' {
            let number_bytes = &input[start..*pos];
            
            //Convert bytes into text. 
            let number_text = std::str::from_utf8(number_bytes)
                .map_err(|_| "number is not valid utf-8".to_string())?;

            //skip the \r\n.
            *pos +=2;

            return number_text
                .parse::<i64>()
                .map_err(|_| format!("invalid number: {number_text}"))
        }
        *pos += 1;
    }

    Err("missing CRLF after number".to_string())
}


// Verifies the next two bytes are \r\n, then advances pos.
fn expect_crlf(input: &[u8], pos: &mut usize) -> Result<(), String> {
    if *pos + 1 >= input.len() {
        return Err("missing CRLF".to_string());
    }

    if input[*pos] != b'\r' || input[*pos + 1] != b'\n' {
        return Err("expected CRLF".to_string());
    }

    *pos += 2;
    Ok(())
}