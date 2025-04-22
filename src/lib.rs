use embedded_hal::i2c::I2c;
use linux_embedded_hal::I2cdev;
mod font;
use font::{FONT8X8,FONT16X16, FONT24X24, Font};

pub enum OLEDColorMode {
    ColorNormal = 0, // 正常模式 黑底白字
    ColorReserved,   // 反色模式 白底黑字
}

pub const HEIGHT: usize = 64; // OLED 高度
pub const WIDTH: usize = 128; // OLED 宽度
pub const COLUMN_SIZE: usize = 8; // 列大小(单位：bit)
pub const PAGE: usize = HEIGHT / COLUMN_SIZE; // 页数

static mut I2C_ADDR: u8 = 0x3C; // I2C 地址
static mut FRAME_BUFFER: [[u8; WIDTH]; PAGE] = [[0; WIDTH]; PAGE];

fn send(i2c: &mut I2cdev, data: u8) {
    unsafe {
        let _ = i2c.write(I2C_ADDR, &[0x40, data]);
    }
}

fn sendcmd(i2c: &mut I2cdev, cmd: u8) {
    unsafe {
        let _ = i2c.write(I2C_ADDR, &[0x00, cmd]);
    }
}

pub fn init(i2c: &mut I2cdev) {
    sendcmd(i2c, 0xAEu8); /*关闭显示 display off*/

    sendcmd(i2c, 0x20u8);
    sendcmd(i2c, 0x10u8);

    sendcmd(i2c, 0xB0u8);

    sendcmd(i2c, 0xC8u8);

    sendcmd(i2c, 0x00u8);
    sendcmd(i2c, 0x10u8);

    sendcmd(i2c, 0x40u8);

    sendcmd(i2c, 0x81u8);

    sendcmd(i2c, 0xDFu8);
    sendcmd(i2c, 0xA1u8);

    sendcmd(i2c, 0xA6u8);
    sendcmd(i2c, 0xA8u8);

    sendcmd(i2c, 0x3Fu8);

    sendcmd(i2c, 0xA4u8);

    sendcmd(i2c, 0xD3u8);
    sendcmd(i2c, 0x00u8);

    sendcmd(i2c, 0xD5u8);
    sendcmd(i2c, 0xF0u8);

    sendcmd(i2c, 0xD9u8);
    sendcmd(i2c, 0x22u8);

    sendcmd(i2c, 0xDAu8);
    sendcmd(i2c, 0x12u8);

    sendcmd(i2c, 0xDBu8);
    sendcmd(i2c, 0x20u8);

    sendcmd(i2c, 0x8Du8);
    sendcmd(i2c, 0x14u8);

    sendcmd(i2c, 0xAFu8); /*开启显示 display ON*/
}

pub fn show(i2c: &mut I2cdev, x: u8, y: u8, data: u8) {
    sendcmd(i2c, 0xb0 + y);
    sendcmd(i2c, 0x00 + (x & 0x0f));
    sendcmd(i2c, 0x10 + ((x & 0xf0) >> 4));
    send(i2c, data);
}

pub fn clear(i2c: &mut I2cdev) {
    for i in 0..8 {
        sendcmd(i2c, 0xb0 + i);
        sendcmd(i2c, 0x00);
        sendcmd(i2c, 0x10);
        for _j in 0..128 {
            send(i2c, 0x00);
        }
    }
}

pub struct Buffer {
    data: Vec<String>, // 使用String代替&'a str
    len: usize,
    len_max: usize,
    index: usize,
}

impl Buffer {
    pub fn new(len_max: usize) -> Self {
        Buffer {
            data: Vec::new(),
            len: 0,
            len_max: len_max,
            index: 0,
        }
    }

