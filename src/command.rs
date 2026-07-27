use crate::resp::Resp;

//Represent commands that our Redis clone understands. 
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    //Ping Return PONG
    Ping, 

    // ECHO returns the same message back.
    Echo(String),

     // Unknown command name or invalid arguments.
    Unknown(String),
}

impl Command {
     
    // Convert a Resp array into a Command. 

    pub fn from_resp(resp: Resp) -> Command {
        let items = match resp {
            Resp::Array(items) => items,
            _ => return Command::Unknown("Expected command array".to_string()),
        };

        if items.is_empty() {
            return Command::Unknown("empty command".to_string());
        }

        let command_name = match bulk_string_to_string(&items[0]) {
            Some(name) => name.to_ascii_uppercase(), 
            None => return Command::Unknown("command must be a bulk string".to_string()),
        };

        match command_name.as_str() {
            "PING" => Command::Ping,

            "ECHO" => {
                if items.len() != 2 {
                    return Command::Unknown("ECHO expects one argument".to_string());
                }

                match bulk_string_to_string(&items[1]) {
                    Some(message) => Command::Echo(message),
                    None => Command::Unknown("ECHO argument must be a bulk string".to_string()),
                }
            }

            other => Command::Unknown(format!("unknown command: {other}")),
        }
    }

    pub fn execute(self) -> Resp {
        match self {
            Command::Ping => Resp::SimpleString("PONG".to_string()),

            Command::Echo(message) => Resp::BulkString(message.into_bytes()),

            Command::Unknown(message) => Resp::Error(format!("ERR {message}")),
        }
    }
}

fn bulk_string_to_string(resp: &Resp) -> Option<String> {
    match resp {
        Resp::BulkString(bytes) => String::from_utf8(bytes.clone()).ok(),
        _ => None,
    }
}