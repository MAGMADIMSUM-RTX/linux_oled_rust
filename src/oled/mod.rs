#![allow(dead_code)]

use embedded_hal::i2c::I2c;
use linux_embedded_hal::I2cdev;
mod font;
use font::{FONT8X8, FONT24X24, Font};

pub enum OLEDColorMode {
    ColorNormal = 0, // 正常模式 黑底白字
    ColorReserved,   // 反色模式 白底黑字
}

const HEIGHT: usize = 64; // OLED 高度
const WIDTH: usize = 128; // OLED 宽度
const COLUMN_SIZE: usize = 8; // 列大小(单位：bit)
const PAGE: usize = HEIGHT / COLUMN_SIZE ; // 页数

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

pub fn newframe() {
    unsafe {
        FRAME_BUFFER = [[0; WIDTH];PAGE];
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
pub fn print_char(x: u8, y: u8, font: &Font, ch: char)-> Option<u8> { //返回值是字符宽度
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
                    // if (x as usize + j) < WIDTH && (page as usize + i) < PAGE {
                        FRAME_BUFFER[page as usize + i][x as usize + j] |= 
                            byte << page_offset;
                    // }
                    // if (x as usize + j) < WIDTH && (page as usize + i + 1) < PAGE {
                        FRAME_BUFFER[page as usize + i + 1][x as usize + j] |=
                            byte >> (COLUMN_SIZE as u8 - page_offset);
                    // }
                }
            }
        }
    }
    font.get_char_width(ch)
}

pub fn print_string(x: u8, y: u8, height: u8, str: &str) {
    // unimplemented!()
    let font = match height {
        8 => FONT8X8,
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
        match print_char(column, row, &font, ch){
            Some(w) => {
                column += w;
            }
            None => {
                println!("Error: Unsupported character :{}", ch);
                return;
            }
        }
        
        if column >= WIDTH as u8 {
            column = 0;
            row += font.get_font_height();
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

// fn set_byte_fine(page:u8,column:u8,data:u8,start:u8,end:u8,color:OLEDColorMode){
//     let mut temp:u8 = 0;
//     // if page >= 8 {
//     //     return; // Add a condition to handle invalid page values
//     // }
//     if color == OLEDColorMode::ColorReserved {
//         // temp = data;
//         data = !data;
//     }

//     temp = data| 0xff<<(end+1)|0xff>>(8-start);
//     FRAME_BUFFER[page as usize][column as usize] &= temp;

// }
