use chrono::Local;
use linux_embedded_hal::I2cdev;
use std::thread;
use std::time::Duration;

use crate::oled::OLEDColorMode;
mod oled;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut i2c = I2cdev::new("/dev/i2c-1")?;

    oled::init(&mut i2c);

    let mut last_time_str = String::new();
    let mode = OLEDColorMode::ColorNormal;
    oled::set_color_mode(&mut i2c, mode);

    loop {
        let now = Local::now();
        let time_str = now.format("%H:%M:%S").to_string();

        // 只在时间发生变化时才更新显示
        if time_str != last_time_str {
            // let start = Instant::now();

            oled::newframe();
            for (i, ch) in time_str.chars().enumerate() {
                oled::print_char((i * 12 + 16) as u8, 2, ch);
            }
            oled::showframe(&mut i2c);

            last_time_str = time_str;
        }

        thread::sleep(Duration::from_millis(100));
    }
}
