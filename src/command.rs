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
    Set { key: String, value: String },

    // Read a value by key
    Get { key: String },

    // Delete one or more keys.
    Delete { keys: Vec<String> },

    // Count how many of the given keys exist.
    Exists { keys: Vec<String> },

    //Adding the Key as integer
    Incr { key: String },

    // Decrease the integer counter.
    Decr { key: String },

    // Set a timeout on a key.
    Expire { key: String, seconds: u64 },

    // Return the remaining time-to-live for a key.
    Ttl { key: String },

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
            }

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

            "DEL" => {
                // DEL needs at least one key.
                if items.len() < 2 {
                    return Command::Unknown("DEL expects at least one key".to_string());
                }

                let keys = match bulk_string_to_keys(&items[1..]) {
                    Ok(keys) => keys,
                    Err(err) => return Command::Unknown(format!("DEL {err}")),
                };

                Command::Delete { keys }
            }

            "EXISTS" => {
                // EXISTS needs at least one key.
                if items.len() < 2 {
                    return Command::Unknown("EXISTS expects at least one key".to_string());
                }

                let keys = match bulk_string_to_keys(&items[1..]) {
                    Ok(keys) => keys,
                    Err(err) => return Command::Unknown(format!("EXISTS {err}")),
                };

                Command::Exists { keys }
            }

            "INCR" => {
                // INCR needs exactly: INCR key
                if items.len() != 2 {
                    return Command::Unknown("INCR expects key".to_string());
                }

                let key = match bulk_string_to_string(&items[1]) {
                    Some(key) => key,
                    None => {
                        return Command::Unknown("INCR key must be a bulk string".to_string());
                    }
                };

                Command::Incr { key }
            }

            "DECR" => {
                // DECR needs exactly: DECR key
                if items.len() != 2 {
                    return Command::Unknown("DECR expects key".to_string());
                }

                let key = match bulk_string_to_string(&items[1]) {
                    Some(key) => key,
                    None => {
                        return Command::Unknown("DECR key must be a bulk string".to_string());
                    }
                };

                Command::Decr { key }
            }

            "EXPIRE" => {
                // EXPIRE needs exactly: EXPIRE key seconds
                if items.len() != 3 {
                    return Command::Unknown("EXPIRE expects key and seconds".to_string());
                }

                let key = match bulk_string_to_string(&items[1]) {
                    Some(key) => key,
                    None => {
                        return Command::Unknown("EXPIRE key must be a bulk string".to_string());
                    }
                };

                let seconds_text = match bulk_string_to_string(&items[2]) {
                    Some(seconds) => seconds,
                    None => {
                        return Command::Unknown("EXPIRE seconds must be a bulk string".to_string());
                    }
                };

                let seconds = match seconds_text.parse::<u64>() {
                    Ok(seconds) => seconds,
                    Err(_) => {
                        return Command::Unknown("EXPIRE seconds must be an integer".to_string());
                    }
                };

                Command::Expire { key, seconds }
            }

            "TTL" => {
                // TTL needs exactly: TTL key
                if items.len() != 2 {
                    return Command::Unknown("TTL expects key".to_string());
                }

                let key = match bulk_string_to_string(&items[1]) {
                    Some(key) => key,
                    None => return Command::Unknown("TTL key must be a bulk string".to_string()),
                };

                Command::Ttl { key }
            }

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

            Command::Delete { keys } => {
                // Redis DEL returns how many keys were removed.
                let removed = store.del_many(&keys).await;
                Resp::Integer(removed)
            }

            Command::Exists { keys } => {
                // Redis EXISTS returns how many requested keys exist.
                let count = store.exists_many(&keys).await;
                Resp::Integer(count)
            }

            Command::Incr { key } => match store.incr(&key).await {
                Ok(value) => Resp::Integer(value),
                Err(err) => Resp::Error(format!("ERR {err}")),
            },

            Command::Decr { key } => match store.decr(&key).await {
                Ok(value) => Resp::Integer(value),
                Err(err) => Resp::Error(format!("ERR {err}")),
            },
            Command::Expire { key, seconds } => {
                // Redis EXPIRE returns 1 if timeout was set, 0 if key does not exist.
                if store.expire(&key, seconds).await {
                    Resp::Integer(1)
                } else {
                    Resp::Integer(0)
                }
            }

            Command::Ttl { key } => {
                // Redis TTL returns seconds, -1 for no expiry, or -2 for missing key.
                Resp::Integer(store.ttl(&key).await)
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

// Converts all RESP bulk strings after the command name into keys.
fn bulk_string_to_keys(items: &[Resp]) -> Result<Vec<String>, String> {
    let mut keys = Vec::new();

    for item in items {
        match bulk_string_to_string(item) {
            Some(key) => keys.push(key),
            None => return Err("key must be a bulk string".to_string()),
        }
    }

    Ok(keys)
}
