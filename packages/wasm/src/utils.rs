#[allow(dead_code)]
pub fn set_panic_hook() {
  // When the `console_error_panic_hook` feature is enabled, we can call the
  // `set_panic_hook` function at least once during initialization, and then
  // we will get better error messages if our code ever panics.
  //
  // For more details see
  // https://github.com/rustwasm/console_error_panic_hook#readme
  #[cfg(feature = "console_error_panic_hook")]
  console_error_panic_hook::set_once();
}

pub fn bits_to_u8(bits: Vec<u8>, no_panic: bool) -> u8 {
  bits.into_iter().enumerate().fold(0, |acc, (index, bit)| {
    // Ensure that each bit is either 0 or 1.
    if bit > 1 {
      if !no_panic {
        panic!("Each element in the input vector should be either 0 or 1.");
      } else {
        return 0u8;
      }
    }

    // Shift the bit to its correct position and combine it with the accumulator.
    acc | (bit << (7 - index))
  })
}

pub fn bits_to_u32(bits: Vec<u8>, no_panic: bool) -> u32 {
  if bits.len() != 32 {
    if !no_panic {
      panic!("The input vector must have exactly 32 elements.");
    } else {
      return 0u32;
    }
  }

  let mut result = 0u32;
  for bit in bits {
    // Ensure that each element in the vector is either 0 or 1.
    if bit != 0 && bit != 1 {
      if !no_panic {
        panic!("Each element in the input vector must be 0 or 1.");
      } else {
        return 0u32;
      }
    }

    result <<= 1; // Shift the result left by 1 bit to make room for the next bit.
    result |= bit as u32; // Set the last bit of the result to the current bit value.
  }
  result
}
