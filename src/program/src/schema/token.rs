#![cfg(feature = "program")]

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
};
use std::{char, convert::TryInto};

//
// Define the data struct
//
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Token {
  pub symbol: [char; 4],
  pub total_supply: u64,
  pub decimals: u8,
  pub initialized: bool,
}

//
// Implement Sealed trait
//
impl Sealed for Token {}

//
// Implement IsInitialized trait
//
impl IsInitialized for Token {
  fn is_initialized(&self) -> bool {
    self.initialized
  }
}

//
// Implement Pack trait
//
impl Pack for Token {
  // Fixed length
  const LEN: usize = 4 * 4 + 8 + 1 + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 26];
    let (symbol, total_supply, decimals, initialized) = array_refs![src, 16, 8, 1, 1];
    let vec_symbol: Vec<_> = symbol
      .chunks(4)
      .map(|slice| slice.try_into().unwrap())
      .map(|slice| u32::from_le_bytes(slice))
      .map(|slice| char::from_u32(slice).unwrap())
      .collect();
    Ok(Token {
      symbol: [vec_symbol[0], vec_symbol[1], vec_symbol[2], vec_symbol[3]],
      total_supply: u64::from_le_bytes(*total_supply),
      decimals: u8::from_le_bytes(*decimals),
      initialized: match initialized {
        [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidAccountData),
      },
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 26];
    let (dst_symbol, dst_total_supply, dst_decimals, dst_initialized) =
      mut_array_refs![dst, 16, 8, 1, 1];
    let (first_sym, second_sym, third_sym, forth_sym) = mut_array_refs![dst_symbol, 4, 4, 4, 4];
    let &Token {
      symbol,
      total_supply,
      decimals,
      initialized,
    } = self;
    symbol[0].encode_utf8(first_sym);
    symbol[1].encode_utf8(second_sym);
    symbol[2].encode_utf8(third_sym);
    symbol[3].encode_utf8(forth_sym);
    *dst_total_supply = total_supply.to_le_bytes();
    *dst_decimals = decimals.to_le_bytes();
    *dst_initialized = [initialized as u8];
  }
}
