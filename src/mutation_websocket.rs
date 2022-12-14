use crate::sequencer::SynchronisationController;
use common::MutationWithSource;
use common::{deserialize, serialize, SequencerMutation};
use flume::{Receiver, Sender};
use futures::stream::SplitStream;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use futures_util::StreamExt;
use log::info;
use std::env;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use url::Url;
use uuid::Uuid;

async fn forward_user_actions(
    user_mutations: Receiver<MutationWithSource>,
    mut ws_write: SplitSink<
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >,
) {
    info!("Starting forwarding user actions loop");
    while let Ok(mutation) = user_mutations.recv_async().await {
        match mutation {
            MutationWithSource::Local(mutation) => {
                info!("From local: {:?}", mutation);
                let serialised = serialize(&mutation);
                ws_write.send(Message::Binary(serialised)).await.unwrap();
            }
            MutationWithSource::Remote(_) => (),
        }
    }
}

pub async fn send_mutations_to_server(
    receiver: Receiver<MutationWithSource>,
    synchronisation_controller: Arc<Mutex<SynchronisationController>>,
) {
    let uuid = Uuid::new_v4();

    let url = match env::var("LIBRETAKT_SERVER") {
        Ok(value) => Url::parse(value.as_str()).unwrap(),
        Err(_) => {
            let mut url = String::from("ws://localhost:8081/");
            url.push_str("3f33ef73-4104-4c84-a826-11336ee24d65");
            Url::parse(url.as_str()).unwrap()
        }
    };

    info!("Connecting to url {url}");

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    info!("WebSocket connected");
    let (ws_write, ws_read) = ws_stream.split();

    let apply_remote_users_actions = {
        ws_read.for_each(|message| async {
            let data = message.unwrap().into_data();
            if let Some(mutation) = deserialize(&data) {
                info!("From remote user: {:?}", mutation);
                synchronisation_controller
                    .lock()
                    .unwrap()
                    .mutate_remote(mutation);
            }
        })
    };

    futures::join!(
        forward_user_actions(receiver, ws_write),
        apply_remote_users_actions
    );
}
