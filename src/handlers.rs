use axum::extract::{Path, Query, State};
use axum::response::Html;
use std::collections::HashMap;
use crate::AppState;
use tera::Tera;

// Our root handler
// This handler responds to `GET /` requests
pub async fn get_root() -> &'static str {
    "Hello, World!"
}

pub async fn get_portfolio(State(state): State<AppState>) -> Html<String> {
    let tera = state.get_templater();
    let mut context = tera::Context::new();
    context.insert("portfolio_entries", &state.get_portfolio_entries());

    return match tera.render("portfolio.tera.html", &context) {
        Ok(rendered) => Html(rendered),
        Err(e) => {
            eprintln!("Error rendering template: {}", e);
            Html("Error rendering portfolio".to_string())
        }
    };
}

pub async fn get_resume(State(state): State<AppState>) -> Html<String> {
    let tera = state.get_templater();
    let context = tera::Context::new();

    return match tera.render("resume.tera.html", &context) {
        Ok(rendered) => Html(rendered),
        Err(e) => {
            eprintln!("Error rendering template: {}", e);
            Html("Error rendering blog posts".to_string())
        }
    };
}

