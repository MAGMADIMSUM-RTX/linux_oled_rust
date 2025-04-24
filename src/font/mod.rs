#![allow(unused)]
// #[allow(dead_code)]

use std::ptr;
mod font_bytes;
use font_bytes::{ASCII_8X6, ASCII_16X8, ASCII_24X12, NONE_CHAR};
struct ASCIIFont {
    h: u8,
    w: u8,
    chars: &'static [u8],
}

pub struct Font {
    h: u8,
    w: u8,
    chars: &'static [char],
    ascii: ASCIIFont,
}

#[allow(dead_code)]
pub struct Buffer<'a> {
    pub data: Vec<&'a str>,
    pub len: usize,
}

impl Font {
    pub fn get_char(&self, ch: char) -> Option<Vec<Vec<u8>>> {
        // if !ch.is_ascii() || ch < ' ' || ch > 'z' {
        //     return None;
        // }
        let (char_data, rows, cols) = if ch.is_ascii() && ch < ' ' {
            return None;
        } else if ch.is_ascii() {
            let index = (ch as u8 - ' ' as u8) as usize;
            let len: usize = ((self.ascii.h / 8) * self.ascii.w) as usize;
            let char_data = &self.ascii.chars[index * len..(index + 1) * len];
            // 确定行数（高度除以8，因为每个字节代表8个垂直像素）
            let rows = (self.ascii.h / 8) as usize;
            let cols = self.ascii.w as usize;
            (char_data, rows, cols)
        } else {
            // 处理非ASCII字符
            // if let Some(index) = self.chars.iter().position(|&c| c == ch) {
            //     let len = ((self.h / 8) * self.w) as usize;
            //     let char_data = self.chars[index * len..(index + 1) * len]
            //         .iter()
            //         .map(|&c| c as u8)
            //         .collect::<Vec<u8>>()
            //         .as_slice();
            //     let rows = (self.h / 8) as usize;
            //     let cols = self.w as usize;
            //     (char_data, rows, cols)
            println!("Unsupported character: {}", ch);
            // } else {
            return None;
            // }
        };

        // 创建二维数组
        let mut result: Vec<Vec<u8>> = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut row_data: Vec<u8> = Vec::with_capacity(cols);
            for col in 0..cols {
                row_data.push(char_data[row * cols + col]);
            }
            result.push(row_data);
        }
        Some(result)
    }

    pub fn get_char_width(&self, ch: char) -> Option<u8> {
        if ch.is_ascii() {
            if ch < ' ' {
                return None;
            }
            return Some(self.ascii.w);
        } else {
            if self.chars.iter().any(|&c| c == ch) {
                return Some(self.w);
            }
            return None;
        }
    }

    pub fn get_font_height(&self) -> u8 {
        self.h
    }
}

struct Image {
    w: u8,
    h: u8,
    data: &'static [u8],
}

const AFONT8X6: ASCIIFont = ASCIIFont {
    h: 8,
    w: 6,
    chars: unsafe {
        ptr::slice_from_raw_parts(ASCII_8X6.as_ptr() as *const u8, 92 * 6)
            .as_ref()
            .unwrap()
    },
};

pub const FONT8X8: Font = Font {
    h: 8,
    w: 8,
    chars: &NONE_CHAR,
    ascii: AFONT8X6,
};

const AFONT16X8: ASCIIFont = ASCIIFont {
    h: 16,
    w: 8,
    chars: unsafe {
        ptr::slice_from_raw_parts(ASCII_16X8.as_ptr() as *const u8, 95 * 16)
            .as_ref()
            .unwrap()
    },
};

pub const FONT16X16: Font = Font {
    h: 16,
    w: 16,
    chars: &NONE_CHAR,
    ascii: AFONT16X8,
};

const AFONT24X12: ASCIIFont = ASCIIFont {
    h: 24,
    w: 12,
    chars: unsafe {
        ptr::slice_from_raw_parts(ASCII_24X12.as_ptr() as *const u8, 95 * 36)
            .as_ref()
            .unwrap()
    },
};

pub const FONT24X24: Font = Font {
    h: 24,
    w: 24,
    chars: &NONE_CHAR,
    ascii: AFONT24X12,
};