    pub fn push(&mut self, s: String) {
        // 确保至少有一个字符串在data中
        if self.data.is_empty() {
            self.data.push(String::new());
            self.len = 1;
        }

        for c in s.chars() {
            if c == '\n' {
                // 遇到换行符，在头部添加新行
                self.data.insert(0, String::new());
                self.len += 1;
                // 保证最大行数
                if self.len > self.len_max {
                    self.data.pop();
                    self.len -= 1;
                }
                self.index = 0; // 换行后索引重置
            } else if c == '\r' {
                // 遇到回车符，将索引重置为0
                self.index = 0;
            } else {
                // 其他字符，添加到当前行的index位置
                let current_line = &mut self.data[0];

                // 确保当前行长度足够
                if self.index >= current_line.chars().count() {
                    // 填充空格直到index
                    let fill_count = self.index - current_line.chars().count() + 1;
                    current_line.extend(std::iter::repeat(' ').take(fill_count));
                }

                // 计算字节索引
                let char_indices: Vec<usize> =
                    current_line.char_indices().map(|(i, _)| i).collect();
                if self.index < char_indices.len() {
                    let start = char_indices[self.index];
                    let end = if self.index + 1 < char_indices.len() {
                        char_indices[self.index + 1]
                    } else {
                        current_line.len()
                    };
                    current_line.replace_range(start..end, &c.to_string());
                } else {
                    // 追加
                    current_line.push(c);
                }

                // 索引加1
                self.index += 1;
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        if index < self.len {
            Some(&self.data[index])
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.len = 0;
        self.index = 0;
    }
}

pub fn display_buffer(buffer: &Buffer, x: u8, y: u8, height: u8, i2c: &mut I2cdev) {
    newframe();
    for (i, s) in buffer.data.iter().enumerate() {
        print_string(x, y + (i as u8) * height, height, s);
    }
    showframe(i2c);
}

pub fn newframe() {
    unsafe {
        FRAME_BUFFER = [[0; WIDTH]; PAGE];
    }
}

pub fn showframe(i2c: &mut I2cdev) {
    unsafe {
        for i in 0..PAGE {
            sendcmd(i2c, 0xb0 + i as u8);
            sendcmd(i2c, 0x00);
            sendcmd(i2c, 0x10);
            for j in 0..WIDTH {
                send(i2c, FRAME_BUFFER[i as usize][j as usize]);
            }
        }
    }
}

pub fn setpixel(x: u8, y: u8, color: bool) {
    let page = y / 8;
    let page_offset = y % 8;
    unsafe {
        if color {
            if x < 128 && y < 64 {
                FRAME_BUFFER[page as usize][x as usize] |= 0x01 << page_offset;
            }
        } else {
            if x < 128 && y < 64 {
                FRAME_BUFFER[page as usize][x as usize] &= !(0x01 << page_offset);
            }
        }
    }
}

// todo 将y变成像素点
pub fn print_char(x: u8, y: u8, font: &Font, ch: char) -> Option<(u8, u8)> {
    //返回值是字符宽度
    // let font = match height {
    //     8 => FONT8X8,
    //     24 => FONT24X24,
    //     _ => {
    //         println!("Unsupported font height");
    //         FONT8X8
    //     }
    // };

    let page = y / 8;
    let page_offset = y % 8;
    let char_bytes: Option<Vec<Vec<u8>>> = font.get_char(ch);
    unsafe {
        if let Some(char_matrix) = char_bytes {
            for (i, row) in char_matrix.iter().enumerate() {
                for (j, &byte) in row.iter().enumerate() {
                    match page_offset {
                        0 => {
                            if (x as usize + j) < WIDTH && (page as usize + i) < PAGE {
                                FRAME_BUFFER[page as usize + i][x as usize + j] = byte;
                            }
                        }
                        _ => {
                            if (x as usize + j) < WIDTH && (page as usize + i) < PAGE {
                                FRAME_BUFFER[page as usize + i][x as usize + j] =
                                    byte << page_offset;
                            }
                            if (x as usize + j) < WIDTH && (page as usize + i + 1) < PAGE {
                                FRAME_BUFFER[page as usize + i + 1][x as usize + j] =
                                    byte >> (COLUMN_SIZE as u8 - page_offset);
                            }
                        }
                    }
                }
            }
        }
    }
    match font.get_char_width(ch) {
        Some(w) => {
            return Some((w, font.get_font_height()));
        }
        None => {
            println!("Error: Unsupported character :{}", ch);
            return None;
        }
    }
}

pub fn print_string(x: u8, y: u8, height: u8, str: &str) {
    // unimplemented!()
    let font = match height {
        8 => FONT8X8,
        16 => FONT16X16,
        24 => FONT24X24,
        _ => {
            println!("Unsupported font height");
            FONT8X8
        }
    };

    let mut column = x;
    let mut row = y;
    // let mut page = y / 8;
    // let page_offset = y % 8;

    for ch in str.chars() {
        match print_char(column, row, &font, ch) {
            Some((w, h)) => {
                column += w;
                if column >= WIDTH as u8 {
                    column = 0;
                    row += h;
                }
            }
            None => {
                println!("Error: Unsupported character :{}", ch);
                return;
            }
        }
    }
}

pub fn set_color_mode(i2c: &mut I2cdev, mode: OLEDColorMode) {
    match mode {
        OLEDColorMode::ColorNormal => {
            sendcmd(i2c, 0xA6u8);
        }
        OLEDColorMode::ColorReserved => {
            sendcmd(i2c, 0xA7u8);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

// #[cfg(test)]
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let mut i2c = I2cdev::new("/dev/i2c-1")?;

//     oled::init(&mut i2c);

//     let mut last_time_str = String::new();
//     let mode = OLEDColorMode::ColorNormal;
//     oled::set_color_mode(&mut i2c, mode);

//     let mut row = 16;
//     let mut col = 16;

//     loop {
//         let now = Local::now();
//         let time_str = now.format("%H:%M:%S").to_string();

//         // 只在时间发生变化时才更新显示
//         if time_str != last_time_str {
//             oled::print_string( col,row, 24, &time_str);

//             oled::showframe(&mut i2c);
//             oled::newframe();

//             last_time_str = time_str;
//         }

//         thread::sleep(Duration::from_millis(200));
//     }
// }
