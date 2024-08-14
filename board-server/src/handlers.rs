use std::sync::Arc;

use axum::extract::Path;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use tokio::sync::broadcast;

use crate::mongo::{client::Mongod, users::User};
use crate::{error, AppError};
use axum::extract::Query;
use mongodb::results::InsertOneResult;
use rand::Rng;
use serde::Deserialize;
use tracing_wrapper::tracing::{self, instrument};

use futures::{sink::SinkExt, stream::StreamExt};

use crate::Json;
#[derive(Deserialize, Debug)]

pub struct HelloNameGetName {
    pub name: String,
}

pub struct AppState {
    tx: broadcast::Sender<String>,
}

pub fn routes() -> Router {
    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState { tx });

    Router::new()
        .route("/health-check", get(health_check))
        .route("/api/hello", get(hello_name))
        .route("/api/addData", post(insert_user))
        .route("/api/getData/:id", get(get_user_with_id))
        .route("/rand", post(rand_integer))
        .route("/ws/rand", get(find_all_integers))
        .layer(Extension(app_state))
}

#[utoipa::path(
    get,
    path = "/health-check", 
    responses(
        (status = 200, description = "The service is online", body = String)
    )
)]
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Service is healthy")
}

//Function for Hello <Name>
#[instrument(skip_all)]
pub async fn hello_name(user: Query<HelloNameGetName>) -> String {
    format!("Hello {}!", user.name)
}

pub async fn insert_user(
    Extension(db_con): Extension<Mongod<'_>>,
    Json(payload): Json<User>,
) -> Result<Json<InsertOneResult>, error::AppError> {
    let x = db_con
        .insert_user_in_base(payload.id, payload.name, payload.age)
        .await?;

    Ok(Json(x))
}

pub async fn get_user_with_id(
    Extension(db_con): Extension<Mongod<'_>>,
    Path(id): Path<String>,
) -> Result<Json<User>, error::AppError> {
    let x = db_con.find_use_in_base(id).await?;
    Ok(Json(x))
}

pub async fn rand_integer(
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Json<i32>, AppError> {
    let mut rng = rand::thread_rng();
    let number: i32 = rng.gen();

    //send the message for new number
    let message = format!("The random number is: {}", number);
    let _ = app_state.tx.send(message);
    Ok(Json(number))
}

async fn find_all_integers(
    ws: WebSocketUpgrade,
    Extension(app_state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, app_state))
}

async fn handle_socket(socket: WebSocket, app_state: Arc<AppState>) {
    //sender to write
    //and receiver to read
    tracing::info!("Open WebSocket");
    let (mut sender, mut _receiver) = socket.split();

    let mut rx = app_state.tx.subscribe();

    //write message from app_state in websocket
    let mut _send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            tracing::info!("The number is {} ", msg.clone());
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
}
