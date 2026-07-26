use crate::resp::Resp;

//Represent commands that our Redis clone understands. 
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    //Ping Return PONG
    Ping, 
    Echo(String),
    Unknown(String),
}