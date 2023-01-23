use flume::Sender;
use log::{error, info};

pub async fn get_predictions(results_pipe: Sender<String>) {
    info!("Predictions loop started");

    loop {
        tokio::time::sleep(core::time::Duration::new(5, 0)).await;
        info!("Making a request to predict api");
        let client = reqwest::Client::new();
        let res = client
            .get("http://70.34.252.191:8000/prediction")
            .send()
            .await;
        info!("Request made, processing...");

        match res {
            Ok(data) => match data.text().await {
                Ok(text) => {
                    results_pipe.try_send(text);
                }
                Err(error) => error!("{}", error),
            },
            Err(error) => error!("{}", error),
        };
    }
}
