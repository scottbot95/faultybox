mod api;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use axum::{response::IntoResponse, Router};
use std::path::Path;
use clap::Parser;

use tokio::signal;

use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::api::api_router;

#[derive(Parser, Debug)]
#[clap(name = "server", about = "A simple sever for Gecko")]
struct Opt {
    /// Set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// Set the listen address
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// Set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// Set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
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


    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        opt.port,
    ));
    let listener = tokio::net::TcpListener::bind(sock_addr)
        .await
        .unwrap_or_else(|_| panic!("Could not bind to {}", sock_addr));

    log::info!("listening on http://{}", listener.local_addr().unwrap());

    let app = make_router(opt);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Unable to start server")
}

fn make_router(opt: Opt) -> Router {
    let static_dir = Path::new(&opt.static_dir);
    let serve_dir = ServeDir::new(&opt.static_dir);

    Router::new()
        .nest("/api", api_router())
        .nest_service("/assets", serve_dir)
        .fallback_service(ServeFile::new(static_dir.join("index.html")))
        .layer(TraceLayer::new_for_http())
}

async fn hello() -> impl IntoResponse {
    "Hello from server!"
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
