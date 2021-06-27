use anyhow::Result;
use redis::Commands;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::Responder;
use rocket::State;
use rocket::{get, launch, post, routes};
use std::future::Future;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use rocket_sync_db_pools::{database, rusqlite};

#[database("keys")]
struct KeysDbConn(rusqlite::Connection);


type SharedRedisClient = Arc<Mutex<redis::Client>>;

#[derive(Responder, Serialize, Deserialize)]
#[response(status = 200, content_type = "json")]
struct VerifyNonce {
    nonce: String,
}

#[derive(Serialize, Deserialize)]
struct Presentation {
    holder: String,
}

#[derive(Responder, Serialize, Deserialize)]
enum VerifyResponse {
    #[response(status = 200, content_type = "json")]
    Success { message: String },
    #[response(status = 400, content_type = "json")]
    InvalidNonce { message: String },
}

#[get("/verify")]
fn get_verify(redis_client: &State<SharedRedisClient>) -> VerifyNonce {
    let mut redis = redis_client.lock().unwrap().get_connection().unwrap();
    let nonce = Uuid::new_v4().to_string();

    let _: () = redis
        .set_ex(nonce.to_owned(), nonce.to_owned(), 60)
        .unwrap();

    VerifyNonce { nonce }
}

#[post("/verify/<nonce>", format = "json", data = "<presentation>")]
fn post_verify(
    redis_client: &State<SharedRedisClient>,
    nonce: String,
    presentation: Json<Presentation>,
) -> VerifyResponse {
    let mut redis = redis_client.lock().unwrap().get_connection().unwrap();

    let value: Option<String> = redis.get(nonce.to_owned()).unwrap();

    if value.is_none() {
        VerifyResponse::InvalidNonce { message: "".to_owned() }
    } else {
        VerifyResponse::Success { message: "".to_owned() }
    }
}

#[launch]
fn rocket() -> _ {
    let redis_client = redis::Client::open("redis://127.0.0.1").unwrap();
    rocket::build()
        .attach(KeysDbConn::fairing())
        .manage(Arc::new(Mutex::new(redis_client)))
        .mount("/", routes![get_verify, post_verify])
}
