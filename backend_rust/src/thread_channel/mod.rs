use futures_util::{lock::Mutex, stream::SplitSink, SinkExt};
use std::{sync::Arc, time::Instant};
use sqlx::SqlitePool;
use tokio::{net::TcpStream, sync::broadcast::Receiver};
use tokio_tungstenite::{self, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, trace};

use crate::{
    env::AppEnv,
    sql::ModelAlarm,
    sysinfo::SysInfo,
    ws::{PiStatus, Response, StructuredResponse},
};

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    LightOn,
    LightOff,
    Close,
    Status,
    Response(Response, Option<bool>),
}

pub struct ThreadChannel {
    rx: Receiver<ChannelMessage>,
    writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    db: Arc<SqlitePool>,
    app_envs: AppEnv,
    connected_for: Instant,
}

impl ThreadChannel {
    pub async fn init(
        rx: Receiver<ChannelMessage>,
        writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        db: Arc<SqlitePool>,
        app_envs: AppEnv,
    ) {
        Self {
            rx,
            writer,
            db,
            app_envs,
            connected_for: Instant::now(),
        }
        .start()
        .await;
    }

    /// Handle all the internal messages, sent from ws socker handler, and the alarm scheduler
    async fn start(&mut self) {
        while let Ok(message) = self.rx.recv().await {
            match message {
                ChannelMessage::LightOn => {
                    let response = Response::LedStatus { status: true };
                    Self::send_ws_response(self, response, None).await;
                }
                ChannelMessage::LightOff => {
                    let response = Response::LedStatus { status: false };
                    Self::send_ws_response(self, response, None).await;
                }
                ChannelMessage::Status => Self::get_status(self).await,
                ChannelMessage::Close => self.writer.lock().await.close().await.unwrap(),
                ChannelMessage::Response(response, cache) => {
                    Self::send_ws_response(self, response, cache).await
                }
            }
        }
    }

    /// Return basic info, time, alarms, ip address, uptime etc
    async fn get_status(&mut self) {
        let info = SysInfo::new(&self.db, &self.app_envs).await.unwrap();
        let alarms = ModelAlarm::get_all(&self.db).await.unwrap();
        let info = PiStatus::new(info, alarms, self.connected_for.elapsed().as_secs());
        let response = Response::Status(info);
        debug!("send status");
        Self::send_ws_response(self, response, Some(true)).await;
    }

    /// Send a message to the socket
    async fn send_ws_response(&mut self, response: Response, cache: Option<bool>) {
        match self
            .writer
            .lock()
            .await
            .send(StructuredResponse::data(response, cache))
            .await
        {
            Ok(_) => trace!("Message sent"),
            Err(e) => error!("toggle_list::SEND-ERROR::{:?}", e),
        }
    }
}
