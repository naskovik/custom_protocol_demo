use custom_protocol_demo::{Protocol, Request, Response};
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

struct Args {
    addr: SocketAddr,
}

fn handle_connection(stream: TcpStream, clients: &mut HashSet<SocketAddr>) -> std::io::Result<()> {
    let peer_addr = stream.peer_addr().expect("Stream has peer_addr");
    let is_new = (*clients).insert(peer_addr);
    let mut protocol = Protocol::with_stream(stream)?;

    let initial_request = protocol.read_message::<Request>()?;
    match initial_request {
        Request::Connect => {
            let res = Response::Ack;
            protocol.send_message(&res)?;
        }
        _ => {
            let res = Response::Error;
            protocol.send_message(&res)?;
        }
    };

    loop {
        let request = protocol.read_message::<Request>()?;
        match request {
            Request::Disconnect => {
                protocol.send_message(&Response::Ack)?;
                clients.remove(&peer_addr);
                break;
            }
            Request::Join(room_id) => {
                let res = if is_new {
                    Response::Joined(room_id)
                } else {
                    Response::JoinReject
                };

                protocol.send_message(&res)?;
            }
            Request::Message { message, .. } => {
                println!("Message received: {}", message);
                //let res = Response::MsgSent;
                //protocol.send_message(&res)?;
            }
            _ => {
                let res = Response::Error;
                protocol.send_message(&res)?;
            }
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Args {
        addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 42069),
    };

    let server_uuid = Uuid::new_v4();
    println!("Server id: {}", server_uuid);

    let mut _clients: Arc<Mutex<HashSet<SocketAddr>>> = Arc::new(Mutex::new(HashSet::new()));

    eprintln!("Starting server on {}", args.addr);
    let listener = TcpListener::bind(args.addr)?;
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let clients = Arc::clone(&_clients);
            std::thread::spawn(move || {
                let mut clients = clients.lock().unwrap();
                handle_connection(stream, &mut clients)
                    .map_err(|err| eprintln!("Error {}", err))
                    .unwrap(); // TODO actually handle error
            });
        }
    }
    Ok(())
}
