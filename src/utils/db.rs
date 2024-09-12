use std::{collections::HashMap, error::Error};

use sqlx::{Database, Executor, Pool, Postgres};
use tracing::warn;

use crate::models::db_objects::{SeasonResult, Team};

pub async fn update_season_results(pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
    let mut tx = pool.begin().await?;

    let seasons =
        sqlx::query!("SELECT season FROM seasons WHERE finished=true and requires_recalc = true")
            .fetch_all(&mut *tx)
            .await?;
    for season in seasons {
        let season = season.season;
        sqlx::query!("DELETE FROM season_result WHERE season = $1", season)
            .execute(&mut *tx)
            .await?;

        let mut personal_results = sqlx::query!("
            SELECT driver_id, sum(points) as total_points, max(team_id) as team_id
            FROM result
                JOIN public.has_result hr on result.result_id = hr.result_id
                JOIN public.drives_for df on hr.seat_id = df.seat_id
                JOIN public.drives_in di on hr.seat_id = di.seat_id
                JOIN points p on result.season = p.season and result.position = p.position and result.pole = p.pole and result.leading_lap = p.leading_lap and result.fastest_lap = p.fastest_lap
            WHERE p.season = $1
            GROUP BY driver_id;", season).fetch_all(&mut *tx).await?;
        personal_results.sort_unstable_by(|a, b| b.total_points.cmp(&a.total_points));
        let personal_results: Vec<(usize, i32, i32)> = personal_results
            .iter()
            .enumerate()
            .map(|(position, record)| {
                (
                    position + 1,
                    record.driver_id,
                    record
                        .team_id
                        .expect("Driver got Points despite not being in a team???"),
                )
            })
            .collect();

        let mut team_results = sqlx::query!("
            SELECT team_id, sum(points) as total_points
            FROM result
                JOIN public.has_result hr on result.result_id = hr.result_id
                JOIN public.drives_for df on hr.seat_id = df.seat_id
                JOIN points p on result.season = p.season and result.position = p.position and result.pole = p.pole and result.leading_lap = p.leading_lap and result.fastest_lap = p.fastest_lap
            WHERE p.season = $1
            GROUP BY team_id;", season).fetch_all(&mut *tx).await?;
        team_results.sort_unstable_by(|a, b| b.total_points.cmp(&a.total_points));

        let mut team_result_map = HashMap::new();
        team_results.iter().enumerate().for_each(|(index, item)| {
            let _ = team_result_map.insert(item.team_id, index + 1);
        });

        for (driver_id, position, team_position) in personal_results.iter().map(|(position, driver_id, team_id)| (driver_id, position, team_result_map.get(team_id).expect("Drvied scored points without being in ateam that scored points that season"))){
            sqlx::query!("INSERT INTO season_result (driver_id, driver_result, team_result, season) VALUES ($1, $2, $3, $4)",
            *driver_id,
            *position as i32,
            *team_position as i32,
            season
        ).execute(&mut *tx).await?;
    }
    }

    sqlx::query!("UPDATE seasons SET requires_recalc=false")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_teams<'e, 'c, T>(pool: T, driver_id: i32) -> Result<Vec<Team>, sqlx::Error>
where
    T: 'e + Executor<'c, Database = Postgres>,
{
    sqlx::query_as!(
        Team,
        "SELECT team.team_id, color, name
                FROM team
            JOIN public.drives_for df on team.team_id = df.team_id
            JOIN public.drives_in di on df.seat_id = di.seat_id
            WHERE driver_id = $1;",
        driver_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_season_results<'e, 'c, T>(
    pool: T,
    driver_id: i32,
) -> Result<Vec<SeasonResult>, sqlx::Error>
where
    T: 'e + Executor<'c, Database = Postgres>,
{
    sqlx::query_as!(SeasonResult, 
        "SELECT driver_result, team_result, season FROM season_result WHERE driver_id = $1"
        , driver_id).fetch_all(pool).await
}
