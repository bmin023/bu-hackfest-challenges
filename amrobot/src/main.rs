use std::{
    collections::HashMap,
    sync::Arc,
    time::Instant,
};

use askama::Template;
use axum::{
    self,
    extract::State,
    response::Html,
    routing::{get, post},
    Form, Router,
};
use rand::prelude::*;
use serde::Deserialize;
use tokio::sync::Mutex;
use uuid::Uuid;

use anyhow::Result;

#[derive(Clone)]
struct AppState {
    flag: Arc<Mutex<String>>,
    challenges: Arc<Mutex<HashMap<Uuid, (Instant, i64)>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        flag: Arc::new(Mutex::new("None".to_string())),
        challenges: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(|| async { Html(IndexTemplate.render().unwrap()) }))
        .route("/challenge", post(serve_challenge))
        .route("/submit", post(handle_submit))
        .route("/flag", get(serve_flag))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn serve_flag(State(state): State<AppState>) -> Html<String> {
    let flag = state.flag.lock().await;
    Html(flag.to_string())
}

async fn serve_challenge(State(state): State<AppState>) -> Html<String> {
    let challenge = generate_challenge();

    println!("{}", &challenge.2);

    let mut challenges = state.challenges.lock().await;

    challenges.insert(challenge.0.clone(), (Instant::now(), challenge.2));

    // remove all challenges older than 480 seconds
    challenges.retain(|_, (instant, _)| instant.elapsed().as_secs() < 480);

    Html(
        ChallengeTemplate {
            challenge: Some(Challenge {
                uuid: challenge.0,
                question: challenge.1,
            }),
            message: None,
            loading: true,
        }
        .render()
        .unwrap(),
    )
}

#[derive(Deserialize)]
struct SubmitForm {
    uuid: Uuid,
    flag: String,
    answer: i64,
}

async fn handle_submit(
    State(state): State<AppState>,
    Form(form): Form<SubmitForm>,
) -> Html<String> {
    let mut challenges = state.challenges.lock().await;

    if let Some((instant, answer)) = challenges.get(&form.uuid) {
        if *answer == form.answer {
            println!("Time Passed: {}", instant.elapsed().as_secs());
            if instant.elapsed().as_secs() <= 5 {
                let mut flag = state.flag.lock().await;
                *flag = form.flag.clone();
                challenges.remove(&form.uuid);
                Html(
                    ChallengeTemplate {
                        challenge: None,
                        message: Some("Welcome fellow robot! The flag has been changed.".into()),
                        loading: false,
                    }
                    .render()
                    .unwrap(),
                )
            } else {
                Html(
                    ChallengeTemplate {
                        challenge: None,
                        message: Some("Correct! But it took you more than 5 seconds so you are clearly not a robot!".into()),
                        loading: false,
                    }
                    .render()
                    .unwrap(),
                )
            }
        } else {
            Html(
                ChallengeTemplate {
                    challenge: None,
                    message: Some("Incorrect Answer! You are clearly not a robot".into()),
                    loading: false,
                }
                .render()
                .unwrap(),
            )
        }
    } else {
        Html(
            ChallengeTemplate {
                challenge: None,
                message: Some("I don't recognize this challenge. Either it was already solved or given too long ago.".into()),
                loading: false,
            }
            .render()
            .unwrap(),
        )
    }
}

/// Generates a challenge (a math problem) and its answer.
fn generate_challenge() -> (Uuid, String, i64) {
    let mut rng = rand::rng();
    let numbers: (i16, i16, i16, i16) = rng.random();
    let n1 = numbers.0 as i64 / 2;
    let n2 = numbers.1 as i64 / 2;
    let n3 = numbers.2 as i64 / 2;
    let n4 = numbers.3 as i64 / 2;
    let answer = n1 + n2 * n3 - n4;
    let challenge = format!(
        "What is {} + {} * {} - {}?",
        n1, n2, n3, n4
    );
    return (Uuid::new_v4(), challenge, answer);
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

struct Challenge {
    uuid: Uuid,
    question: String,
}

#[derive(Template)]
#[template(path = "challenge.html")]
struct ChallengeTemplate {
    challenge: Option<Challenge>,
    message: Option<String>,
    loading: bool,
}
