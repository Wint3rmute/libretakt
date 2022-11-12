mod lobby;
mod ws;
use lobby::Lobby;
mod messages;
mod start_connection;
use actix::Actor;
use start_connection::start_connection as start_connection_route;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let chat_server = Lobby::default().start(); //create and spin up a lobby
    println!("Beginning");
    HttpServer::new(move || {
        App::new()
            .service(start_connection_route) //register our route. rename with "as" import or naming conflict
            .data(chat_server.clone()) //register the lobby
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
