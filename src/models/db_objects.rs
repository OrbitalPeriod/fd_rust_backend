use serde::de::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use sqlx::mysql::MySqlRow;
use sqlx::prelude::FromRow;
use sqlx::Column;
use sqlx::MySqlPool;
use sqlx::Row;

#[derive(Debug, Clone, Serialize)]
pub struct Boolean(bool);

impl From<i8> for Boolean {
    fn from(value: i8) -> Self {
        if value == 0{
            Boolean(false)
        } else {
            Boolean(true)
        }
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct Driver {
    pub username: String,
    pub driver_number: i32,
    pub driver_image_url: String,
    pub driver_id: i32,
    pub seats: Vec<Seat>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct DriverInfo {
    pub driver_id: i32,
    pub username: String,
    pub driver_number: i32,
    pub driver_image_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Seat {
    pub seat_id: i32,
    pub results: Vec<RaceResult>,
    pub team: Vec<Team>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Team {
    pub team_id: i32,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RaceResult {
    pub position: Position,
    pub bot_result: Boolean,
    pub pole: Boolean,
    pub leading_lap: Boolean,
    pub fastest_lap: Boolean,
    pub qualy_result: Option<i32>,
    pub season: i32,
    pub race_id: i32,
    pub race_name: String,
    pub points: i32,
}

#[derive(Debug, Clone, Serialize)]
pub enum Position {
    Finished(i32),
    Dnf,
    Dsq,
    Dns,
}

impl Position {
    pub fn new(position: i32) -> Self {
        match position {
            111 => Position::Dnf,
            101 => Position::Dsq,
            100 => Position::Dns,
            _ => Position::Finished(position),
        }
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

impl<'r> FromRow<'r, MySqlRow> for Position {
    fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
        let value: Result<i32, sqlx::Error> = row.try_get("position");
        match value {
            Ok(value) => Ok(Position::Finished(value)),
            Err(_) => Err(sqlx::Error::TypeNotFound {
                type_name: "position".into(),
            }),
        }
    }
}

impl From<i32> for Position {
    fn from(value: i32) -> Self {
        Position::new(value)
    }
}
