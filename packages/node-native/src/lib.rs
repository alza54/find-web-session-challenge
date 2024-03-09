use core::fmt;
use std::path;
use neon::prelude::*;
use image::{DynamicImage, ImageBuffer, ImageError, Pixel, Rgba, RgbaImage};

pub mod encoder {
  use super::*;

  pub struct EncodeConfig {
    ignore_alpha: bool,
    ignore_white_pixels: bool,
    ignore_black_pixels: bool,
    debug: bool,
    no_panic: bool
  }

  pub enum Channel {
    Red,
    Green,
    Blue,
    Alpha
  }

  impl Channel {
    pub fn new(i: usize) -> Channel {
      match i {
        0 => Channel::Red,
        1 => Channel::Green,
        2 => Channel::Blue,
        3 => Channel::Alpha,
        _ => panic!("Invalid channel index")
      }
    }

    pub fn name(&self) -> &str {
      match self {
        Channel::Red => "Red",
        Channel::Green => "Green",
        Channel::Blue => "Blue",
        Channel::Alpha => "Alpha"
      }
    }
  }

  #[repr(u8)]
  pub enum CharacterEncoding {
    #[allow(clippy::upper_case_acronyms)]
    ASCII = 0x7,
    UTF8 = 0x8,
    UTF16 = 0x10,
    UTF32 = 0x20
  }

  impl CharacterEncoding {
    pub fn new(input: u8) -> Self {
      match input {
        0x7 => CharacterEncoding::ASCII,
        0x8 => CharacterEncoding::UTF8,
        0x10 => CharacterEncoding::UTF16,
        0x20 => CharacterEncoding::UTF32,
        _ => panic!("Invalid character encoding size")
      }
    }

    pub fn to_bit_value(&self) -> u8 {
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

  /// Calculate the total number of bits available for encoding in the image.
  pub fn calculate_capacity(image: &RgbaImage, config: &EncodeConfig) -> usize {
    let channels_num = if config.ignore_alpha { 3 } else { 4 };
    image.width() as usize * image.height() as usize * channels_num
  }

  pub fn calculate_bits_of_encoded_string(s: &str) -> (CharacterEncoding, u32) {
    let mut max_code_point: u32 = 0u32;
    for c in s.chars() {
      let code_point: u32 = c as u32;
      if code_point > max_code_point {
        max_code_point = code_point;
      }
    }

    if max_code_point <= 0x7F {
      // Can be encoded in ASCII
      (CharacterEncoding::ASCII, s.len() as u32 * 8)
    } else if max_code_point <= 0xFFFF {
      // Can be encoded in UTF-8 or UTF-16 without surrogates
      let utf8_bits: u32 = s.chars().map(|c| {
        let code_point: u32 = c as u32;
        if code_point <= 0x7F { 8 } // 1 byte
        else if code_point <= 0x7FF { 16 } // 2 bytes
        else { 24 } // 3 bytes for characters up to U+FFFF
      }).sum();

       // UTF-16 uses 2 bytes (16 bits) for characters up to U+FFFF
      let utf16_bits = s.len() as u32 * 16;

      if utf8_bits <= utf16_bits { (CharacterEncoding::UTF8, utf8_bits) }
      else { (CharacterEncoding::UTF16, utf16_bits) }
    } else {
      (CharacterEncoding::UTF32, s.chars().count() as u32 * 32)
    }
  }

  pub fn to_binary_chunks(text: &str, chunk_size: usize) -> String {
    text
      .as_bytes()
      .chunks(chunk_size)
      .map(std::str::from_utf8)
      .filter_map(Result::ok)
      .collect::<Vec<&str>>()
      .join(" ")
  }

  /// Encode the message length, encoding enum value and the message into the image.
  fn encode_image(img: &RgbaImage, message: &str, encoding_input: CharacterEncoding, config: EncodeConfig) -> RgbaImage {
    let mut encoded_img = img.clone();

    let mut bit_iter_32 = message_to_bit_iter_utf32(message);
    let mut bit_iter_8 = message_to_bit_iter(message);

    // Encode message length first
    let (encoding, message_size) = calculate_bits_of_encoded_string(&message);

    let encoding_bits = format!("{:08b}", encoding.to_bit_value());
    let size_bits = format!("{:032b}", message_size);

    let mut encoding_bit_iter = encoding_bits.chars();
    let mut size_bit_iter = size_bits.chars();

    if encoding.to_string() != encoding_input.to_string() {
      if config.no_panic {
        eprintln!("Error: The encoding of the message does not match the encoding input");
        return encoded_img;
      } else {
        panic!("Fatal Error: The encoding of the message does not match the encoding input");
      }
    }

    if config.debug {
      println!("\n[Encoder] Chosen string encoding:\n  - String value = \"{}\"\n  - Hexadecimal = 0x{:x}\n  - Binary = \"{}\"",
        encoding, encoding.to_bit_value(), to_binary_chunks(&encoding_bits, 4));
      println!("[Encoder] Calculated message size:\n  - Decimal = {}\n  - Hexadecimal = 0x{:x}\n  - Binary = \"{}\"\n",
        message_size, message_size, to_binary_chunks(&size_bits, 4));
    }

    let mut iter_chain = || (
      encoding_bit_iter.next()
        .or_else(|| size_bit_iter.next())
        .or_else(|| {
          if encoding.to_bit_value() == CharacterEncoding::UTF32.to_bit_value() {
            return bit_iter_32.next();
          }

          return bit_iter_8.next();
        })
    );

    for (x, y, pixel) in encoded_img.enumerate_pixels_mut() {
      // White pixels often correspond to background,
      // and should be left untouched if possible.
      // Usually not a problem,
      // but I want to make this example as clean as possible.
      let is_white: bool = pixel.channels()[0..3] == [255, 255, 255];
      let is_black: bool = pixel.channels()[0..3] == [0, 0, 0];

      if config.debug {
        println!("[(Pixel at ({}, {})) == {:?}] White: {} Black: {}", x, y, pixel,
          if is_white { "yes" } else { "no" }, if is_black { "yes" } else { "no" });
      }

      if (config.ignore_white_pixels && is_white) || (config.ignore_black_pixels && is_black) {
        continue;
      }

      let channel_range = if config.ignore_alpha { 0..3 } else { 0..4 };
      for i in channel_range {
        if let Some(bit) = iter_chain() {
          let bit: u8 = bit.to_digit(10).unwrap() as u8;

          pixel.0[i] &= 0xFE; // Clear the least significant bit

          if config.debug {
            println!("  Channel({}) [{} -> {}]. Bit \"{}\"", Channel::new(i).name(), pixel.0[i], pixel.0[i] | bit, bit);
          }

          pixel.0[i] |= bit; // Set the least significant bit to the message bit
        } else {
          // Stop if there are no more bits to encode
          return encoded_img;
        }
      }
    }

    encoded_img
  }

  /// Convert a message string to an iterator of bits.
  fn message_to_bit_iter(message: &str) -> impl Iterator<Item = char> + '_ {
    message
      .as_bytes() // Convert the string to a byte slice using UTF-8 encoding
      .iter()
      .flat_map(|&b| format!("{:08b}", b).chars().collect::<Vec<char>>()) // Convert each byte to its binary representation
  }

