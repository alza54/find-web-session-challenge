mod utils;

use core::fmt;
use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;
use image::Pixel;
use utils::{bits_to_u8, bits_to_u32};

#[wasm_bindgen]
extern "C" {
  fn alert(s: &str);
}

struct EncodeConfig {
  ignore_alpha: bool,
  ignore_white_pixels: bool,
  ignore_black_pixels: bool,
  debug: bool,
  no_panic: bool
}

enum Channel {
  Red,
  Green,
  Blue,
  Alpha
}

impl Channel {
  fn new(i: usize) -> Channel {
    match i {
      0 => Channel::Red,
      1 => Channel::Green,
      2 => Channel::Blue,
      3 => Channel::Alpha,
      _ => panic!("Invalid channel index")
    }
  }

  fn name(&self) -> &str {
    match self {
      Channel::Red => "Red",
      Channel::Green => "Green",
      Channel::Blue => "Blue",
      Channel::Alpha => "Alpha"
    }
  }
}

#[repr(u8)]
enum CharacterEncoding {
  ASCII = 0x7,
  UTF8 = 0x8,
  UTF16 = 0x10,
  UTF32 = 0x20
}

impl CharacterEncoding {
  fn new(input: u8) -> Self {
    match input {
      0x7 => CharacterEncoding::ASCII,
      0x8 => CharacterEncoding::UTF8,
      0x10 => CharacterEncoding::UTF16,
      0x20 => CharacterEncoding::UTF32,
      _ => panic!("Invalid character encoding size")
    }
  }

  fn to_bit_value(&self) -> u8 {
    match self {
      CharacterEncoding::ASCII => 0x7,
      CharacterEncoding::UTF8 => 0x8,
      CharacterEncoding::UTF16 => 0x10,
      CharacterEncoding::UTF32 => 0x20
    }
  }
}

impl fmt::Display for CharacterEncoding {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CharacterEncoding::ASCII => write!(f, "ASCII"),
      CharacterEncoding::UTF8 => write!(f, "UTF8"),
      CharacterEncoding::UTF16 => write!(f, "UTF16"),
      CharacterEncoding::UTF32 => write!(f, "UTF32")
    }
  }
}

#[wasm_bindgen]
pub fn decode_image (data: Uint8Array) -> String {
  let image_bytes = data.to_vec();
  if let Ok(image) = image::load_from_memory(&image_bytes) {
    let mut bit_iter = Vec::new();

    let mut character_encoding: Option<CharacterEncoding> = None;
    let mut message_length: Option<u32> = None;

    let config = EncodeConfig {
      ignore_alpha: true,
      ignore_white_pixels: true,
      ignore_black_pixels: false,
      debug: true,
      no_panic: true
    };

    let img = image.to_rgba8();

    for pixel in img.pixels() {
      let is_white: bool = pixel.channels()[0..3] == [255, 255, 255];
      let is_black: bool = pixel.channels()[0..3] == [0, 0, 0];

      if config.ignore_white_pixels && is_white {
        continue;
      }

      if config.ignore_black_pixels && is_black {
        continue;
      }

      let channel_range = if config.ignore_alpha { 0..3 } else { 0..4 };
      for i in channel_range {
        let bit: u8 = pixel.0[i] & 0x01; // Isolate the least significant bit

        if config.debug {
          println!("  Channel({}) [{} -> {}]. Bit \"{}\"", Channel::new(i).name(), pixel.0[i], bit, bit);
        }

        bit_iter.push(bit);

        if bit_iter.len() == 8 + 32 {
          character_encoding = Some(
            CharacterEncoding::new(bits_to_u8(bit_iter[0..8].to_vec(), config.no_panic))
          );

          message_length = Some(
            bits_to_u32(bit_iter[8..40].to_vec(), config.no_panic)
          );
        } else if message_length.is_some() && bit_iter.len() == message_length.unwrap() as usize + 8 + 32 {
          let encoding_enum_value = character_encoding.as_ref();
          let encoding = encoding_enum_value.unwrap().to_bit_value();

          if config.debug {
            println!("Message Length: {:?}", message_length.unwrap());
            println!("Encoding: {:?}", encoding_enum_value.unwrap().to_string());
          }

          if encoding == CharacterEncoding::ASCII.to_bit_value() {
            let message_vec: Vec<char> = bit_iter[40..]
              .chunks(8)
              .map(|chunk| {
                let code_point = chunk.iter().fold(0u8, |acc, &bit| (acc << 1) | bit);
                code_point as char
              })
              .collect();

            let message: String = message_vec.into_iter().collect();

            return message;
          } else if encoding == CharacterEncoding::UTF8.to_bit_value() {
            let message_vec: Vec<u8> = bit_iter[40..]
              .chunks(8)
              .map(|chunk| bits_to_u8(chunk.to_vec(), config.no_panic))
              .collect();

            let message: String = String::from_utf8(message_vec)
              .unwrap_or_else(|_| "Invalid UTF-8".to_string());

            return message;
          } else if encoding == CharacterEncoding::UTF16.to_bit_value() {
            let message_vec: Vec<u16> = bit_iter[40..]
              .chunks(16)
              .map(|chunk| {
                  chunk.iter().fold(0u16, |acc, &bit| (acc << 1) | (bit as u16))
              })
              .collect();

            let message: String = String::from_utf16(&message_vec)
              .unwrap_or_else(|_| "Invalid UTF-16".to_string());

            return message;
          } else if encoding == CharacterEncoding::UTF32.to_bit_value() {
            let message_vec: Vec<char> = bit_iter[40..]
              .chunks(32)
              .map(|chunk| {
                  let mut code_point = 0u32;
                  for &bit in chunk {
                      code_point <<= 1; // Shift left by 1 to make space for the next bit
                      code_point |= if bit == 1 { 1 } else { 0 }; // Set the last bit based on the current char
                  }
                  std::char::from_u32(code_point).unwrap_or('?') // Convert the code point to a char, using '?' for invalid code points
              })
              .collect();

            let message: String = message_vec.into_iter().collect();

            return message;
          }
        }
      }
    }

    return String::from("");
  } else {
    return String::from("error");
  }
}

#[wasm_bindgen]
pub fn greet(name: &str) {
  alert(&format!("Hello, {}!", name));
}
