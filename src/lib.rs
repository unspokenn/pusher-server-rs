#![crate_type = "lib"]

mod app;
mod routes;
mod handlers;

use std::net::SocketAddr;
use crate::app::{Pusher, PusherServer};

const APPLICATION_NAME: &'static str = env!("CARGO_PKG_NAME");

pub async fn start(app_id: &str, app_key: &str, app_secret: &str, bind_address: &str) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let server: PusherServer = PusherServer::new(Pusher::new(app_id.parse::<u32>()?, app_key, app_secret));
    let bind_address: SocketAddr = bind_address.parse().expect("BIND_ADDRESS is invalid");
    let routes = routes::routes(server, &APPLICATION_NAME);

    eprintln!("starting websocket server...");
    eprintln!("bind address: {}", &bind_address);

    warp::serve(routes).run(bind_address).await;

    Ok(())
}


