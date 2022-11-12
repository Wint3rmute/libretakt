use awc::Client;
use uuid::Uuid;

#[actix_web::main]
async fn main() {

    let client = Client::new();
    let url = format!("ws://localhost:8081/{}", Uuid::new_v4());
    println!("{}",url);

    let res = client
         .get(url)    // <- Create request builder
         .insert_header(("User-Agent", "Actix-web"))
         .send()                             // <- Send http request
         .await;

    println!("Response: {:?}", res); 

    
}