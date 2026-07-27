use crate::resp::Resp;
use crate::store::Store;

//Represent commands that our Redis clone understands. 
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    //Ping Return PONG
    Ping, 

    // ECHO returns the same message back.
    Echo(String),

    // Store a key-value pair.
    Set {key: String, value: String},

    // Read a value by key
    Get {key: String},

    // Delete the Key value

    Delete {key: String},

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

            "SET" => {
                // SET needs exactly: SET key value
                if items.len() != 3 {
                    return Command::Unknown("SET expects key and value".to_string());
                }

                let key = match bulk_string_to_string(&items[1]) {
                    Some(key) => key,
                    None => return Command::Unknown("SET key must be a bulk string".to_string()),
                };

                let value = match bulk_string_to_string(&items[2]) {
                    Some(value) => value,
                    None => return Command::Unknown("SET value must be a bulk string".to_string()),
                };

                Command::Set { key, value }
            },

            "GET" => {
                // GET needs exactly: GET key
                if items.len() != 2 {
                    return Command::Unknown("GET expects key".to_string());
                }

                let key = match bulk_string_to_string(&items[1]) {
                    Some(key) => key,
                    None => return Command::Unknown("GET key must be a bulk string".to_string()),
                };

                Command::Get { key }
            }

            "DELETE" => {
                // DELETE needs exactly: DELETE key
                if items.len() != 2 {
                    return Command::Unknown("DELETE expects key".to_string());
                }

                let key = match bulk_string_to_string(&items[1]) {
                    Some(key) => key,
                    None => return Command::Unknown("DELETE key must be a bulk string".to_string()),
                };

                Command::Delete { key }
            },


            other => Command::Unknown(format!("unknown command: {other}")),
        }
    }

    pub async fn execute(self, store: Store) -> Resp {
        match self {
            Command::Ping => Resp::SimpleString("PONG".to_string()),

            Command::Echo(message) => Resp::BulkString(message.into_bytes()),

            Command::Set { key, value } => {
                // Save the key-value pair.
                store.set(key, value).await;

                Resp::SimpleString("OK".to_string())
            }

            Command::Get { key } => {
                // Return the value if it exists.
                match store.get(&key).await {
                    Some(value) => Resp::BulkString(value.into_bytes()),
                    None => Resp::Null,
                }
            }

            Command::Delete { key } => {
                // Return OK if deleted, null if not found.
                match store.del(&key).await {
                    true => Resp::SimpleString("OK".to_string()),
                    false => Resp::Null,
                }
            }

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