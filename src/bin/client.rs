use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use custom_protocol_demo::{Protocol, Request, Response};
use uuid::Uuid;

fn main() -> std::io::Result<()> {
    //temporal random room-id
    let room_id = Uuid::new_v4().as_u128();
    let client_req = Request::Join(room_id);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 42069);
    let mut client = Protocol::connect(addr)?;

    client.send_message(&client_req)?;
    client.read_message::<Response>().map(|resp| match resp {
        Response::Joined(room_id) => println!("Joined room {}", room_id),
        _ => panic!("Join unsuccessfull"),
    })?;

    client
        .send_message(&Request::Message {
            room_id,
            message: "Hola mundo".to_string(),
        })
        .and_then(|_| client.read_message::<Response>())?;

    /*
    loop {
        let mut response: Response = Response::JoinReject;
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                response = client
                    .send_message(&Request::Message {
                        room_id,
                        message: input,
                    })
                    .and_then(|_| client.read_message::<Response>())?;
            }
            Err(_) => println!("Write something before sending!"),
        }

        match response {
            Response::Error => break,
            _ => {}
        }
    }
    */
    Ok(())
}
