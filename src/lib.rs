#![allow(dead_code)]
use byteorder::*;
/// Pretending that I'm making some sort of chat application.
/// This would be a custom protocol for that.
/// Probably not useful in a real scenario.
///
///
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
};

pub trait Serialize {
    fn serialize(&self, buf: &mut impl Write) -> io::Result<()>;
}

pub trait Deserialize {
    type Output;
    fn deserialize(buf: &mut impl Read) -> io::Result<Self::Output>;
}

pub enum Request {
    Join(u128),
    Message { room_id: u128, message: String },
}

impl Request {
    pub fn get_message(&self) -> Option<&String> {
        match self {
            Request::Message { message, .. } => Some(message),
            _ => None,
        }
    }

    pub fn get_room(&self) -> u128 {
        match self {
            Request::Message { room_id, .. } => *room_id,
            Request::Join(room_id) => *room_id,
        }
    }
}

impl From<&Request> for u8 {
    fn from(value: &Request) -> Self {
        match value {
            Request::Join(_) => 0,
            Request::Message { .. } => 1,
        }
    }
}

impl Serialize for Request {
    fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        buf.write_u8(self.into())?;
        match self {
            Request::Join(room_id) => {
                buf.write_u8(128)?;
                buf.write_u128::<NetworkEndian>(*room_id)?;
            }
            Request::Message { room_id, message } => {
                buf.write_u8(128)?;
                buf.write_u128::<NetworkEndian>(*room_id)?;
                let message = message.as_bytes();
                buf.write_u16::<NetworkEndian>(message.len() as u16)?;
                buf.write_all(message)?;
            }
        }
        Ok(())
    }
}

impl Deserialize for Request {
    type Output = Request;
    fn deserialize(mut buf: &mut impl Read) -> io::Result<Self::Output> {
        let req_type = buf.read_u8()?;
        match req_type {
            0 => {
                buf.read_u8()?;
                let room_id = buf.read_u128::<NetworkEndian>()?;
                Ok(Request::Join(room_id))
            }
            1 => {
                buf.read_u8()?;
                let room_id = buf.read_u128::<NetworkEndian>()?;
                let message = extract_string(&mut buf)?;

                Ok(Request::Message { room_id, message })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request type",
            )),
        }
    }
}

pub enum Response {
    Joined(u128),
    JoinReject,
    MsgSent(u128),
    Error,
}

impl Response {
    pub fn get_inner(&self) -> Option<u128> {
        match self {
            Response::Joined(room_id) | Response::MsgSent(room_id) => Some(*room_id),
            _ => None,
        }
    }
}

impl From<&Response> for u8 {
    fn from(value: &Response) -> Self {
        match value {
            Response::Error => 0,
            Response::JoinReject => 1,
            Response::Joined(_) => 2,
            Response::MsgSent(_) => 3,
        }
    }
}

impl Serialize for Response {
    fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        buf.write_u8(self.into())?;
        match self {
            Response::Joined(room_id) => {
                buf.write_u16::<NetworkEndian>(128)?;
                buf.write_u128::<NetworkEndian>(*room_id)?;
            }
            Response::MsgSent(msg_id) => {
                buf.write_u16::<NetworkEndian>(128)?;
                buf.write_u128::<NetworkEndian>(*msg_id)?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl Deserialize for Response {
    type Output = Response;
    fn deserialize(buf: &mut impl Read) -> io::Result<Self::Output> {
        let response_type = buf.read_u8()?;
        match response_type {
            0 => Ok(Response::Error),
            1 => Ok(Response::JoinReject),
            2 => {
                let _room_id_len = buf.read_u16::<NetworkEndian>()?; // 128
                let room_id = buf.read_u128::<NetworkEndian>()?;
                Ok(Response::Joined(room_id))
            }
            3 => {
                let _msg_id_len = buf.read_u16::<NetworkEndian>()?; // 128
                let msg_id = buf.read_u128::<NetworkEndian>()?;
                Ok(Response::MsgSent(msg_id))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Unsupported response type byte found at Response::deserialize",
            )),
        }
    }
}

fn extract_string(buf: &mut impl Read) -> io::Result<String> {
    let len = buf.read_u16::<NetworkEndian>()?;
    let mut bytes = vec![0u8; len as usize];
    buf.read_exact(&mut bytes)?;

    String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Utf8"))
}

pub struct Protocol {
    reader: io::BufReader<TcpStream>,
    stream: TcpStream,
}

impl Protocol {
    pub fn with_stream(stream: TcpStream) -> io::Result<Self> {
        let reader = io::BufReader::new(stream.try_clone()?);
        Ok(Self { reader, stream })
    }

    pub fn connect(dest: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(dest)?;
        Self::with_stream(stream)
    }

    pub fn send_message(&mut self, message: &mut impl Serialize) -> io::Result<()> {
        message.serialize(&mut self.stream)?;
        self.stream.flush()
    }

    pub fn read_message<T: Deserialize>(&mut self) -> io::Result<T::Output> {
        T::deserialize(&mut self.reader)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_request_join() {
        let req = Request::Join(10 as u128);
        let mut bytes: Vec<u8> = vec![];
        req.serialize(&mut bytes).unwrap();

        let mut reader = Cursor::new(bytes);
        let join_req = Request::deserialize(&mut reader).unwrap();

        assert!(matches!(join_req, Request::Join(_)));
        assert_eq!(10, join_req.get_room());
    }

    #[test]
    fn test_request_message() {
        let req = Request::Message {
            room_id: 10 as u128,
            message: "Hola".to_string(),
        };
        let mut bytes: Vec<u8> = vec![];
        req.serialize(&mut bytes).unwrap();

        let mut reader = Cursor::new(bytes);
        let message_req = Request::deserialize(&mut reader).unwrap();

        assert!(matches!(message_req, Request::Message { .. }));
        assert_eq!(10 as u128, message_req.get_room());
        assert_eq!("Hola", message_req.get_message().unwrap());
    }

    #[test]
    fn test_response_joined() {
        let res = Response::Joined(10);
        let mut bytes: Vec<u8> = vec![];

        res.serialize(&mut bytes).unwrap();

        let mut reader = Cursor::new(bytes);
        let joined_res = Response::deserialize(&mut reader).unwrap();

        assert!(matches!(joined_res, Response::Joined(_)));
        assert_eq!(joined_res.get_inner().unwrap(), 10);
    }

    #[test]
    fn test_response_msg_sent() {
        let res = Response::MsgSent(10);
        let mut bytes: Vec<u8> = vec![];

        res.serialize(&mut bytes).unwrap();

        let mut reader = Cursor::new(bytes);
        let msg_res = Response::deserialize(&mut reader).unwrap();

        assert!(matches!(msg_res, Response::MsgSent(_)));
        assert_eq!(msg_res.get_inner().unwrap(), 10);
    }

    #[test]
    fn test_response_rest() {
        let res_err = Response::Error;
        let res_jr = Response::JoinReject;

        let mut buff_err: Vec<u8> = vec![];
        let mut buff_jr: Vec<u8> = vec![];

        res_err.serialize(&mut buff_err).unwrap();
        res_jr.serialize(&mut buff_jr).unwrap();

        let mut reader_err = Cursor::new(buff_err);
        let mut reader_jr = Cursor::new(buff_jr);

        let err_res = Response::deserialize(&mut reader_err).unwrap();
        let jr_res = Response::deserialize(&mut reader_jr).unwrap();

        assert!(matches!(err_res, Response::Error));
        assert!(matches!(jr_res, Response::JoinReject));
    }
}
