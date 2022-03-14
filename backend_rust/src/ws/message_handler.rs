use anyhow::Result;
use sqlx::SqlitePool;
use std::process;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use time::OffsetDateTime;
use time_tz::{timezones, Offset, TimeZone};
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex as TokioMutex;
use tokio_tungstenite::{self, tungstenite::Message};
use tracing::{debug, error, info};

use crate::alarm_schedule::AlarmSchedule;
use crate::ChannelMessage;
use crate::{
    env::AppEnv,
    light::LightControl,
    sql::{ModelAlarm, ModelTimezone},
    ws::{
        message_incoming::{to_struct, MessageValues, ParsedMessage},
        message_outgoing::Response,
    },
};

#[derive(Debug)]
enum MessageType {
    Ping,
    Pong,
    Text(MessageHandler),
    Binary,
    Close(MessageHandler),
    Empty,
}

#[derive(Debug)]
struct MessageHandler {
    alarm_scheduler: Arc<TokioMutex<AlarmSchedule>>,
    app_envs: AppEnv,
    db: Arc<SqlitePool>,
    light_status: Arc<AtomicBool>,
    message: String,
    sender: Sender<ChannelMessage>,
}

impl<'a>
    From<(
        Arc<TokioMutex<AlarmSchedule>>,
        AppEnv,
        Arc<SqlitePool>,
        Arc<AtomicBool>,
        &Message,
        Sender<ChannelMessage>,
    )> for MessageType
{
    fn from(
        input: (
            Arc<TokioMutex<AlarmSchedule>>,
            AppEnv,
            Arc<SqlitePool>,
            Arc<AtomicBool>,
            &Message,
            Sender<ChannelMessage>,
        ),
    ) -> Self {
        match input.4 {
            m if m.is_binary() => MessageType::Binary,
            m if m.is_close() => MessageType::Close(MessageHandler {
                alarm_scheduler: input.0,
                app_envs: input.1,
                db: input.2,
                light_status: input.3,
                message: input.4.to_string(),
                sender: input.5,
            }),
            m if m.is_ping() => MessageType::Ping,
            m if m.is_pong() => MessageType::Pong,
            m if m.is_text() => MessageType::Text(MessageHandler {
                alarm_scheduler: input.0,
                app_envs: input.1,
                db: input.2,
                light_status: input.3,
                message: input.4.to_string(),
                sender: input.5,
            }),

            _ => MessageType::Empty,
        }
    }
}

impl MessageHandler {
    
	/// Handle text message, in this program they will all be json text
    async fn on_text(&mut self) {
        let start = std::time::Instant::now();
        if let Some(data) = to_struct(&self.message) {
            match data {
                MessageValues::Invalid(error) => error!("{:?}", error),
                MessageValues::Valid(data) => match data {
                    ParsedMessage::DeleteAll => Self::delete_all(self).await,
                    ParsedMessage::DeleteOne(id) => Self::delete_one(self, id.alarm_id).await,
                    ParsedMessage::LedStatus => Self::led_status(self).await,
                    ParsedMessage::Restart => Self::restart(),
                    ParsedMessage::TimeZone(timezone) => Self::time_zone(self, timezone.zone).await,
                    ParsedMessage::AddAlarm(data) => {
                        Self::add_alarm(self, data.days, data.hour, data.minute).await
                    }
                    ParsedMessage::Light { status } => Self::toggle_light(self, status).await,
                    ParsedMessage::Status => {
                        self.sender.send(ChannelMessage::Status).unwrap();
                    }
                },
            };
            let done = start.elapsed();
            let message_handler = format!("{}ms, {}μs", done.as_millis(), done.as_micros(),);
            debug!(%message_handler);
        }
    }

