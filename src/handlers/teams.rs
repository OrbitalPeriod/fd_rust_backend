use actix_web::web;
use sqlx::{Pool, Postgres};
use tracing::warn;

use crate::models::{api_response::ApiResponse, db_objects::Team};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/all_teams").get(get_all_teams));
}

pub async fn get_all_teams(pool: web::Data<Pool<Postgres>>) -> ApiResponse<Vec<Team>> {
    let pool = pool.get_ref();

    let query = sqlx::query_as!(Team, "SELECT name, color, team_id FROM team")
        .fetch_all(pool)
        .await;

    match query{
        Ok(data) => {
            ApiResponse::new_ok("Query succesfull", data)
        }
        Err(e) => {
            warn!("failed to fetch teams: {:?}", e);
            ApiResponse::new_internal_error("Failed to fetch teams")
        }
    }
}
