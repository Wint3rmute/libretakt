use common::{deserialize, serialize, SequencerMutation};
use tungstenite::{connect, Message};
use url::Url;
use uuid::Uuid;

fn main() {
    env_logger::init();

    let uuid = Uuid::new_v4();
    let mut url = String::from("ws://localhost:8080/");
    url.push_str(uuid.to_string().as_str());
    let url = Url::parse(url.as_str()).unwrap();
    let (mut socket, response) = connect(url).expect("Can't connect");
    // connect(Url::parse("ws://echo.websocket.org").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    // println!("Response contains the following headers:");
    // for (ref header, _value) in response.headers() {
    //     println!("* {}", header);
    // }
    let message = SequencerMutation::UpdateTrackParam(0, 0, 2);
    let serialised = serialize(message);

    socket.write_message(Message::Binary(serialised)).unwrap();
    loop {
        let msg = socket.read_message().expect("Error reading message");
        let mutation_raw = msg.into_data();
        let mutation = deserialize(mutation_raw.as_ref());

        println!("Received: {:?}", mutation);
    }
    // socket.close(None);
}
