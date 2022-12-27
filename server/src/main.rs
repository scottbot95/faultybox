use std::{net::{SocketAddr, IpAddr, Ipv4Addr}, str::FromStr};

use axum::{Router, routing::get, response::IntoResponse};
use clap::Parser;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;


#[derive(Parser, Debug)]
#[clap(name = "server", about = "A simple sever for Gecko")]
struct Opt {
    /// Set the log level
    #[clap(short = 'l', long = "log", default_value="debug")]
    log_level: String,

    /// Set the listen address
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// Set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // Setup logging and RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    
    // enable console logging
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(hello))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        opt.port
    ));

    log::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server")
}


async fn hello() -> impl IntoResponse {
    "Hello from server!"
}