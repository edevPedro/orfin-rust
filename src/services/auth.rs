use axum::{Extension, Json, response::IntoResponse};
use dotenvy::dotenv;
use reqwest::Client;
use serde_json::{Value, json};
use std::env;
use std::sync::Arc;

pub async fn get_key() -> impl IntoResponse {
    println!("Requisição recebida");
    dotenv().ok();
    let id_key = env::var("PLUGGY_ID").expect("Pluggy Id not defined.");
    let secret_key = env::var("PLUGGY_SECRET").expect("Pluggy SECRET not defined.");
    let client = Client::new();
    let payload = json!({"clientId": key.id_key, "clientSecret": key.secret_key});
    let req = client
        .post("https://api.pluggy.ai/auth")
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("Error sending the request");

    let res: Value = req.json().await.expect("Failure parsing json");

    println!("{:#}", res);
    let jsonres = json!({"Response": res});
    Json(jsonres)
}

