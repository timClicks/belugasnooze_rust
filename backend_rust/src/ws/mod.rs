mod connect;
mod connection_details;
mod incoming_serializer;
mod message_handler;
mod message_incoming;
mod message_outgoing;

use connect::ws_upgrade;
use connection_details::ConnectionDetails;
use futures_util::{lock::Mutex, stream::SplitStream, StreamExt};
use message_handler::message_handler;
use sqlx::SqlitePool;
use std::sync::{atomic::AtomicBool, Arc};
use time::{OffsetDateTime, UtcOffset};
use tokio::{
    net::TcpStream,
    sync::{broadcast::Sender, Mutex as TokioMutex},
};
use tokio_tungstenite::{self, MaybeTlsStream, WebSocketStream};
use tracing::error;

use crate::{
    alarm_schedule::AlarmSchedule,
    env::AppEnv,
    light::LightControl,
    sql::ModelTimezone,
    thread_channel::{ChannelMessage, ThreadChannel},
};

pub use message_outgoing::{PiStatus, Response};

pub use self::message_outgoing::StructuredResponse;
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

//handle each incoming ws message
async fn incoming_ws_message(
    alarm_scheduler: Arc<TokioMutex<AlarmSchedule>>,
    args: &AppEnv,
    db: Arc<SqlitePool>,
    light_status: Arc<AtomicBool>,
    mut reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    rx: Sender<ChannelMessage>,
) {
    while let Some(message) = reader.next().await {
        let args = args.clone();
        let light_status = Arc::clone(&light_status);
        let db = Arc::clone(&db);
        let alarm_schedule = Arc::clone(&alarm_scheduler);
        let sender = rx.clone();

        tokio::spawn(async move {
            message_handler(alarm_schedule, args, db, light_status, message, sender).await
        });
    }
}

/// try to open WS connection, and spawn a ThreadChannel message handler
pub async fn open_connection(
    cron_alarm: Arc<TokioMutex<AlarmSchedule>>,
    app_envs: AppEnv,
    db: Arc<SqlitePool>,
    light_status: Arc<AtomicBool>,
    tx: Sender<ChannelMessage>,
) {
    let mut connection_details = ConnectionDetails::new();

	loop {
        connection_details.reconnect_delay().await;

        let atx = tx.subscribe();

        match ws_upgrade(&app_envs).await {
            Ok(socket) => {
                connection_details.valid_connect();

                let (writer, reader) = socket.split();
                let message_writer = Arc::new(Mutex::new(writer));

                let db_timezone = ModelTimezone::get(&db).await.unwrap();

                let allowable = 7u8..=10;
                if allowable.contains(
                    &OffsetDateTime::now_utc()
                        .to_offset(
                            UtcOffset::from_hms(
                                db_timezone.offset_hour,
                                db_timezone.offset_minute,
                                db_timezone.offset_second,
                            )
                            .unwrap(),
                        )
                        .hour(),
                ) {
                    LightControl::rainbow(Arc::clone(&light_status)).await;
                }


                let alarm_writer = Arc::clone(&message_writer);
                let light_db = Arc::clone(&db);
                let light_app_envs = app_envs.clone();

                tokio::spawn(async move {
                    ThreadChannel::init(atx, alarm_writer, light_db, light_app_envs).await
                });

                // Send a message to all connected clients
                tx.clone().send(ChannelMessage::Status).unwrap();

                incoming_ws_message(
                    Arc::clone(&cron_alarm),
                    &app_envs,
                    Arc::clone(&db),
                    Arc::clone(&light_status),
                    reader,
                    tx.clone(),
                )
                .await;
                
            }
            Err(e) => {
                let connect_error = format!("{}", e);
                error!(%connect_error);
                connection_details.fail_connect();
            }
        }
    }
}
