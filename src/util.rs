use {
    crate::ApiError,
    serde::{de, Deserialize, Deserializer},
    sqlx::{Executor, SqlitePool},
    std::{fmt, str::FromStr},
    url::{Host, Url},
};

/// Serde deserialization decorator to map empty Strings to None,
pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[inline]
pub fn is_global_ip_url(url: &Url) -> bool {
    match url.scheme() {
        "http" | "https" => match url.host() {
            Some(Host::Ipv4(ip)) => ip.is_global(),
            Some(Host::Ipv6(ip)) => ip.is_global(),
            Some(Host::Domain(str)) if str != "localhost" => true,
            _ => false,
        },
        _ => false,
    }
}

pub async fn setup_openvgdb(path: &str) -> Result<sqlx::SqlitePool, ApiError> {
    let pool = SqlitePool::connect(path).await?;
    let mut conn = pool.acquire().await?;
    let exists = conn
        .fetch_optional(
            "select DISTINCT tbl_name from sqlite_master where tbl_name = 'releases_fts'",
        )
        .await?
        .is_some();
    if !exists {
        conn.execute(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS releases_fts USING fts5 (
                id,
                name,
                boxart,
                system,
                region
            );
            "#,
        )
        .await?;
        conn.execute(
            r#"
            INSERT INTO releases_fts(id, name, boxart, system, region)
            SELECT
                releaseID,
                releaseTitleName,
                releaseCoverFront,
                TEMPsystemShortName,
                TEMPregionLocalizedName
            FROM RELEASES
            WHERE LOWER(TEMPsystemShortName) IN ('gba', 'gb', 'gbc', 'nes', 'snes', 'sms', 'gg');
            "#,
        )
        .await?;
    }
    Ok(pool)
}
