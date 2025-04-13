#![allow(dead_code)]

use std::string;

use embedded_hal::i2c::I2c;
use linux_embedded_hal::I2cdev;
mod font;
use font::FONT24X24;

pub enum OLEDColorMode {
    ColorNormal = 0, // 正常模式 黑底白字
    ColorReserved,   // 反色模式 白底黑字
}

static mut I2C_ADDR: u8 = 0x3C; // I2C 地址
static mut FRAME_BUFFER: [[u8; 128]; 8] = [[0; 128]; 8];

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
        FRAME_BUFFER = [[0; 128]; 8];
    }
}

pub fn showframe(i2c: &mut I2cdev) {
    unsafe {
        for i in 0..8 {
            sendcmd(i2c, 0xb0 + i);
            sendcmd(i2c, 0x00);
            sendcmd(i2c, 0x10);
            for j in 0..128 {
                send(i2c, FRAME_BUFFER[i as usize][j as usize]);
            }
        }
    }
}

pub fn setpixel(x: usize, y: usize, color: bool) {
    unsafe {
        if color {
            if x < 128 && y < 64 {
                FRAME_BUFFER[y / 8][x] |= 0x01 << (y % 8);
            }
        } else {
            if x < 128 && y < 64 {
                FRAME_BUFFER[y / 8][x] &= !(0x01 << (y % 8));
            }
        }
    }
}

// todo 将y变成像素点
pub fn print_char(x: u8, y: u8, ch: char) {
    unsafe {
        // let len: usize = FONT24X24.get_char_len();
        let char_bytes: Option<Vec<Vec<u8>>> = FONT24X24.get_char(ch);

        if let Some(char_matrix) = char_bytes {
            for (i, row) in char_matrix.iter().enumerate() {
                for (j, &byte) in row.iter().enumerate() {
                    if (x as usize + j) < 128 && (y as usize + i) < 64 {
                        FRAME_BUFFER[(y as usize + i)][x as usize + j] |= byte;
                    }
                }
            }
        }
    }
}

pub fn print_string(x: u8, y: u8, string: String) {
    unimplemented!()
    // todo!()
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
