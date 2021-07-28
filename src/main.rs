use actix_web::{App, HttpServer};
use log::*;
use redis::aio::ConnectionManager;
use redis::RedisResult;

#[cfg(test)]
extern crate fake;

pub mod actors;
pub mod errors;
pub mod handlers;
pub mod utils;

const DEFAULT_PORT: &str = "8080";

async fn try_establish_redis_conn() -> RedisResult<ConnectionManager> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    redis::aio::ConnectionManager::new(redis::Client::open(redis_url)?).await
}

fn setup_env_logger() {
    let env = env_logger::Env::default()
        .filter_or("CONVEY-LOG-LEVEL", "info")
        .write_style_or("CONVEY-LOG-LEVEL", "always");

    env_logger::init_from_env(env);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_env_logger();
    let server_url = std::env::var("CONVEY-URL").unwrap_or_else(|_| "127.0.0.1".to_string());
    info!("Starting server.");
    match try_establish_redis_conn().await {
        Ok(conn) => {
            HttpServer::new(move || {
                App::new().configure(|cfg| {
                    handlers::config_handlers(cfg);
                    actors::config_actors(cfg, conn.clone());
                })
            })
            .bind(format!("{}:{}", server_url, DEFAULT_PORT))?
            .run()
            .await
        }
        Err(err) => {
            log::error!("Cannot establish connection over Redis => {}", err);
            Ok(())
        }
    }
}
