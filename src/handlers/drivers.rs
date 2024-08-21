use actix_web::web;
use sqlx::MySqlPool;
use tokio::task::JoinSet;
use tracing::{info, warn};

use crate::models::api_response::ApiResponse;
use crate::models::db_objects::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/test").to(test));
    cfg.service(web::resource("/all_drivers").get(get_all_drivers));
    cfg.service(web::resource("/{driver_id}/info").get(get_driver_information));
}

async fn test() -> ApiResponse<()> {
    ApiResponse::new_ok_no_data("Driver route test")
}

async fn get_all_drivers(pool: web::Data<MySqlPool>) -> ApiResponse<Vec<DriverInfo>> {
    let pool = pool.get_ref();
    let query = sqlx::query_as!(
        DriverInfo,
        "SELECT driver_id, username, driver_number, driver_image_url FROM driver"
    )
    .fetch_all(pool)
    .await;

    match query {
        Ok(drivers) => ApiResponse::new_ok("Successfully fetched drivers", drivers),
        Err(e) => {
            warn!("Failed to fetch drivers: {:?}", e);
            ApiResponse::new_internal_error("Failed to fetch drivers")
        }
    }

}

async fn get_driver_information(
    pool: web::Data<MySqlPool>,
    driver_id: web::Path<i32>,
) -> ApiResponse<Driver> {
    let pool = pool.get_ref();
    let driver_id: i32 = driver_id.into_inner();

    let driver_info = sqlx::query_as!(
        DriverInfo,
        "SELECT driver_id, username, driver_number, driver_image_url FROM driver WHERE driver_id = ?",
        driver_id
    )
    .fetch_one(pool)
    .await;

    let driver_info = match driver_info {
        Ok(driver_info) => driver_info,
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                info!("Driver not found: {:?}", e);
                return ApiResponse::new_not_found_error("Driver not found");
            } else {
                warn!("Failed to fetch driver information: {:?}", e);
                return ApiResponse::new_internal_error("Failed to fetch driver information");
            }
        }
    };

    let seat_id = sqlx::query!(
        "SELECT seat_id FROM drives_in WHERE driver_id = ?",
        driver_id
    ).fetch_all(pool).await;

    let seat_id: Vec<i32> = match seat_id{
        Ok(seat_id) => seat_id.iter().map(|x| x.seat_id).collect(),
        Err(e) => {
            warn!("Failed to fetch seat information: {:?}", e);
            return ApiResponse::new_internal_error("Failed to fetch seat information");
        }
    };

    let mut joinset = JoinSet::new();
    for &seat_id in seat_id.iter(){
        let pool = pool.clone();
        joinset.spawn(async move {
            let race_results = sqlx::query_as!(
                RaceResult,
                r#"
                SELECT 
                    result.position AS position, 
                    bot_result, 
                    result.pole AS pole, 
                    points.leading_lap AS leading_lap, 
                    points.fastest_lap as fastest_lap, 
                    qualy_result, 
                    result.season as season, 
                    races.race_id as race_id, 
                    race_name, 
                    points
                FROM result
                JOIN races ON result.race_id = races.race_id
                JOIN points ON result.season = points.season 
                    AND result.position = points.position 
                    AND result.pole = points.pole 
                    AND result.leading_lap = points.leading_lap 
                    AND result.fastest_lap = points.fastest_lap 
                    AND races.season = points.season
                WHERE result_id IN (SELECT result_id FROM has_result WHERE seat_id = ?);
                "#,
                seat_id
            ).fetch_all(&pool).await;
    
    
            let race_results = match race_results{
                Ok(teams) => teams,
                Err(e) => {
                    warn!("Failed to fetch result information : {:?}", e);
                    return Err(ApiResponse::new_internal_error("Unabbe to fetch teams"));
                }
            };
    
            
            let teams = sqlx::query_as!(
                Team,
                "SELECT team_id, name, color FROM team WHERE team_id IN (SELECT team_id FROM drives_for WHERE drives_for.seat_id = ?)",
                seat_id
            ).fetch_all(&pool).await;
    
            let teams = match teams {
                Ok(teams) => teams,
                Err(e) => {
                    warn!("Failed to fetch team information: {:?}", e);
                    return Err(ApiResponse::new_internal_error("Failed to fetch team information"));
                }
            };
    
            Ok(Seat{
                seat_id,
                team: teams,
                results: race_results,
            })
        });
    }

    let mut seats : Vec<Seat> = Vec::new();
    while let Some(seat) = joinset.join_next().await{
        match seat.unwrap(){
            Ok(seat) => seats.push(seat),
            Err(e) => return e,
        }
    }


    ApiResponse::new_ok("succes", Driver{
        driver_id: driver_info.driver_id,
        username: driver_info.username,
        driver_number: driver_info.driver_number,
        driver_image_url: driver_info.driver_image_url,
        seats,
    })
}
