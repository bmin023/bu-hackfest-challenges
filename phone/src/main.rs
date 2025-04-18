use std::{collections::HashMap, sync::{Arc, Mutex}};

use axum::{self, extract::State, response::Html, Json};
use tokio;
use reqwest;

#[tokio::main]
async fn main() {
    let flag = Arc::new(Mutex::new("None".to_string()));

    let app = axum::Router::new().route(
        "/",
        axum::routing::get(|| async { Html(include_str!("index.html")) }),
    ).route(
        "/call",
        axum::routing::post(handle_submit),
    ).route("/flag", axum::routing::get(serve_flag)).with_state(flag);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_flag(
    axum::extract::State(flag): axum::extract::State<Arc<Mutex<String>>>,
) -> String {
    let flag = flag.lock().unwrap();
    flag.to_string()
}

#[derive(serde::Deserialize)]
struct T9 {
    number: String,
}

async fn handle_submit(
    State(flag_state): State<Arc<Mutex<String>>>,
    Json(t9): Json<T9>,
) -> String {
    // Convert T9 to string
    let flag = t9to_string(&t9.number);

    println!("Got request for: {} -> {}", &t9.number, flag);

    // Check if the flag is correct
    match check_flag(flag.clone()).await {
        Ok(_) => {
            let mut old_flag = flag_state.lock().unwrap();
            old_flag.clear();
            old_flag.push_str(&flag);
            format!("Flag has been changed to: {}", flag)
        },
        Err(_) => "Unknown flag".to_string(),
    }
}

fn t9to_string(t9: &String) -> String {
    let map: HashMap<&str, char> = HashMap::from([
        ("2", 'a'), ("22", 'b'), ("222", 'c'),
        ("3", 'd'), ("33", 'e'), ("333", 'f'),
        ("4", 'g'), ("44", 'h'), ("444", 'i'),
        ("5", 'j'), ("55", 'k'), ("555", 'l'),
        ("6", 'm'), ("66", 'n'), ("666", 'o'),
        ("7", 'p'), ("77", 'q'), ("777", 'r'), ("7777", 's'),
        ("8", 't'), ("88", 'u'), ("888", 'v'),
        ("9", 'w'), ("99", 'x'), ("999", 'y'), ("9999", 'z'),
    ]);
    let mut result = String::new();
    for c in t9.split('-') {
        if let Some(&ch) = map.get(c) {
            result.push(ch);
        } else {
            result.push_str(c);
        }
    };
    result.to_uppercase()
}

async fn check_flag(flag: String) -> Result<String,String> {
    // grab list of flags from scoreboard. If scoreboard not available, create dummy list
    let resp = if let Ok(res) = reqwest::get("http://board.hack.fest/flags").await {
        res.text().await.unwrap()
    } else {
        "DOG\nCAT\nDUCK".to_string()
    };

    // parse the response
    let flags: Vec<String> = resp.split('\n')
        .map(|s| s.to_string())
        .collect();

    println!("Flags: {:?}", flags);

    // check if the flag is in the list
    if flags.contains(&flag) {
        Ok("Flag is changed".to_string())
    } else {
        Err("Incorrect flag".to_string())
    }
}
