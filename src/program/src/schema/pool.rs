#![cfg(feature = "program")]

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

//
// Define the data struct
//
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pool {
  pub token: Pubkey,
  pub reserve: u64,
  pub sen: u64,
  pub fee_numerator: u16,
  pub fee_denominator: u16,
}

//
// Implement Sealed trait
//
impl Sealed for Pool {}

//
// Implement IsInitialized trait
//
impl IsInitialized for Pool {
  fn is_initialized(&self) -> bool {
    true
  }
}

//
// Implement Pack trait
//
impl Pack for Pool {
  // Fixed length
  const LEN: usize = 32 + 8 + 8 + 2 + 2;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 52];
    let (token, reserve, sen, fee_numerator, fee_denominator) = array_refs![src, 32, 8, 8, 2, 2];
    Ok(Pool {
      token: Pubkey::new_from_array(*token),
      reserve: u64::from_le_bytes(*reserve),
      sen: u64::from_le_bytes(*sen),
      fee_numerator: u64::from_le_bytes(*fee_numerator),
      fee_denominator: u64::from_le_bytes(*fee_denominator),
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 52];
    let (dst_token, dst_reserve, dst_sen, dst_fee_numerator, dst_fee_denominator) =
      mut_array_refs![dst, 32, 8, 8, 2, 2];
    let &Pool {
      token,
      reserve,
      sen,
      fee_numerator,
      fee_denominator,
    } = self;
    dst_token.copy_from_slice(token.as_ref());
    *dst_reserve = reserve.to_le_bytes();
    *dst_sen = sen.to_le_bytes();
    *dst_fee_numerator = fee_numerator.to_le_bytes();
    *dst_fee_denominator = fee_denominator.to_le_bytes();
  }
}
