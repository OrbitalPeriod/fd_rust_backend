

use chrono::Utc;
use serde::de::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use sqlx::prelude::FromRow;
use sqlx::Column;
use sqlx::Database;
use sqlx::Decode;
use sqlx::Postgres;
use sqlx::Row;
use sqlx::Type;
use tracing::warn;
use tracing::{info, debug};
use serde::Serializer;
use sqlx::postgres::PgRow;

#[derive(Debug, Clone, Serialize)]
pub struct Driver {
    pub username: String,
    pub driver_number: i32,
    pub driver_image_url: String,
    pub driver_id: i32,
    pub country : String,
    pub birthday :  Option<chrono::NaiveDate>,
    pub seats: Vec<Seat>,
    pub season_results : Vec<SeasonResult>
}

#[derive(Debug, Clone, Serialize)]
pub struct SeasonResult{
    pub driver_result : i32,
    pub team_result : i32,
    pub season : i32,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow, Clone)]
pub struct DriverInfo {
    pub driver_id: i32,
    pub username: String,
    pub driver_number: i32,
    pub driver_image_url: String,
    pub country : String,
    pub birthday : Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Seat {
    pub seat_id: i32,
    pub results: Vec<RaceResult>,
    pub team: Team,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Team {
    pub team_id: i32,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RaceResult {
    pub position: Position,
    pub bot_result: bool,
    pub pole: bool,
    pub leading_lap: bool,
    pub fastest_lap: bool,
    pub qualy_result: Option<i32>,
    pub season: i32,
    pub race_id: i32,
    pub race_name: String,
    pub points: i32,
}

#[derive(Debug, Clone)]
pub enum Position {
    Finished(i32),
    Dnf,
    Dsq,
    Dns,
}

impl Position {
    pub fn new(position: i32) -> Self {
        match position {
            101 => Position::Dnf,
            111 => Position::Dsq,
            100 => Position::Dns,
            _ => Position::Finished(position),
        }
    }
}

impl Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match *self {
            Position::Finished(pos) => pos,
            Position::Dnf => 101,
            Position::Dsq => 111,
            Position::Dns => 100,
        };
        serializer.serialize_i32(value)
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Position, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        let s: i32 = s
            .parse()
            .map_err(|_| serde::de::Error::custom("could not map position to number"))?;
        Ok(Self::new(s))
    }
}

impl<'r> FromRow<'r, PgRow> for Position {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let value: Result<i32, sqlx::Error> = row.try_get("position");
        match value {
            Ok(value) => Ok(Position::Finished(value)),
            Err(_) => Err(sqlx::Error::TypeNotFound {
                type_name: "position".into(),
            }),
        }
    }
}

impl<'r, DB : Database> sqlx::Decode<'r, DB> for Position
where i32: Decode<'r, DB>
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let number = <i32 as Decode<DB>>::decode(value)?;
        Ok(Position::new(number))
    }
}

impl Type<Postgres> for Position{
    fn type_info() -> <Postgres as Database>::TypeInfo {
        <i32 as Type<Postgres>>::type_info()
    }
}

impl From<i32> for Position {
    fn from(value: i32) -> Self {
        Position::new(value)
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Season {
    pub season: i32,
    pub season_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SeasonInfo{
    pub season : Season,
    pub races: Vec<Race>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Race{
    pub race_name : String,
    pub season : i32,
    pub results : Vec<PersonalResult>,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct RaceInfo{
    pub race_name : String,
    pub season : i32,
    pub race_id : i32,
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct PersonalResult{
    #[sqlx(flatten)]
    pub race_result : RaceResult,
    #[sqlx(flatten)]
    pub driver_info : DriverInfo,
    #[sqlx(flatten)]
    pub team : Team,
}