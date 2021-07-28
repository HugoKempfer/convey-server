use std::convert::TryFrom;
use std::time::SystemTime;

use actix::{Actor, Context, Handler, Message, ResponseFuture};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::errors::ConveyError;
use crate::utils::{gen_token, is_magnet_link_valid};
use redis::aio::ConnectionManager;

pub type HandlerResponse = Result<SessionInfos, ConveyError>;

#[derive(Message, Clone)]
#[rtype(result = "Result<SessionInfos, ConveyError>")]
pub enum SessionMsg {
    OpenSession(OpenSessionReq),
    CloseSession { id: String, revocation_token: String },
    GetSessionInfos(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenSessionReq {
    pub(crate) key: String,
    pub(crate) host_id: String,
    pub(crate) magnet_link: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionInfos {
    pub key: String,
    pub host_id: String,
    pub magnet_link: String,
    pub opened_at: SystemTime,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub revocation_token: String,
}

impl SessionInfos {
    pub fn try_new(key: String, host_id: String, magnet_link: String) -> Result<Self, ConveyError> {
        if !is_magnet_link_valid(magnet_link.as_str()) {
            return Err(ConveyError::BadRequest("Invalid magnet link.".to_string()));
        }
        if key.is_empty() || host_id.is_empty() {
            return Err(ConveyError::BadRequest("Bad ids.".to_string()));
        }
        Ok(Self {
            key,
            host_id,
            magnet_link,
            opened_at: SystemTime::now(),
            revocation_token: gen_token()?,
        })
    }
}

impl TryFrom<OpenSessionReq> for SessionInfos {
    type Error = ConveyError;

    fn try_from(req: OpenSessionReq) -> Result<SessionInfos, Self::Error> {
        Self::try_new(req.key, req.host_id, req.magnet_link)
    }
}

pub struct RedisCacheActor {
    conn: redis::aio::ConnectionManager,
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
        let infos = SessionInfos::try_from(req)?;
        conn.set(&infos.key, serde_json::to_string(&infos)?).await?;
        Ok(infos)
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

impl Handler<SessionMsg> for RedisCacheActor {
    type Result = ResponseFuture<HandlerResponse>;

    fn handle(&mut self, msg: SessionMsg, _ctx: &mut Self::Context) -> Self::Result {
        let conn = self.conn.clone();
        Box::pin(async move {
            match msg {
                SessionMsg::OpenSession(req) => Self::open_session(conn, req).await,
                SessionMsg::CloseSession { id, revocation_token } => {
                    Self::close_session(conn, id, revocation_token).await
                }
                SessionMsg::GetSessionInfos(id) => Self::retrieve_session(conn, id).await,
            }
        })
    }
}

#[cfg(test)]
mod tests {}
