use core::fmt;
use std::{env, path};
use image::{DynamicImage, ImageBuffer, Pixel, Rgba, RgbaImage};
use session::Session;
use prost::Message;

mod session {
  include!(concat!(env!("OUT_DIR"), "/_.rs"));
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

/// Calculate the total number of bits available for encoding in the image.
fn calculate_capacity(image: &RgbaImage, config: &EncodeConfig) -> usize {
  let channels_num = if config.ignore_alpha { 3 } else { 4 };
  image.width() as usize * image.height() as usize * channels_num
}

fn calculate_bits_of_encoded_string(s: &str) -> (CharacterEncoding, u32) {
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

fn to_binary_chunks(text: &str, chunk_size: usize) -> String {
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

fn decode_image(img: &RgbaImage, config: EncodeConfig) -> String {
  let mut bit_iter = Vec::new();

  let mut character_encoding: Option<CharacterEncoding> = None;
  let mut message_length: Option<u32> = None;

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
          CharacterEncoding::new(bits_to_u8(bit_iter[0..8].to_vec()))
        );

        message_length = Some(
          bits_to_u32(bit_iter[8..40].to_vec())
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
            .map(|chunk| bits_to_u8(chunk.to_vec()))
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

  let mut iter = bit_iter.chunks(32);
  let mut result = String::new();

  while let Some(chunk) = iter.next() {
    let mut code_point = 0u32;
    for &bit in chunk {
      code_point <<= 1; // Shift left by 1 to make space for the next bit
      code_point |= if bit == 1 { 1 } else { 0 }; // Set the last bit based on the current char
    }
    result.push(std::char::from_u32(code_point).unwrap_or('?')); // Convert the code point to a char, using '?' for invalid code points
  }

  result
}

fn bits_to_u8(bits: Vec<u8>) -> u8 {
  bits.into_iter().enumerate().fold(0, |acc, (index, bit)| {
    // Ensure that each bit is either 0 or 1.
    if bit > 1 {
        panic!("Each element in the input vector should be either 0 or 1.");
    }

    // Shift the bit to its correct position and combine it with the accumulator.
    acc | (bit << (7 - index))
  })
}

fn bits_to_u32(bits: Vec<u8>) -> u32 {
  if bits.len() != 32 {
      panic!("The input vector must have exactly 32 elements.");
  }

  let mut result = 0u32;
  for bit in bits {
      // Ensure that each element in the vector is either 0 or 1.
      if bit != 0 && bit != 1 {
          panic!("Each element in the input vector must be 0 or 1.");
      }
      
      result <<= 1; // Shift the result left by 1 bit to make room for the next bit.
      result |= bit as u32; // Set the last bit of the result to the current bit value.
  }
  result
}

fn protobufs() {
  let session = Session {
    username: "alza54".to_string(),
    password: "pass_w0rd".to_string(),
    token: "abcdef-abcdef-abcdef-abcdef".to_string()
  };

  // Encode the person into a vector of bytes (Vec<u8>)
  let mut buf = Vec::new();
  buf.reserve(session.encoded_len());
  session.encode(&mut buf).unwrap();
  println!("Encoded: {:?}", buf);

  // Decode the vector of bytes back into a person
  let decoded: Session = Session::decode(&*buf).unwrap();
  println!("Decoded: {:?}", decoded);
}

fn main() {
  let args: Vec<String> = env::args().collect();

  protobufs();

  if args.len() < 3 {
    eprintln!("Usage: {} <encode|decode> <image_path>", args[0]);
    return;
  }

  let result_img_path = if args.len() >= 4 {
    String::from(&args[3])
  } else {
    args[2].split(path::MAIN_SEPARATOR).last().unwrap().to_string() + "_encoded.png"
  };

  if args[1] == "decode" {
    // Decode the image
    let img_path: &String = &args[2];
    let dynamic_img: DynamicImage = image::open(img_path)
      .expect("Failed to open the image");
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = dynamic_img.to_rgba8();

    let config = EncodeConfig {
      ignore_alpha: true,
      ignore_white_pixels: true,
      ignore_black_pixels: false,
      debug: true,
      no_panic: true
    };

    let decoded: String = decode_image(&img, config);

    println!("Decoded message: {}", decoded);

    return;
  }

  if args[1] != "encode" {
    eprintln!("Usage: {} encode <image_path> [<output_path>]", args[0]);
    return;
  }

  // Load the image
  let img_path: &String = &args[2];
  let dynamic_img: DynamicImage = image::open(img_path)
    .expect("Failed to open the image");
  let img = dynamic_img.to_rgba8();

  let config = EncodeConfig {
    ignore_alpha: true,
    ignore_white_pixels: true,
    ignore_black_pixels: false,
    debug: true,
    no_panic: true
  };

  let message: String = String::from("Hello, world! Witaj, świecie ! Здравствуйте ! 你好");

  let capacity = calculate_capacity(&img, &config);
  let (encoding, mut message_size) = calculate_bits_of_encoded_string(&message);

  message_size += 32; // 32 bits for the length
  message_size += 8; // 8 bits for the encoding enum value

  println!("Message: \"{}\". Length: {}, Determined Encoding: {}, Message size: {} bits",
    message, message.chars().count(), encoding, message_size);

  println!("Capacity: {} bits", capacity);

  if message_size as usize > capacity {
    eprintln!("Error: The message is too long to fit in the given image.");
    return;
  }

  let encoded_img: ImageBuffer<Rgba<u8>, Vec<u8>> = encode_image(&img, &message, 
    encoding, config);

  encoded_img
    .save(result_img_path)
    .expect("Failed to save the image");
}