  fn message_to_bit_iter_utf32(message: &str) -> impl Iterator<Item = char> + '_ {
    message.chars().flat_map(|c| {
        format!("{:032b}", c as u32).chars().collect::<Vec<char>>()
    })
  }

  pub fn generate_image (mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let img_path = &cx.argument::<JsString>(0)?.value(&mut cx);
    let dynamic_img: Result<DynamicImage, ImageError> = image::open(img_path);

    if dynamic_img.is_err() {
      eprintln!("Error: Failed to open the image");
      return Err(cx.throw_error("Failed to open the image")?);
    }

    let img = dynamic_img.unwrap().to_rgba8();

    let result_img_path = if cx.argument::<JsString>(2).is_ok() {
      cx.argument::<JsString>(2)?.value(&mut cx)
    } else {
      img_path.split(path::MAIN_SEPARATOR).last().unwrap().to_string() + "_encoded.png"
    };

    let config = EncodeConfig {
      ignore_alpha: true,
      ignore_white_pixels: true,
      ignore_black_pixels: false,
      debug: true,
      no_panic: true
    };

    // let message: String = String::from("Hello, world! Witaj, świecie ! Здравствуйте ! 你好");
    let message: String = cx.argument::<JsString>(1)?.value(&mut cx);

    let capacity = calculate_capacity(&img, &config);
    let (encoding, mut message_size) = calculate_bits_of_encoded_string(&message);

    message_size += 32; // 32 bits for the length
    message_size += 8; // 8 bits for the encoding enum value

    println!("Message: \"{}\". Length: {}, Determined Encoding: {}, Message size: {} bits",
       message, message.chars().count(), encoding, message_size);

    println!("Capacity: {} bits", capacity);

    if message_size as usize > capacity {
      eprintln!("Error: The message is too long to fit in the given image.");
      return Err(cx.throw_error("The message is too long to fit in the given image")?);
    }

    let encoded_img: ImageBuffer<Rgba<u8>, Vec<u8>> = encode_image(&img, &message,
      encoding, config);

    encoded_img
      .save(result_img_path)
      .expect("Failed to save the image");

    Ok(cx.boolean(true))
  }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function("generate_image", encoder::generate_image)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::encoder::{CharacterEncoding, calculate_bits_of_encoded_string};

  #[test]
  fn test_calculate_bits_of_encoded_string_ascii() {
    let message = "ASCII message";
    let (encoding, bits) = calculate_bits_of_encoded_string(message);
    assert_eq!(encoding.to_bit_value(), CharacterEncoding::ASCII.to_bit_value());
    // "ASCII message" has 13 characters, each should be 8 bits in ASCII
    assert_eq!(bits, 13 * 8);
  }

  #[test]
  fn test_calculate_bits_of_encoded_string_utf8() {
    // Using a character that requires 2 bytes in UTF-8
    let message = "ñ";
    let (encoding, bits) = calculate_bits_of_encoded_string(message);
    assert_eq!(encoding.to_bit_value(), CharacterEncoding::UTF8.to_bit_value());
    // "ñ" is represented with 2 bytes in UTF-8
    assert_eq!(bits, 16);
  }

  #[test]
  fn test_calculate_bits_of_encoded_string_utf32() {
    // Using a character that is outside the basic multilingual plane and requires 4 bytes
    let message = "𐍈"; // Example of a character that would be encoded using UTF-32
    let (encoding, bits) = calculate_bits_of_encoded_string(message);
    assert_eq!(encoding.to_bit_value(), CharacterEncoding::UTF32.to_bit_value());
    // Each character in UTF-32 is represented with 4 bytes
    assert_eq!(bits, 32);
  }
}
