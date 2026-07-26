use crate::resp::Resp;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn run(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, client_addr) = listener.accept().await?;
        println!("client Connected: {client_addr}");

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream).await {
                eprint!("client Error: {err}");
            }
        });
    }
}

pub async fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0_u8; 1024];

    loop {
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Ok(());
        }

        let input = &buffer[..n];

        println!("Received bytes: {input:?}");

        let response = match Resp::parse(input) {
            Ok(Resp::Array(items)) => handle_array_command(items),
            Ok(_) => Resp::Error("ERR expected command array".to_string()),
            Err(err) => Resp::Error(format!("ERR {err}")),
        };

        stream.write_all(&response.encode()).await?;
    }
}

// Converts a RESP array into a Redis command response.
// For now, we only support PING.
fn handle_array_command(items: Vec<Resp>) -> Resp {
    // Empty arrays are not valid commands.
    if items.is_empty() {
        return Resp::Error("ERR empty command".to_string());
    }

    // Redis commands arrive as arrays of bulk strings.
    let command = match &items[0] {
        Resp::BulkString(bytes) => String::from_utf8_lossy(bytes).to_ascii_uppercase(),
        _ => return Resp::Error("ERR command must be a bulk string".to_string()),
    };

    match command.as_str() {
        // Respond to Redis PING command.
        "PING" => Resp::SimpleString("PONG".to_string()),
        
        // Basic placeholder for Redis HELLO command.
        // Real Redis returns server/protocol metadata, but for now we can return OK.
        "HELLO" => Resp::SimpleString("OK".to_string()),
        _ => Resp::Error("ERR unknown command".to_string()),
    }
}
