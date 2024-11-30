use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use custom_protocol_demo::{Protocol, Request, Response};

fn main() -> std::io::Result<()> {
    let client_req = Request::Connect;

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 42069);
    let mut client = Protocol::connect(addr)?;

    client.send_message(&client_req)?;
    client
        .read_message::<Response>()
        .map(|resp| match resp {
            Response::Connected(my_id) => println!("Id in server: {}", my_id),
            _ => panic!("Connection unsuccessfull"),
        })
        .and_then(|_| {
            let message_req = Request::Message("Hola Mundo".into());
            client.send_message(&message_req)
        })
        .and_then(|_| client.send_message(&Request::Disconnect))?;

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
