use awc::Client;

#[actix_web::main]
async fn main() {

    let client = Client::new();

    let res = client
         .get("ws://127.0.0.1:8081")    // <- Create request builder
         .insert_header(("User-Agent", "Actix-web"))
         .send()                             // <- Send http request
         .await;

    println!("Response: {:?}", res); 
}