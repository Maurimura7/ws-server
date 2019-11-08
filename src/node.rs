use crate::server::{Connect, Disconnect, List, Message, Server};
use actix::*;
use actix_web_actors::ws;
use json::*;

#[derive(Clone, Message)]
#[rtype(String)]
pub struct Connected(pub String);

/// Handler for Disconnect message.
impl Handler<Connected> for Server {
    type Result = String;

    fn handle(&mut self, msg: Connected, _: &mut Context<Self>) -> Self::Result {
        println!("{:?}", msg.0);
        msg.0
    }
}

pub struct Node {
    pub id: usize,
    pub name: String,
    pub addr: Addr<Server>,
}

impl Actor for Node {
    type Context = ws::WebsocketContext<Self>;
    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx2| {
                match res {
                    Ok(res) => {
                        println!("ID Matched: {:?}", res);
                        act.id = res;
                    }
                    // something is wrong with chat server
                    _ => ctx2.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);

        self.addr
            .send(List)
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(clients) => {
                        let mut data = array! [];
                        for client in clients {
                            let _= data.push(client);
                        }
                        ctx.text(json::stringify(data))
                    }
                    _ => println!("Something is wrong"),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        println!("Someone dissconected");
        // notify chat server
        self.addr.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for Node {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WS: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => (),
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => ctx.stop(),
            _ => (),
        }
    }
}

impl Handler<Message> for Node {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
