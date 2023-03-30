use crate::{util::empty_string_as_none, ApiError, ApiState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, sqlx::FromRow, Debug)]
struct Game {
    name: String,
    boxart: String,
    system: String,
    region: String,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    q: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    system: Option<String>,
}

pub async fn handle(
    Query(params): Query<Params>,
    State(ApiState { openvgdb, .. }): State<ApiState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some(query) = params.q else {
        return Err(ApiError::MissingQuery("missing query param"));
    };

    let query = query
        .to_lowercase()
        .split_whitespace()
        .map(|substring| {
            substring
                .chars()
                .filter(|x| char::is_alphanumeric(*x))
                .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join(" AND ");

    let mut conn = openvgdb.acquire().await?;
    let rows = sqlx::query(
        r#"
        SELECT
            name,
            boxart,
            system,
            region
        FROM releases_fts
        WHERE
            name MATCH(?1) AND
            (?2 IS NULL OR LOWER(system) IS LOWER(?2)) AND
            boxart IS NOT NULL
        ORDER BY RANK;
        "#,
    )
    .bind(query)
    .bind(&params.system)
    .fetch_all(&mut conn)
    .await?;

    let games: Vec<Game> = rows
        .iter()
        .filter_map(|row| Game::from_row(row).ok())
        .collect();

    Ok(Json(games))
}
