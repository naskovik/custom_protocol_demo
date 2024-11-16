use custom_protocol_demo::{Protocol, Request, Response};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use uuid::Uuid;

struct Args {
    addr: SocketAddr,
}

fn handle_connection(stream: TcpStream) -> std::io::Result<()> {
    let peer_addr = stream.peer_addr().expect("Stram has peer_addr");
    eprintln!("Incoming [{}]", peer_addr);
    let mut protocol = Protocol::with_stream(stream)?;

    let request = protocol.read_message::<Request>()?;
    let res = match request {
        Request::Message { .. } => Response::MsgSent(Uuid::new_v4().as_u128()),
        Request::Join(room_id) => Response::Joined(room_id),
    };

    protocol.send_message(&res)
}

fn main() -> std::io::Result<()> {
    let args = Args {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 42069),
    };
    eprintln!("Starting server on {}", args.addr);
    let listener = TcpListener::bind(args.addr)?;
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            std::thread::spawn(move || {
                handle_connection(stream).map_err(|err| eprintln!("Error {}", err))
            });
        }
    }
    Ok(())
}
