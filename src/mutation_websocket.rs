use common::{deserialize, serialize, SequencerMutation};
use flume::{Receiver, Sender};
use futures_util::stream::SplitSink;
use futures_util::StreamExt;
use log::info;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use url::Url;
use uuid::Uuid;

async fn forward_user_actions(
    user_mutations: Receiver<SequencerMutation>,
    ws_write: SplitSink<
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >,
) {
    while let Ok(mutation) = user_mutations.recv_async().await {
        info!("Received: {:?}", mutation);
        let serialised = serialize(mutation);
    }
}

pub async fn send_mutations_to_server(receiver: Receiver<SequencerMutation>) {
    let uuid = Uuid::new_v4();
    let mut url = String::from("ws://localhost:8080/");
    url.push_str(uuid.to_string().as_str());
    let url = Url::parse(url.as_str()).unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (ws_write, ws_read) = ws_stream.split();

    tokio::spawn(forward_user_actions(receiver, ws_write));
}
