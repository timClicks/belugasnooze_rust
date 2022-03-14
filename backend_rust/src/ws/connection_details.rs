use std::time::{Duration, Instant};
use time::OffsetDateTime;
use tokio::time::sleep;
use tracing::debug;

#[derive(Debug)]
pub struct ConnectionDetails {
    pub count: usize,
    wait: Wait,
    connection_instant: Option<Instant>,
    is_connected: bool,
}

#[derive(Debug)]
enum Wait {
    Short,
    Long,
}

impl ConnectionDetails {
    pub fn new() -> Self {
        Self {
            count: 0,
            wait: Wait::Short,
            connection_instant: None,
            is_connected: false,
        }
    }

    // pub fn should_try_reconnect(&self) -> bool {
    //     self.count < 40
    // }

    /// increase attempt count, and set delay to long if 20+ attempts
    /// Set is_connected to 0 and time to none
    pub fn fail_connect(&mut self) {
        self.count += 1;
        self.is_connected = false;
        self.connection_instant = None;
        if self.count >= 20 {
            // if self.count >= 5 {
            self.wait = Wait::Long;
        }
    }

    /// delay the recconnect attempt by x seconds, depedning on ho wmany attempts already made
    pub async fn reconnect_delay(&self) {
        if self.count == 0 {
            return;
        }
        let sec = match self.wait {
            Wait::Short => 5,
            Wait::Long => 60,
        };
        sleep(Duration::from_secs(sec)).await
    }

    /// called on each connect, to reset connection, count etc
    pub fn valid_connect(&mut self) {
        self.wait = Wait::Short;
        self.count = 1;
        self.is_connected = true;
        let now = OffsetDateTime::now_utc();
        self.connection_instant = Some(Instant::now());
        let connected_at = format!("{} {}", now.date(), now.time());
        debug!(%connected_at);
    }
}
