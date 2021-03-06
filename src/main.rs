mod node;
mod server;
extern crate json;

use actix::*;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::collections::{HashMap};


use server::*;

fn main() -> std::io::Result<()> {
    env_logger::init();
    let sys = System::new("ws-server");

    // Start chat server actor
    let ws_server = Server {
        name: "Server".to_string(),
        clients: HashMap::new(),
    };
    let server = ws_server.start();

    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .service(web::resource("/").to(|| {
                HttpResponse::Found()
                    .header("LOCATION", "/static/index.html")
                    .finish()
            }))
            .service(fs::Files::new("/static/", "client/"))
            .service(web::resource("/ws/{name}").to(index))
            .service(web::resource("/ws/").to(index))
    })
    .bind("127.0.0.1:3000")?
    .start();

    sys.run()


}
