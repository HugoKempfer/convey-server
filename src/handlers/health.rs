use actix_web::{get, Responder};

#[get("/health")]
pub async fn get_health_status() -> impl Responder {
    "OK"
}
