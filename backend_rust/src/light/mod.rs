use crate::thread_channel::ChannelMessage;
use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::broadcast::Sender;
use tokio::time::{sleep, Instant};
use tracing::debug;

#[cfg(not(target_arch = "x86_64"))]
use blinkt::Blinkt;

pub struct LightControl;

#[derive(Debug)]
enum LimitMinutes {
    Five,
    Fifteen,
}

impl fmt::Display for LimitMinutes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = match self {
            Self::Fifteen => "15",
            Self::Five => "5",
        };
        write!(f, "{}", x)
    }
}

impl LightControl {
    /// whilst light_status is true, set all lights to on
    /// use light_limit to make sure led is only on for 5 minutes max
    #[cfg(not(target_arch = "x86_64"))]
    pub async fn turn_on(light_status: Arc<AtomicBool>) {
        let mut led_strip = Blinkt::new().unwrap();
        led_strip.clear();
        led_strip.set_all_pixels(255, 200, 15);
        led_strip.set_all_pixels_brightness(1.0);
        let start = Instant::now();
        while light_status.load(Ordering::SeqCst) {
            Self::light_limit(start, &LimitMinutes::Five);
            led_strip.show().unwrap();
            sleep(Duration::from_millis(250)).await;
        }
    }

    /// whilst light_status is true, set all lights to on
    /// use light_limit to make sure led is only on for 5 minutes max
    #[cfg(target_arch = "x86_64")]
    pub async fn turn_on(light_status: Arc<AtomicBool>) {
        let start = Instant::now();
        while light_status.load(Ordering::SeqCst) {
            if Self::light_limit(start, &LimitMinutes::Five) {
                light_status.store(false, Ordering::SeqCst);
            }
            debug!("light on");
            sleep(Duration::from_millis(250)).await;
        }

        debug!("turn_on finished");
    }

    /// Turn light on in steps of 10% brightness, 5 minutes for each step, except last step which stays on for 15 minutes
    /// Will stop if the light_status atomic bool is changed elsewhere during the execution
    #[cfg(not(target_arch = "x86_64"))]
    pub async fn alarm_illuminate(light_status: Arc<AtomicBool>, sx: Sender<ChannelMessage>) {
        light_status.store(true, Ordering::SeqCst);
        tokio::spawn(async move {
            let mut brightness = 1.0;
            let mut led_strip = Blinkt::new().unwrap();
            let mut step = 0;
            let mut start = Instant::now();
            led_strip.clear();
            led_strip.set_all_pixels(255, 200, 15);
            led_strip.set_all_pixels_brightness(brightness / 10.0);
            while light_status.load(Ordering::SeqCst) {
                led_strip.show().unwrap();
                let limit = if step < 9 {
                    LimitMinutes::Five
                } else {
                    LimitMinutes::Fifteen
                };
                if Self::light_limit(start, &limit) {
                    let status = format!("step: {}, brightness: {}", step, brightness / 10.0);
                    debug!(%status);
                    start = Instant::now();
                    step += 1;
                    brightness += 1.0;
                    led_strip.set_all_pixels_brightness(brightness / 10.0);
                    if let LimitMinutes::Fifteen = limit {
                        light_status.store(false, Ordering::SeqCst);
                        led_strip.clear();
                    };
                };
                sleep(Duration::from_millis(250)).await;
            }
            sx.send(ChannelMessage::LightOff).unwrap();
        });
    }

    /// Turn light on in steps of 10% brightness, 5 minutes for each step, except last step which stays on for 15 minutes
    /// Will stop if the light_status atomic bool is changed elsewhere during the execution
    /// On finish, send an message via thread_channel, in order to send a ws message to any connected clients
    #[cfg(target_arch = "x86_64")]
    pub async fn alarm_illuminate(light_status: Arc<AtomicBool>, sx: Sender<ChannelMessage>) {
        light_status.store(true, Ordering::SeqCst);
        tokio::spawn(async move {
            let mut brightness = 1.0;
            let mut step = 0;
            let mut start = Instant::now();
            while light_status.load(Ordering::SeqCst) {
                let limit = if step < 9 {
                    LimitMinutes::Five
                } else {
                    LimitMinutes::Fifteen
                };
                if Self::light_limit(start, &limit) {
                    let status = format!("step: {}, brightness: {}", step, brightness / 10.0);
                    debug!(%status);
                    step += 1;
                    brightness += 1.0;
                    start = Instant::now();
                    if let LimitMinutes::Fifteen = limit {
                        light_status.store(false, Ordering::SeqCst);
                    };
                };
                sleep(Duration::from_millis(250)).await;
            }
            sx.send(ChannelMessage::LightOff).unwrap();
            debug!("alarm_illuminate finished");
        });
    }

    /// Return true in start time longer than given limit
    #[cfg(not(target_arch = "x86_64"))]
    fn light_limit(start: Instant, limit: &LimitMinutes) -> bool {
        let duration = match limit {
            LimitMinutes::Five => 60 * 5,
            LimitMinutes::Fifteen => 60 * 15,
        };
        start.elapsed().as_secs() > duration
    }
    /// Return true in start time longer than given limit
    #[cfg(target_arch = "x86_64")]
    fn light_limit(start: Instant, limit: &LimitMinutes) -> bool {
        let duration = match limit {
            LimitMinutes::Five => 5,
            LimitMinutes::Fifteen => 15,
        };
        start.elapsed().as_secs() > duration
    }

    /// Show color on single led light for 50 ms
    #[cfg(target_arch = "x86_64")]
    async fn show_rainbow(pixel: usize, color: (u8, u8, u8)) {
        use tracing::trace;
        let pixel = format!("{pixel} - ({},{},{})", color.0, color.1, color.2);
        trace!(%pixel);
    }

    /// Show color on single led light for 50 ms
    #[cfg(not(target_arch = "x86_64"))]
    async fn show_rainbow(pixel: usize, color: (u8, u8, u8)) {
        let brightness = 1.0;
        let mut led_strip = Blinkt::new().unwrap();
        led_strip.clear();
        led_strip.set_pixel_brightness(pixel, brightness);
        led_strip.set_pixel(pixel, color.0, color.1, color.2);
        led_strip.show().unwrap();
        sleep(Duration::from_millis(50)).await;
    }

    /// Loop over array of rgb colors, send each to the led strip one at a time
    pub async fn rainbow(x: Arc<AtomicBool>) {
        if !x.load(Ordering::SeqCst) {
            let rainbow_colors = [
                (255, 0, 0),
                (255, 127, 0),
                (255, 255, 0),
                (0, 255, 0),
                (0, 0, 255),
                (39, 0, 51),
                (139, 0, 255),
                (255, 255, 255),
            ];

            for (pixel, color) in rainbow_colors.into_iter().enumerate() {
                Self::show_rainbow(pixel, color).await;
            }

            for (pixel, color) in rainbow_colors.into_iter().rev().enumerate() {
                Self::show_rainbow(rainbow_colors.len() - 1 - pixel, color).await;
            }
        }
    }
}
