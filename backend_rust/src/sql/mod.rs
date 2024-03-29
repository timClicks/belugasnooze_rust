mod model_alarm;
mod model_timezone;

pub use model_alarm::ModelAlarm;
pub use model_timezone::ModelTimezone;

use anyhow::Result;
use sqlx::{ConnectOptions, SqlitePool};
use tokio::fs;
use tracing::debug;

use crate::env::AppEnv;

/// If file doesn't exist on disk, create
async fn file_exists(filename: &str) {
    if !filename.ends_with(".db") {
        return;
    }
    if fs::metadata(filename).await.is_err() {
        let path = filename
            .split_inclusive('/')
            .filter(|f| !f.ends_with(".db"))
            .collect::<String>();
        fs::create_dir_all(&path).await.unwrap();
        fs::File::create(filename).await.unwrap();
    };
}

/// Open Sqlite pool connection, and return
/// max_connections need to be 1, see https://github.com/launchbadge/sqlx/issues/816
async fn get_db(app_envs: &AppEnv) -> Result<SqlitePool, sqlx::Error> {
    let mut connect_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&app_envs.location_sqlite)
        .serialized(true);
    if !app_envs.trace {
        connect_options.disable_statement_logging();
    }
    let db = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
        .max_connections(app_envs.sql_threads)
        .connect_with(connect_options)
        .await
        .unwrap();
    Ok(db)
}


/// Check if timezone in db, if not then insert
async fn insert_env_timezone(db: &SqlitePool, app_envs: &AppEnv) {
    if ModelTimezone::get(db).await.is_none() {
        ModelTimezone::insert(db, app_envs).await.unwrap();
    }
}

async fn create_tables(db: &SqlitePool) {
    sqlx::query(
        "BEGIN;

		CREATE TABLE IF NOT EXISTS alarm  (
			alarm_id INTEGER PRIMARY KEY AUTOINCREMENT,
			day INTEGER NOT NULL CHECK (day >= 0 AND day <= 6),
			hour INTEGER NOT NULL CHECK (hour >= 0 AND hour <= 23),
			minute INTEGER NOT NULL CHECK (minute >= 0 AND minute <= 59),
			UNIQUE (day, hour, minute)
		) STRICT;
				
		CREATE TABLE IF NOT EXISTS timezone  (
			timezone_id INTEGER PRIMARY KEY AUTOINCREMENT CHECK (timezone_id = 1),
			zone_name TEXT NOT NULL,
			offset_hour INTEGER NOT NULL CHECK (offset_hour >= -23 AND offset_hour <= 23),
			offset_minute INTEGER NOT NULL CHECK (offset_minute >= 0 AND offset_minute <= 59),
			offset_second INTEGER NOT NULL CHECK (offset_second >= 0 AND offset_second <= 59)
		) STRICT;

		COMMIT;
        ",
    )
    .execute(db)
    .await
    .unwrap();
}

/// Init db connection, works if folder/files exists or not
pub async fn init_db(app_envs: &AppEnv) -> Result<SqlitePool, sqlx::Error> {
    file_exists(&app_envs.location_sqlite).await;
    debug!("File should now exists");
    let db = get_db(app_envs).await.unwrap();
    create_tables(&db).await;
    insert_env_timezone(&db, app_envs).await;
    Ok(db)
}

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[cfg(test)]
/// Sql Test
///
/// cargo watch -q -c -w src/ -x 'test sql_mod -- --test-threads=1 --nocapture'
mod tests {
    use super::*;
    use std::{fs, time::SystemTime};
    use time::UtcOffset;

    fn cleanup() {
        fs::remove_dir_all("/ramdrive/test_db_files/").unwrap()
    }

    fn gen_args(timezone: String, hour_offset: i8, location_sqlite: String) -> AppEnv {
        let na = String::from("na");
        AppEnv {
            trace: false,
            location_ip_address: na.clone(),
            location_log_combined: na.clone(),
            timezone,
            location_log_error: na.clone(),
            location_sqlite,
            debug: true,
            start_time: SystemTime::now(),
            utc_offset: UtcOffset::from_hms(hour_offset, 0, 0).unwrap(),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_auth_address: na.clone(),
            ws_password: na,
            sql_threads: 2,
        }
    }

    #[tokio::test]
    async fn sql_mod_exists_created() {
        // FIXTURES
        let name = "testing_file.db";

        // ACTION
        file_exists(name).await;

        // CHECK
        let exists = fs::metadata(name).is_ok();
        assert!(exists);

        // CLEANUP
        fs::remove_file(name).unwrap();
    }

    #[tokio::test]
    async fn sql_mod_exists_nested_created() {
        // FIXTURES
        let path = "/ramdrive/test_db_files/";
        let name = format!("{path}/testing_file.db");

        // ACTION
        file_exists(&name).await;

        // CHECK
        let dir_exists = fs::metadata(path).unwrap().is_dir();
        let exists = fs::metadata(&name).is_ok();
        assert!(exists);
        assert!(dir_exists);

        // CLEANUP
        cleanup()
    }

    #[tokio::test]
    async fn sql_mod_exists_invalid_name() {
        // FIXTURES
        let name = "testing_file.sql";

        // ACTION
        file_exists(name).await;

        // CHECK
        let exists = fs::metadata(name).is_err();
        assert!(exists);
    }

    #[tokio::test]
    async fn sql_mod_db_created() {
        // FIXTURES
        let sql_name = String::from("/ramdrive/test_db_files/sql_file_db_created.db");
        let sql_sham = format!("{sql_name}-shm");
        let sql_wal = format!("{sql_name}-wal");

        let args = gen_args("America/New_York".into(), -5, sql_name.clone());

        // ACTION
        init_db(&args).await.unwrap();

        // CHECK
        assert!(fs::metadata(&sql_name).is_ok());
        assert!(fs::metadata(&sql_sham).is_ok());
        assert!(fs::metadata(&sql_wal).is_ok());

        // CLEANUP
        cleanup()
    }

    #[tokio::test]
    async fn sql_mod_db_created_with_timezone() {
        // FIXTURES
        let sql_name = String::from("/ramdrive/test_db_files/sql_file_db_created_with_timezone.db");
        let timezone = "America/New_York";
        let args = gen_args(timezone.into(), -5, sql_name.clone());
        init_db(&args).await.unwrap();
        let db = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
            .max_connections(1)
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&args.location_sqlite))
            .await
            .unwrap();

        // ACTION
        let result = sqlx::query_as("SELECT * FROM timezone")
            .fetch_one(&db)
            .await;

        // CHECK
        assert!(result.is_ok());
        let result: (i64, String, i64, i64, i64) = result.unwrap();
        assert_eq!(result.0, 1);
        assert_eq!(result.1, "America/New_York");
        assert_eq!(result.2, -5);
        assert_eq!(result.3, 0);
        assert_eq!(result.4, 0);

        // CLEANUP
        cleanup()
    }
}
