use crate::actors::redis_cache_actor::CacheMsg::{CloseSession, GetSessionInfos, OpenSession};
use crate::actors::redis_cache_actor::{OpenSessionReq, RedisCacheActor, SessionInfos};
use crate::errors::ConveyError;
use actix::Addr;
use actix_web::web::Json;
use actix_web::{delete, get, post, web, HttpResponse};

#[post("")]
pub async fn open_file_sharing_session(
    req: web::Json<OpenSessionReq>,
    cache: web::Data<Addr<RedisCacheActor>>,
) -> Result<HttpResponse, ConveyError> {
    let session = cache.send(OpenSession(req.0)).await??;

    Ok(HttpResponse::Created()
        .content_type("application/json")
        .body(serde_json::to_string(&session)?))
}

#[get("/{session_id}")]
pub async fn get_session_infos(
    id: web::Path<String>,
    cache: web::Data<Addr<RedisCacheActor>>,
) -> Result<Json<SessionInfos>, ConveyError> {
    Ok(Json(cache.send(GetSessionInfos(id.0)).await??))
}

#[delete("/{session_id}")]
pub async fn close_session(
    id: web::Path<String>,
    cache: web::Data<Addr<RedisCacheActor>>,
    token: web::Json<String>,
) -> Result<Json<SessionInfos>, ConveyError> {
    Ok(Json(cache.send(CloseSession { id: id.0, revocation_token: token.0 }).await??))
}
