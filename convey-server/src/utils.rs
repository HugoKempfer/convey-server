use crate::errors::ConveyError;

use crate::{actors, handlers};
use actix_web::body::{Body, ResponseBody};
use actix_web::web;
use rand::RngCore;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult};
use serde::Serialize;

pub fn is_magnet_link_valid(magnet: &str) -> bool {
    magnet.len() > 8 && &magnet[0..8] == "magnet:?"
}

pub fn gen_token() -> Result<String, ConveyError> {
    let mut token_bytes: [u8; 32] = [0; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    if let Ok(digest) = orion::hash::digest(&token_bytes) {
        Ok(base64::encode(digest.as_ref()))
    } else {
        Err(ConveyError::InternalError)
    }
}

#[cfg(test)]
pub fn config_test_app(cfg: &mut web::ServiceConfig, conn: ConnectionManager) {
    actors::config_actors(cfg, conn);
    handlers::config_handlers(cfg);
}

#[cfg(test)]
pub async fn redis_eq<P: Serialize>(conn: &mut ConnectionManager, key: &str, value: &P) -> bool {
    if let Ok(payload) = serde_json::to_string(&value) {
        let redis_val: RedisResult<String> = conn.get(key).await;
        if let Ok(val) = redis_val {
            return val == payload;
        }
    }
    false
}

#[cfg(test)]
pub async fn redis_set<V: Serialize>(conn: &mut ConnectionManager, key: &str, value: &V) {
    if let Ok(payload) = serde_json::to_string(&value) {
        let _: RedisResult<()> = conn.set(key, payload).await;
    }
}

pub trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for ResponseBody<Body> {
    fn as_str(&self) -> &str {
        match self {
            ResponseBody::Body(ref b) => match b {
                Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                _ => panic!(),
            },
            ResponseBody::Other(ref b) => match b {
                Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                _ => panic!(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn is_magnet_link_valid_test() {
        assert!(is_magnet_link_valid(
            "magnet:?xt=urn:btih:c12fe1c06bba254a9dc9f519b335aa7c1367a88a"
        ));
        assert!(!is_magnet_link_valid(
            "magnet:xt=urn:btih:c12fe1c06bba254a9dc9f519b335aa7c1367a88a"
        ));
        assert!(!is_magnet_link_valid("magnet:?"));
    }

    #[test]
    fn gen_token_test() {
        let mut token_set: HashSet<String> = std::collections::HashSet::new();

        for _ in 0..100 {
            let token = gen_token();
            assert!(token.is_ok());
            let token_string = token.unwrap();
            assert!(!token_string.is_empty());
            assert!(!token_set.contains(&token_string));
            token_set.insert(token_string);
        }
    }
}
