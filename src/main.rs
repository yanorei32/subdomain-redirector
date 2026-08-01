use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use clap::Parser;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::const_new();

#[derive(Deserialize, Debug)]
struct Rule {
    path: String,
    hostname: String,
    redirect_to: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    rules: Vec<Rule>,
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long, short, env, default_value = "/etc/subdomain-redirector.toml")]
    config: PathBuf,

    #[clap(long, short, env, default_value = "0.0.0.0:3000")]
    listen: SocketAddr,
}

async fn handler(req: Request) -> Response {
    let Some(hostname) = req.headers().get("host").and_then(|v| v.to_str().ok()) else {
        return (StatusCode::BAD_REQUEST,).into_response();
    };

    let path = req.uri().path();

    let entry = CONFIG
        .get()
        .unwrap()
        .rules
        .iter()
        .find(|entry| entry.path == path && entry.hostname == hostname);

    match entry {
        Some(entry) => Redirect::to(&entry.redirect_to).into_response(),
        None => (StatusCode::NOT_FOUND,).into_response(),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = std::fs::read_to_string(&cli.config).unwrap();
    let config = toml::from_str(&config).unwrap();
    CONFIG.set(config).unwrap();

    let app = Router::new().fallback(get(handler));
    let listener = TcpListener::bind(cli.listen).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
