use custom_protocol_demo::{Protocol, Request, Response};
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};
use uuid::Uuid;

struct Args {
    addr: SocketAddr,
}

fn handle_connection(stream: TcpStream, clients: &mut HashSet<SocketAddr>) -> std::io::Result<()> {
    let peer_addr = stream.peer_addr().expect("Stream has peer_addr");
    (*clients).insert(peer_addr);
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

    let server_uuid = Uuid::new_v4();
    println!("Server id: {}", server_uuid);

    let mut clients: HashSet<SocketAddr> = HashSet::new();

    eprintln!("Starting server on {}", args.addr);
    let listener = TcpListener::bind(args.addr)?;
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let _ =
                handle_connection(stream, &mut clients).map_err(|err| eprintln!("Error {}", err));
            // TODO Investigar HashSet multihilo
        }
    }
    Ok(())
}
