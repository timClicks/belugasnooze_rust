use sqlx::SqlitePool;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use time::{OffsetDateTime, Time, UtcOffset};
use tokio::sync::{broadcast::Sender, Mutex};
use tracing::trace;

use crate::{
    light::LightControl,
    sql::{ModelAlarm, ModelTimezone},
    thread_channel::ChannelMessage,
};

#[derive(Debug)]
pub struct AlarmSchedule {
    alarms: Vec<ModelAlarm>,
    time_zone: ModelTimezone,
    offset: UtcOffset,
}

impl AlarmSchedule {
    async fn new(db: &SqlitePool) -> Self {
        let time_zone = ModelTimezone::get(db).await.unwrap();
        let offset = UtcOffset::from_hms(
            time_zone.offset_hour,
            time_zone.offset_minute,
            time_zone.offset_second,
        )
        .unwrap();
        Self {
            alarms: ModelAlarm::get_all(db).await.unwrap(),
            time_zone,
            offset,
        }
    }

    /// Remove all alarms from vector
    pub fn clear_all(&mut self) {
        self.alarms.clear();
    }

    /// Clear current alarms, get alarms from db and set as self.alarms
    /// Also update + replace timezone + offset
    pub async fn refresh_alarms(&mut self, db: &SqlitePool) {
        Self::clear_all(self);
        self.alarms = ModelAlarm::get_all(db).await.unwrap();
        Self::refresh_timezone(self, db).await;
    }

    /// Get timezone from db and store into self, also update offset
    pub async fn refresh_timezone(&mut self, db: &SqlitePool) {
        let time_zone = ModelTimezone::get(db).await.unwrap();
        let offset = UtcOffset::from_hms(
            time_zone.offset_hour,
            time_zone.offset_minute,
            time_zone.offset_second,
        )
        .unwrap();
        self.offset = offset;
        self.time_zone = time_zone
    }

    /// Remove alarm from vector by id
    pub fn remove_alarm(&mut self, id: i64) {
        let alarm_item = self.alarms.iter().enumerate().find(|i| i.1.alarm_id == id);
        if let Some((index, _)) = alarm_item {
            self.alarms.remove(index);
        }
    }

}

pub struct CronAlarm {
    alarm_schedule: Arc<Mutex<AlarmSchedule>>,
    light_status: Arc<AtomicBool>,
}

impl CronAlarm {
    /// create a looper and spawn into it's own async thread
    pub async fn init(
        db: &SqlitePool,
        light_status: Arc<AtomicBool>,
        sx: Sender<ChannelMessage>,
    ) -> Arc<Mutex<AlarmSchedule>> {
        let alarm_schedule = Arc::new(Mutex::new(AlarmSchedule::new(db).await));
        let mut looper_01 = Self {
            alarm_schedule: Arc::clone(&alarm_schedule),
            light_status: Arc::clone(&light_status),
        };

        // WARNING
		// WARNING
        // This is an example of a terrible idea, will never execute anything else past this step
        // tokio::spawn({ async move { looper.start().await } }).await;

        tokio::spawn(async move { looper_01.init_loop(sx).await });

        alarm_schedule
    }

    /// loop every 1 second,check if current time & day matches alarm, and if so execute alarm illuminate
    /// is private, so that it can only be executed during the self.init() method, so that it is correctly spawned onto it's own tokio thread
    async fn init_loop(&mut self, sx: Sender<ChannelMessage>) {
        trace!("alarm looper started");

		loop {
            if !self.alarm_schedule.lock().await.alarms.is_empty() {
                let offset = self.alarm_schedule.lock().await.offset;
                let now_as_utc_offset = OffsetDateTime::now_utc().to_offset(offset);
                let current_time = Time::from_hms(
                    now_as_utc_offset.hour(),
                    now_as_utc_offset.minute(),
                    now_as_utc_offset.second(),
                )
                .unwrap();
                let current_weekday = now_as_utc_offset
                    .to_offset(offset)
                    .weekday()
                    .number_days_from_monday();
                for i in self.alarm_schedule.lock().await.alarms.iter() {
                    if i.day == current_weekday
                        && i.hour == current_time.hour()
                        && i.minute == current_time.minute()
                        && !self.light_status.load(Ordering::SeqCst)
                    {
                        trace!("sending lighton message to via internal channels");
                        sx.send(ChannelMessage::LightOn).unwrap();
                        trace!("alarm illuminate started");
                        LightControl::alarm_illuminate(Arc::clone(&self.light_status), sx.clone())
                            .await;
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
