use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use custom_protocol_demo::{Protocol, Request, Response};
use uuid::Uuid;

pub struct Args {
    id: Uuid,
    room: Option<Uuid>,
    message: Option<String>,
}

fn main() -> std::io::Result<()> {
    let args: Args = Args {
        id: Uuid::new_v4(),
        room: None, // hard-coded for now
        message: Some("Hello World".to_string()),
    };

    let client_req = if let Some(room) = args.room {
        Request::Message {
            room_id: room.as_u128(),
            message: args.message.unwrap_or("EMPTY".to_string()),
        }
    } else {
        Request::Join(Uuid::new_v4().as_u128()) // random, most probably non-existing room.
                                                // Temporal
    };

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 42069);
    Protocol::connect(addr)
        .and_then(|mut client| {
            client.send_message(&client_req)?;
            Ok(client)
        })
        .and_then(|mut client| client.read_message::<Response>())
        .map(|resp| match resp {
            Response::Joined(room_id) => println!("{}", room_id),
            Response::MsgSent(msg_id) => println!("{}", msg_id),
            _ => println!("TODO: Handle JoinReject and Error cases"),
        })
}
