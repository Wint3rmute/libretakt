mod lobby;

mod messages;

mod ws;
use actix::Actor;

use crate::ws::WsConn;
use actix::Addr;
use actix_web::middleware::Logger;
use actix_web::{get, web::Data, web::Path, web::Payload, HttpRequest, HttpResponse};
use actix_web::{App, Error, HttpServer, Responder};
use actix_web_actors::ws::start;
use env_logger;
use env_logger::Env;
use uuid::Uuid;

use lobby::Lobby;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let chat_server = lobby::Lobby::default().start(); //create and spin up a lobby

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(start_connection) //register our route. rename with "as" import or naming conflict
            .service(hello)
            .data(chat_server.clone()) //register the lobby
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/{group_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    Path(group_id): Path<Uuid>,
    srv: Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    println!("gowno");
    let ws = WsConn::new(group_id, srv.get_ref().clone());

    let resp = start(ws, &req, stream)?;
    Ok(resp)
}
