use crate::actors::redis_cache_actor::RedisCacheActor;
use actix::Actor;
use actix_web::web::ServiceConfig;
use redis::aio::ConnectionManager;

pub mod redis_cache_actor;

pub fn config_actors(cfg: &mut ServiceConfig, conn: ConnectionManager) {
    cfg.data(RedisCacheActor::new(conn).start());
}
