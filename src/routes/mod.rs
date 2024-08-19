mod driver_routes;
mod season_routes;

use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/driver").configure(driver_routes::config));
    cfg.service(web::scope("/season").configure(season_routes::config));
}
