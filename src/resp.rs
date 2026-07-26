#[derive(Debug, Clone, PartialEq)]
pub enum Resp {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<Resp>),
    Null,
}

impl Resp {
    pub fn encode(&self) -> Vec<u8> {
        match self {
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
}
