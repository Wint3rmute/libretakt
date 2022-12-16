use crate::lobby::Lobby;
use crate::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::{fut, ActorContext, ActorFuture, ContextFutureSpawner, WrapFuture};
use actix::{Actor, Addr, Running, StreamHandler};
use actix::{AsyncContext, Handler};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::Text;
use std::time::{Duration, Instant};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConn {
    room: Uuid,
    lobby_addr: Addr<Lobby>,
    hb: Instant,
    id: Uuid,
}

impl WsConn {
    pub fn new(room: Uuid, lobby: Addr<Lobby>) -> WsConn {
        WsConn {
            id: Uuid::new_v4(),
            room,
            hb: Instant::now(),
            lobby_addr: lobby,
        }
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WS conn started");
        self.hb(ctx);

        let addr = ctx.address();
        self.lobby_addr
            .send(Connect {
                addr: addr.recipient(),
                lobby_id: self.room,
                self_id: self.id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.lobby_addr.do_send(Disconnect {
            id: self.id,
            room_id: self.room,
        });
        println!("WS conn stopped");
        Running::Stop
    }
}

impl WsConn {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // println!("Disconnecting failed heartbeat");
                // act.lobby_addr.do_send(Disconnect {
                //     id: act.id,
                //     room_id: act.room,
                // });
                // ctx.stop();
                return;
            }

            ctx.ping(b"hi");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
                print!("Pong sended");
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
                print!("Heartbit reseted");
            }
            Ok(ws::Message::Binary(bin)) => {
                if let Some(mutation) = common::deserialize(bin.to_vec().as_slice()) {
                    println!("{:?}", mutation);

                    self.lobby_addr.do_send(ClientActorMessage {
                        id: self.id,
                        msg: mutation,
                        room_id: self.room,
                    })
                } else {
                    println!("Cannot deserialize mutation");
                }
                // ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
                print!("Close message received");
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
                print!("Stop message received");
            }
            Ok(ws::Message::Nop) => (),
            Ok(Text(s)) => (),
            Err(e) => panic!("{}", e),
        }
    }
}

impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.binary(msg.0.clone());
        if let Some(mutation) = common::deserialize(&msg.0) {
            println!("{:?}", mutation);
        } else {
            println!("Cannot deserialize mutation");
        }
    }
}
