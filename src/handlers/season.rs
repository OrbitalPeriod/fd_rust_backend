use std::result;

use actix_web::web;
use itertools::Itertools;
use sqlx::MySqlPool;
use tracing::warn;

use crate::models::api_response::ApiResponse;
use crate::models::db_objects::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/test").to(test));
    cfg.service(web::resource("/all_seasons").get(get_all_seasons));
    cfg.service(web::resource("{season}/info").get(get_season_info));
}

async fn test() -> ApiResponse<()> {
    ApiResponse::new_ok_no_data("Season test route")
}

async fn get_all_seasons(pool: web::Data<MySqlPool>) -> ApiResponse<Vec<Season>> {
    let pool = pool.get_ref();

    let seasons = sqlx::query_as!(Season, "SELECT season, season_name FROM seasons")
        .fetch_all(pool)
        .await;

    match seasons {
        Ok(seasons) => ApiResponse::new_ok("Successfully fetched seasons", seasons),
        Err(e) => {
            warn!("Failed to fetch seasons: {:?}", e);
            ApiResponse::new_internal_error("Failed to fetch seasons")
        }
    }
}

async fn get_season_info(
    pool: web::Data<MySqlPool>,
    season: web::Path<i32>,
) -> ApiResponse<SeasonInfo> {
    let pool = pool.get_ref();
    let season_number = season.into_inner();

    let season = {
        let season = sqlx::query_as!(
            Season,
            "SELECT season, season_name FROM seasons WHERE season = ?",
            season_number
        )
        .fetch_one(pool)
        .await;
        match season {
            Ok(season) => season,
            Err(sqlx::Error::RowNotFound) => {
                return ApiResponse::new_not_found_error("Season not found");
            }
            Err(e) => {
                warn!("Failed to fetch season: {:?}", e);
                return ApiResponse::new_internal_error("Failed to fetch season");
            }
        }
    };

    let results = {
        let query: Result<Vec<PersonalResult>, sqlx::Error> = sqlx::query_as("
            SELECT result.position, result.bot_result, result.pole, result.leading_lap, result.fastest_lap, result.qualy_result, result.season, result.race_id, has_result.seat_id, has_result.result_id, drives_in.driver_id, drives_in.seat_id, d.driver_id, d.username, d.driver_number, d.driver_image_url, drives_for.seat_id, drives_for.team_id, t.team_id, t.name, t.color, r.race_id, r.race_name, r.season, p.season, p.position, p.pole, p.leading_lap, p.fastest_lap, p.position, bot_result, p.pole, p.leading_lap, p.fastest_lap, qualy_result, r.season, r.race_id, race_name, points, d.driver_id, username, driver_number, driver_image_url, t.team_id, name, color
            FROM result
            JOIN has_result ON result.result_id = has_result.result_id
            JOIN drives_in ON drives_in.seat_id = has_result.seat_id
            JOIN FormulaDestruction.driver d on drives_in.driver_id = d.driver_id
            JOIN drives_for on drives_in.seat_id = drives_for.seat_id
            JOIN FormulaDestruction.team t on drives_for.team_id = t.team_id
            JOIN FormulaDestruction.races r on result.race_id = r.race_id
            JOIN FormulaDestruction.points p on result.season = p.season and result.position = p.position and result.pole = p.pole and result.leading_lap = p.leading_lap and result.fastest_lap = p.fastest_lap
            WHERE result.season = ?;"
        )
        .bind(season_number)
        .fetch_all(pool).await;

        match query {
            Ok(result) => result,
            Err(e) => {
                warn!("failed to fetch results: {:?}", e);
                return ApiResponse::new_internal_error("Failed to fetch results");
            }
        }
    };

    let races: Vec<Race> = results
        .iter()
        .chunk_by(|x| x.race_result.race_id)
        .into_iter()
        .flat_map(|mut race| {
            let first = race.1.next();

            match first {
                Some(first) => {
                    let mut races: Vec<PersonalResult> = race.1.cloned().collect();
                    races.push(first.clone());

                    races.sort_by(|a: &PersonalResult, b| {
                        a.race_result.race_id.cmp(&b.race_result.race_id)
                    });

                    Some(Race {
                        race_name: first.race_result.race_name.clone(),
                        season: season_number,
                        results: races,
                    })
                }
                None => None,
            }
        })
        .collect();

    let season = SeasonInfo { season, races };

    ApiResponse::new_ok("Successfully fetched season", season)
}
