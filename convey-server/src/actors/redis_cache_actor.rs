use std::convert::TryFrom;
use std::time::SystemTime;

use actix::{Actor, Context, Handler, Message, ResponseFuture};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::errors::ConveyError;
use crate::utils::is_magnet_link_valid;
use redis::aio::ConnectionManager;

type HandlerResponse = Result<SessionInfos, ConveyError>;

#[derive(Message)]
#[rtype(result = "Result<SessionInfos, ConveyError>")]
pub enum CacheMsg {
    OpenSession(OpenSessionReq),
    CloseSession { id: String, revocation_token: String },
    GetSessionInfos(String),
}

#[derive(Serialize, Deserialize)]
pub struct OpenSessionReq {
    key: String,
    host_id: String,
    magnet_link: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionInfos {
    key: String,
    host_id: String,
    magnet_link: String,
    opened_at: SystemTime,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub revocation_token: String,
}

pub struct RedisCacheActor {
    conn: redis::aio::ConnectionManager,
}

impl TryFrom<OpenSessionReq> for SessionInfos {
    type Error = String;

    fn try_from(req: OpenSessionReq) -> Result<SessionInfos, Self::Error> {
        if !is_magnet_link_valid(req.magnet_link.as_str()) {
            Err("Invalid magnet link.".to_string())
        } else {
            Ok(Self {
                key: req.key,
                host_id: req.host_id,
                magnet_link: req.magnet_link,
                opened_at: SystemTime::now(),
                revocation_token: "random".to_string(),
            })
        }
    }
}

impl RedisCacheActor {
    pub fn new(connection_manager: redis::aio::ConnectionManager) -> Self {
        Self { conn: connection_manager }
    }

    async fn open_session(mut conn: ConnectionManager, req: OpenSessionReq) -> HandlerResponse {
        let key_exist: i32 = conn.exists(&req.key).await?;

        if key_exist == 1 {
            return Err(ConveyError::BadRequest("This session is already open.".to_string()));
        }
        match SessionInfos::try_from(req) {
            Ok(infos) => {
                conn.set(&infos.key, serde_json::to_string(&infos)?).await?;
                Ok(infos)
            }
            Err(err) => Err(ConveyError::BadRequest(err)),
        }
    }

    async fn close_session(
        mut conn: ConnectionManager,
        id: String,
        token: String,
    ) -> HandlerResponse {
        let val: String = conn.get(&id).await?;
        let session: SessionInfos = serde_json::from_str(val.as_str())?;

        if session.revocation_token == token {
            conn.del(&id).await?;
            Ok(session)
        } else {
            Err(ConveyError::UnauthorizedError())
        }
    }

    async fn retrieve_session(mut conn: ConnectionManager, id: String) -> HandlerResponse {
        let val: Option<String> = conn.get(&id).await?;

        if let Some(session_raw) = val {
            let mut session: SessionInfos = serde_json::from_str(&session_raw)?;
            session.revocation_token.clear();
            Ok(session)
        } else {
            Err(ConveyError::NotFound())
        }
    }
}

impl Actor for RedisCacheActor {
    type Context = Context<Self>;
}

impl Handler<CacheMsg> for RedisCacheActor {
    type Result = ResponseFuture<HandlerResponse>;

    fn handle(&mut self, msg: CacheMsg, _ctx: &mut Self::Context) -> Self::Result {
        let conn = self.conn.clone();
        Box::pin(async move {
            match msg {
                CacheMsg::OpenSession(req) => Self::open_session(conn, req).await,
                CacheMsg::CloseSession { id, revocation_token } => {
                    Self::close_session(conn, id, revocation_token).await
                }
                CacheMsg::GetSessionInfos(id) => Self::retrieve_session(conn, id).await,
            }
        })
    }
}
