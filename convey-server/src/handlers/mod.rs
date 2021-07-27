use actix_web::{web, App, Scope};

pub mod session;

pub fn config_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sessions")
            .service(session::open_file_sharing_session)
            .service(session::get_session_infos)
            .service(session::close_session),
    );
}
