use crate::actors::redis_cache_actor::CacheMsg::{CloseSession, GetSessionInfos, OpenSession};
use crate::actors::redis_cache_actor::{CacheMsg, OpenSessionReq, RedisCacheActor, SessionInfos};
use crate::errors::ConveyError;
use actix::{Addr, Recipient};
use actix_web::web::Json;
use actix_web::{delete, get, post, web, HttpResponse};

#[post("")]
pub async fn open_file_sharing_session(
    req: web::Json<OpenSessionReq>,
    cache: web::Data<Recipient<CacheMsg>>,
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

#[cfg(test)]
mod tests {

    use crate::actors::redis_cache_actor::{OpenSessionReq, SessionInfos};
    use crate::try_establish_redis_conn;
    use crate::utils::{config_test_app, redis_eq, redis_set, BodyTest};
    use actix_web::http::StatusCode;
    use actix_web::{test, App};
    use fake::Fake;

    const TEST_MAGNET: &str = "magnet:?xt=urn:btih:c12fe1c06bba254a9dc9f519b335aa7c1367a88a";

    fn get_rand_session() -> SessionInfos {
        SessionInfos::try_new(
            (8..20).fake::<String>(),
            (8..20).fake::<String>(),
            TEST_MAGNET.to_string(),
        )
        .unwrap()
    }

    #[actix_rt::test]
    async fn open_file_sharing_session_test() {
        let mut conn = try_establish_redis_conn().await.unwrap();
        let mut app = test::init_service(App::new().configure(|cfg| {
            config_test_app(cfg, conn.clone());
        }))
        .await;
        let mut req_body = OpenSessionReq {
            key: (8..20).fake::<String>(),
            host_id: (8..20).fake::<String>(),
            magnet_link: TEST_MAGNET.to_string(),
        };
        let req = test::TestRequest::post()
            .header("Content-Type", "application/json")
            .set_payload(serde_json::to_string(&req_body).unwrap())
            .uri("/sessions")
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED);
        let infos: SessionInfos = serde_json::from_str(resp.response().body().as_str()).unwrap();
        assert!(redis_eq(&mut conn, &req_body.key, &infos).await);

        req_body.magnet_link = "magnet:invalid".to_string();
        let req2 = test::TestRequest::post()
            .header("Content-Type", "application/json")
            .set_payload(serde_json::to_string(&req_body).unwrap())
            .uri("/sessions")
            .to_request();
        let resp2 = test::call_service(&mut app, req2).await;

        assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn retrieve_existing_session() {
        let mut conn = try_establish_redis_conn().await.unwrap();
        let mut app = test::init_service(App::new().configure(|cfg| {
            config_test_app(cfg, conn.clone());
        }))
        .await;
        let mut session = get_rand_session();
        redis_set(&mut conn, &session.key, &session).await;
        session.revocation_token.clear();
        let req =
            test::TestRequest::get().uri(&*format!("/sessions/{}", &session.key)).to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.response().body().as_str(), serde_json::to_string(&session).unwrap());

        let req2 = test::TestRequest::get().uri("/sessions/bad_id").to_request();
        let resp2 = test::call_service(&mut app, req2).await;

        assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn delete_existing_session() {
        let mut conn = try_establish_redis_conn().await.unwrap();
        let mut app = test::init_service(App::new().configure(|cfg| {
            config_test_app(cfg, conn.clone());
        }))
        .await;
        let session = get_rand_session();
        redis_set(&mut conn, &session.key, &session).await;
        let req = test::TestRequest::delete()
            .header("Content-Type", "application/json")
            .uri(&*format!("/sessions/{}", &session.key))
            .set_payload(String::from("\"bad_token\""))
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let req2 = test::TestRequest::delete()
            .header("Content-Type", "application/json")
            .uri(&*format!("/sessions/{}", &session.key))
            .set_payload(format!("\"{}\"", session.revocation_token))
            .to_request();
        let resp2 = test::call_service(&mut app, req2).await;

        assert_eq!(resp2.status(), StatusCode::OK);
    }
}
