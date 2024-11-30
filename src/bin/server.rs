use custom_protocol_demo::{Protocol, Request, Response};
use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use uuid::Uuid;

fn handle_connection(stream: TcpStream, clients: &mut HashSet<Uuid>) -> std::io::Result<()> {
    let mut protocol = Protocol::with_stream(stream)?;
    let client_id = Uuid::new_v4();

    let initial_request = protocol.read_message::<Request>()?;
    match initial_request {
        Request::Connect => {
            let _ = (*clients).insert(client_id);
            let res = Response::Connected(client_id.as_u128());
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
                clients.remove(&client_id);
                break;
            }
            Request::Message(message) => {
                println!("Message received: {}", message);
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
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 42069);

    let server_uuid = Uuid::new_v4();
    println!("Server id: {}", server_uuid);

    let mut _clients: Arc<Mutex<HashSet<Uuid>>> = Arc::new(Mutex::new(HashSet::new()));

    eprintln!("Starting server on {}", addr);
    let listener = TcpListener::bind(addr)?;
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
