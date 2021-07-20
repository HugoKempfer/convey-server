use actix_web::{App, HttpServer};
use log::*;
use redis::aio::ConnectionManager;
use redis::RedisResult;

mod actors;
pub mod errors;
mod handlers;
pub mod utils;

async fn try_establish_redis_conn() -> RedisResult<ConnectionManager> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    redis::aio::ConnectionManager::new(redis::Client::open(redis_url)?).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    info!("Starting server.");
    match try_establish_redis_conn().await {
        Ok(conn) => {
            HttpServer::new(move || {
                App::new().configure(|cfg| {
                    handlers::config_handlers(cfg);
                    actors::config_actors(cfg, conn.clone());
                })
            })
            .bind("127.0.0.1:8080")?
            .run()
            .await
        }
        Err(err) => {
            log::error!("Cannot establish connection over Redis => {}", err);
            Ok(())
        }
    }
}
