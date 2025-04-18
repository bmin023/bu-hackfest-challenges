use std::sync::{Arc, Mutex};

use axum::{debug_handler, extract::State, response::Html, routing::{get, post}, Form, Router};
use serde::Deserialize;
use totp_rs::{TOTP, Secret, Algorithm};
use anyhow;
use askama::Template;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    totp: TOTP,
    previous_pass: Arc<Mutex<String>>,
    flag: Arc<Mutex<String>>
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Raw("SicEmBearsThatGoodOleBaylorLine".as_bytes().to_vec()).to_bytes().unwrap(),
        Some("Baylor University".to_string()),
        "Baylor University Secure Login".to_string(),
    ).unwrap();
    let state = AppState {
        totp,
        previous_pass: Arc::new(Mutex::new("None".to_string())),
        flag: Arc::new(Mutex::new("None".to_string()))
    };

    let app = Router::new()
        .route("/", get(serve_main_page))
        .route("/submit", post(handle_submit))
        .route("/flag", get(serve_flag))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(state);
        

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn serve_flag(State(state): State<AppState>) -> String {
    let flag = state.flag.lock().unwrap();
    flag.to_string()
}

#[derive(Deserialize)]
struct SubmitForm {
    otp: String,
    flag: String,
}

#[debug_handler]
async fn handle_submit(State(state): State<AppState>, Form(form): Form<SubmitForm>) -> Html<String> {
    let mut flag = state.flag.lock().unwrap();
    let mut previous_pass = state.previous_pass.lock().unwrap();
    let Ok(correct) = state.totp.generate_current() else {
        return Html(Notification { current_flag: flag.to_string(), message: Some("Failed to generate TOTP. Tell competition staff.".to_string()) }.render().unwrap());
    };
    if form.otp != correct {
        return Html(Notification { current_flag: flag.to_string(), message: Some("Bad Password".to_string()) }.render().unwrap());
    }
    if correct == *previous_pass {
        return Html(Notification { current_flag: flag.to_string(), message: Some("The flag has already been set this cycle. Try again next cycle.".to_string()) }.render().unwrap());
    }
    *previous_pass = correct.clone();
    *flag = form.flag.clone();
    Html(Notification { current_flag: flag.to_string(), message: Some("Flag set!".to_string()) }.render().unwrap())
}

#[debug_handler]
async fn serve_main_page(State(state): State<AppState>) -> Html<String> {
    let flag = state.flag.lock().unwrap();
    Html(MainPage { current_flag: flag.to_string() }.render().unwrap())
}

#[derive(Template)]
#[template(path = "main.html")]
struct MainPage {
    current_flag: String
}

#[derive(Template)]
#[template(path = "notification.html")]
struct Notification {
    current_flag: String,
    message: Option<String>
}
