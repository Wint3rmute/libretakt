use common::{deserialize, serialize, SequencerMutation};
use url::Url;
use uuid::Uuid;

use std::env;

use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let uuid = Uuid::new_v4();
    let mut url = String::from("ws://localhost:8080/");
    url.push_str(uuid.to_string().as_str());
    let url = Url::parse(url.as_str()).unwrap();

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();

            // let msg = socket.read_message().expect("Error reading message");
            // let mutation_raw = msg.into_data();
            // let mutation = deserialize(mutation_raw.as_ref());

            // println!("Received: {:?}", mutation);
            // socket.close(None);

            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);

        let message = SequencerMutation::UpdateTrackParam(0, 0, 2);
        let serialised = serialize(message);

        tx.unbounded_send(Message::Binary(serialised)).unwrap();

        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
