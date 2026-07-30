use crate::command::Command;
use crate::resp::{ParseFrame, Resp};
use crate::store::Store;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn run(addr: &str, store: Store) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, client_addr) = listener.accept().await?;
        println!("client Connected: {client_addr}");

        let store = store.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, store).await {
                eprint!("client Error: {err}");
            }
        });
    }
}

pub async fn handle_client(mut stream: TcpStream, store: Store) -> std::io::Result<()> {
    // Temporary read buffer for each socket read.
    let mut temp = [0_u8; 1024];

    // Persistent buffer for this client.
    // It stores bytes across reads until a full RESP command is available.
    let mut read_buffer = Vec::new();

    loop {
        // Read new bytes from the client.
        let n = stream.read(&mut temp).await?;

        // n == 0 means the client closed the connection.
        if n == 0 {
            return Ok(());
        }

        // Append newly-read bytes to the persistent buffer.
        read_buffer.extend_from_slice(&temp[..n]);

        // Try to parse and execute as many complete commands as possible.
        loop {
            let frame = match Resp::parse_frame(&read_buffer) {
                Ok(frame) => frame,
                Err(err) => {
                    // Protocol error: send error and clear buffer.
                    let response = Resp::Error(format!("ERR {err}"));
                    stream.write_all(&response.encode()).await?;
                    read_buffer.clear();
                    break;
                }
            };

            match frame {
                ParseFrame::Complete { resp, consumed } => {
                    // Remove the bytes used by this one command.
                    read_buffer.drain(..consumed);

                    // Convert RESP into Command, execute it, and write response.
                    let response = Command::from_resp(resp).execute(store.clone()).await;
                    stream.write_all(&response.encode()).await?;

                    // Continue loop in case more complete commands are buffered.
                }

                ParseFrame::Incomplete => {
                    // Wait for the next socket read to complete the command.
                    break;
                }
            }
        }
    }
}
