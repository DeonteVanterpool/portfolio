use crate::state::AppState;
use aws_config::BehaviorVersion;
use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use std::future::IntoFuture;

pub mod handlers;
pub mod models;
pub mod services;
pub mod state;
pub mod tests;

struct Environment {
    portfolio_entries_bucket: String,
    templates_bucket: String,
    dev: bool,
}

impl Environment {
    pub fn new() -> Self {
        let portfolio_entries_bucket = std::env::var("PORTFOLIO_ENTRIES_BUCKET")
            .expect("No environment variable set for portfolio entries bucket");
        let templates_bucket = std::env::var("TEMPLATES_BUCKET").expect("No environment variable set for portfolio entries bucket");
        let environment = std::env::var("ENVIRONMENT").unwrap_or("".to_owned());

        Self {
            portfolio_entries_bucket,
            templates_bucket,
            dev: environment == "dev",
        }
    }
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    // Fails if no .env file found
    if dotenv().is_err() {
        println!("no .env file found...")
    }

    let config = aws_config::load_defaults(BehaviorVersion::v2025_01_17()).await;
    let client = aws_sdk_s3::Client::new(&config);

    let env = Environment::new();

    let localhost_server_state =
        AppState::from_s3_buckets(&client, &env.portfolio_entries_bucket, &env.templates_bucket, env.dev).await;
    let public_server_state = localhost_server_state.clone();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(handlers::get_portfolio))
        .route("/portfolio", get(Redirect::permanent("/")))
        // .route("/blog", get(handlers::get_blog))
        // .route("/blog/{id}", get(handlers::get_blog_post))
        // .route("/blog/posts", get(handlers::get_blog_posts))
        .route("/resume", get(handlers::get_resume))
        .with_state(public_server_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000")
        .await
        .unwrap();

    println!(
        "Serving public API: http://{}",
        listener.local_addr().unwrap()
    );

    let public_api = axum::serve(listener, app).into_future();

    // run both APIs concurrently
    let public_handle = tokio::spawn(public_api);

    // Wait for either API to finish
    tokio::select! {
        _ = public_handle => {},
    }

    println!("Shutting down the server...");
}