    /// Add a new alarm to database, and update alarm_schedule alarm vector
    async fn add_alarm(&mut self, day: Vec<u8>, hour: u8, minute: u8) {
        let mut handles = vec![];
        for i in day {
            handles.push(ModelAlarm::add(&self.db, (i, hour, minute)));
        }
        for handle in handles {
            match handle.await {
                Ok(_) => (),
                Err(e) => debug!(%e),
            }
        }
        self.alarm_scheduler
            .lock()
            .await
            .refresh_alarms(&self.db)
            .await;
        self.sender.send(ChannelMessage::Status).unwrap();
    }

    /// Handle websocket close event
    async fn close(self) {
        let connection_closed = format!(
            "{}",
            OffsetDateTime::now_utc().to_offset(self.app_envs.utc_offset)
        );
        info!(%connection_closed);
        self.sender.send(ChannelMessage::Close).unwrap();
    }

    /// Delete all alarms in database, and update alarm_schedule alarm vector
    /// If the alarm sequence has started, and you delete all alarms, the light is still on
    /// Would need to set the light status to false, but that could also set the light off if on not during an alarm sequence
    async fn delete_all(&mut self) {
        // This isn't working
        ModelAlarm::delete_all(&self.db).await.unwrap();
        self.alarm_scheduler
            .lock()
            .await
            .refresh_alarms(&self.db)
            .await;
        self.sender.send(ChannelMessage::Status).unwrap();
    }

    /// Delete from database a given alarm, by id, and also remove from alarm_schedule alarm vector
    async fn delete_one(&mut self, id: i64) {
        ModelAlarm::delete(&self.db, id).await.unwrap();
        self.alarm_scheduler.lock().await.remove_alarm(id);
        self.sender.send(ChannelMessage::Status).unwrap();
    }

    /// This also needs to be send from alarm sequencer
    /// return true if led light is currently turned on
    async fn led_status(&mut self) {
        let status = self.light_status.load(Ordering::SeqCst);
        let response = Response::LedStatus { status };
        self.sender
            .send(ChannelMessage::Response(response, None))
            .unwrap();
    }

    fn restart() {
        process::exit(0);
    }

    /// Change the timezone in database to new given database,
    /// also update timezone in alarm scheduler
    async fn time_zone(&mut self, zone: String) {
        let offset = timezones::get_by_name(&zone)
            .unwrap()
            .get_offset_utc(&time::OffsetDateTime::now_utc())
            .to_utc();
        ModelTimezone::update(&self.db, &zone, offset)
            .await
            .unwrap();
        self.alarm_scheduler
            .lock()
            .await
            .refresh_timezone(&self.db)
            .await;
        self.sender.send(ChannelMessage::Status).unwrap();
    }

    /// turn light either on or off
    async fn toggle_light(&mut self, new_status: bool) {
        if new_status && !self.light_status.load(Ordering::SeqCst) {
            self.light_status.store(true, Ordering::SeqCst);
            let response = Response::LedStatus { status: new_status };
            self.sender
			.send(ChannelMessage::Response(response, None))
			.unwrap();
			// Could actually spawn this?
			LightControl::turn_on(Arc::clone(&self.light_status)).await;
        } else if !new_status {
            self.light_status.store(false, Ordering::SeqCst);
            Self::led_status(self).await;
        }
    }
}

// do this one step up?
pub async fn message_handler(
    alarm_scheduler: Arc<TokioMutex<AlarmSchedule>>,
    args: AppEnv,
    db: Arc<SqlitePool>,
    light_status: Arc<AtomicBool>,
    message: Result<Message, tokio_tungstenite::tungstenite::Error>,
    sender: Sender<ChannelMessage>,
) -> Result<()> {
    match message {
        Err(_) => error!("spawn_message::unable to handle ws message"),
        Ok(m) => {
            let msg = MessageType::from((alarm_scheduler, args, db, light_status, &m, sender));
            match msg {
                MessageType::Close(value) => value.close().await,
                MessageType::Text(mut value) => value.on_text().await,
                MessageType::Empty => (),
                _ => (),
            };
        }
    }
    Ok(())
}
