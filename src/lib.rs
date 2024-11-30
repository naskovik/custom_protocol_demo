#![allow(dead_code)]
/// Custom protocol for chat server
///
use byteorder::*;
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
    Connect,
    Message(String),
    Disconnect,
}

impl Request {
    pub fn get_message(&self) -> Option<&String> {
        match self {
            Request::Message(message) => Some(message),
            _ => None,
        }
    }
}

impl From<&Request> for u8 {
    fn from(value: &Request) -> Self {
        match value {
            Request::Message(_) => 0,
            Request::Connect => 1,
            Request::Disconnect => 2,
        }
    }
}

impl Serialize for Request {
    fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        buf.write_u8(self.into())?;
        match self {
            Request::Message(message) => {
                let message = message.as_bytes();
                buf.write_u16::<NetworkEndian>(message.len() as u16)?;
                buf.write_all(message)?;
            }
            _ => {}
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
                let message = extract_string(&mut buf)?;
                Ok(Request::Message(message))
            }
            1 => Ok(Request::Connect),
            2 => Ok(Request::Disconnect),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Request type",
            )),
        }
    }
}

pub enum Response {
    Connected(u128),
    Error,
    Ack,
}

impl Response {
    pub fn get_inner(&self) -> Option<u128> {
        match self {
            Response::Connected(client_id) => Some(*client_id),
            _ => None,
        }
    }
}

impl From<&Response> for u8 {
    fn from(value: &Response) -> Self {
        match value {
            Response::Error => 0,
            Response::Ack => 1,
            Response::Connected(_) => 2,
        }
    }
}

impl Serialize for Response {
    fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        buf.write_u8(self.into())?;
        match self {
            Response::Connected(inner) => {
                buf.write_u16::<NetworkEndian>(128)?;
                buf.write_all(&inner.to_be_bytes())?;
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
            1 => Ok(Response::Ack),
            2 => {
                let _client_id_len = buf.read_u16::<NetworkEndian>()?; // 128
                let client_id = buf.read_u128::<NetworkEndian>()?;
                Ok(Response::Connected(client_id))
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

    pub fn send_message(&mut self, message: &impl Serialize) -> io::Result<()> {
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
        let req = Request::Connect;
        let mut bytes: Vec<u8> = vec![];
        req.serialize(&mut bytes).unwrap();

        let mut reader = Cursor::new(bytes);
        let join_req = Request::deserialize(&mut reader).unwrap();

        assert!(matches!(join_req, Request::Connect));
    }

    #[test]
    fn test_request_message() {
        let req = Request::Message("Hola Mundo".into());
        let mut bytes: Vec<u8> = vec![];
        req.serialize(&mut bytes).unwrap();

        let mut reader = Cursor::new(bytes);
        let message_req = Request::deserialize(&mut reader).unwrap();

        assert!(matches!(message_req, Request::Message { .. }));
        assert_eq!("Hola Mundo", message_req.get_message().unwrap());
    }

    #[test]
    fn test_response_connected() {
        let res = Response::Connected(10);
        let mut bytes: Vec<u8> = vec![];

        res.serialize(&mut bytes).unwrap();

        let mut reader = Cursor::new(bytes);
        let joined_res = Response::deserialize(&mut reader).unwrap();

        assert!(matches!(joined_res, Response::Connected(_)));
        assert_eq!(joined_res.get_inner().unwrap(), 10);
    }

    #[test]
    fn test_response_error() {
        let res_err = Response::Error;
        let mut buff_err: Vec<u8> = vec![];

        res_err.serialize(&mut buff_err).unwrap();

        let mut reader_err = Cursor::new(buff_err);
        let err_res = Response::deserialize(&mut reader_err).unwrap();

        assert!(matches!(err_res, Response::Error));
    }
}
