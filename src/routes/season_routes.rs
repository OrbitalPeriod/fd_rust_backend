use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(crate::handlers::season::config);
}