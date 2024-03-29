use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::fs::read_to_string;

use crate::{env::AppEnv, sql::ModelTimezone};

#[derive(Debug, Serialize, Deserialize)]
pub struct SysInfo {
    // Actually only need to send time_zone, and let front end js work out that time based on that
    // pub pi_time: String,
    pub uptime: usize,
    pub version: String,
    pub internal_ip: String,
    pub uptime_app: String,
    pub time_zone: String,
}

impl SysInfo {
    async fn get_ip(app_envs: &AppEnv) -> String {
        let na = String::from("N/A");
        let ip = read_to_string(&app_envs.location_ip_address)
            .await
            .unwrap_or_else(|_| na.clone());
        let output = if ip.len() > 1 {
            ip.trim().to_owned()
        } else {
            na
        };
        output
    }

    async fn get_uptime() -> usize {
        let uptime = read_to_string("/proc/uptime").await.unwrap();
        let (uptime, _) = uptime.split_once('.').unwrap();
        uptime.parse::<usize>().unwrap_or(0)
    }

    pub async fn new(db: &SqlitePool, app_envs: &AppEnv) -> Result<Self> {
        let db_timezone = ModelTimezone::get(db).await.unwrap();
        // Not needed, can just work out the time front end based soley on the timezone
        // let time = OffsetDateTime::now_utc().to_offset(
        //     UtcOffset::from_hms(
        //         db_timezone.offset_hour,
        //         db_timezone.offset_minute,
        //         db_timezone.offset_second,
        //     )
        //     .unwrap(),
        // );

        // let pi_time = format!(
        //     "{:0>2}:{:0>2}:{:0>2}",
        //     time.hour(),
        //     time.minute(),
        //     time.second()
        // );

        Ok(Self {
            // pi_time,
            time_zone: db_timezone.zone_name,
            internal_ip: Self::get_ip(app_envs).await,
            uptime: Self::get_uptime().await,
            uptime_app: match std::time::SystemTime::now().duration_since(app_envs.start_time) {
                Ok(value) => value.as_secs().to_string(),
                Err(_) => "N/A".to_string(),
            },
            version: env!("CARGO_PKG_VERSION").into(),
        })
    }
}

// SysInfo tests
//
/// cargo watch -q -c -w src/ -x 'test sysinfo -- --test-threads=1 --nocapture'
#[cfg(test)]
mod tests {
    use crate::sql::init_db;
    use std::{fs, sync::Arc, time::SystemTime};
    use time::UtcOffset;

    use super::*;

    async fn setup_test_db(
        file_name: &str,
        location_ip_address: String,
    ) -> (Arc<SqlitePool>, AppEnv) {
        let location_sqlite = format!("/ramdrive/test_db_files/{}.db", file_name);
        let na = String::from("na");
        let env = AppEnv {
            trace: false,
            location_ip_address,
            location_log_combined: na.clone(),
            timezone: "America/New_York".to_owned(),
            location_log_error: na.clone(),
            location_sqlite,
            debug: true,
            start_time: SystemTime::now(),
            utc_offset: UtcOffset::from_hms(-5, 0, 0).unwrap(),
            ws_address: na.clone(),
            ws_apikey: na.clone(),
            ws_auth_address: na.clone(),
            ws_password: na,
            sql_threads: 1,
        };
        let db = Arc::new(init_db(&env).await.unwrap());
        (db, env)
    }

    fn cleanup() {
        fs::remove_dir_all("/ramdrive/test_db_files/").unwrap()
    }

    #[tokio::test]
    async fn sysinfo_getuptime_ok() {
        // FIXTURES
        let _ = setup_test_db("sysinfo_getuptime_ok", "".to_owned()).await;

        // ACTIONS
        let result = SysInfo::get_uptime().await;

        // CHECK
        // Assumes ones computer has been turned on for one minute
        assert!(result > 60);
        cleanup();
    }

    #[tokio::test]
    async fn sysinfo_get_ip_na() {
        // FIXTURES
        let app_envs = setup_test_db("sysinfo_get_ip_na", "".to_owned()).await;

        // ACTIONS
        let result = SysInfo::get_ip(&app_envs.1).await;

        // CHECK
        assert_eq!(result, "N/A");
        cleanup();
    }

    #[tokio::test]
    async fn sysinfo_get_ip_ok() {
        // FIXTURES
        let app_envs = setup_test_db("sysinfo_get_ip_ok", "./ip.addr".to_owned()).await;

        // ACTIONS
        let result = SysInfo::get_ip(&app_envs.1).await;

        // CHECK
        assert_eq!(result, "127.0.0.1");
        cleanup();
    }

    #[tokio::test]
    async fn sysinfo_get_sysinfo_ok() {
        // FIXTURES
        let app_envs = setup_test_db("sysinfo_get_sysinfo_ok", "./ip.addr".to_owned()).await;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // ACTIONS
        let result = SysInfo::new(&app_envs.0, &app_envs.1).await;

        // CHECK
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.internal_ip, "127.0.0.1");
        assert_eq!(result.time_zone, "America/New_York");
        assert_eq!(result.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(result.uptime_app, "1");
        // TODO need to check pi_time with regex?
        // assert!(result.pi_time.len() == 8);
        // Again assume ones computer has been turned on for one minute
        assert!(result.uptime > 60);
        cleanup();
    }
}
