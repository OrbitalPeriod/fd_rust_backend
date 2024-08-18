mod driver_routes;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/driver").configure(driver_routes::config));
}
