use crate::actors::redis_cache_actor::{CacheMsg, RedisCacheActor};
use actix::Actor;
use actix_web::web::ServiceConfig;
use redis::aio::ConnectionManager;

pub mod redis_cache_actor;

pub fn config_actors(cfg: &mut ServiceConfig, conn: ConnectionManager) {
    let cache_actor = RedisCacheActor::new(conn).start();
    cfg.data(cache_actor.clone());
    cfg.data(cache_actor.recipient::<CacheMsg>());
}
