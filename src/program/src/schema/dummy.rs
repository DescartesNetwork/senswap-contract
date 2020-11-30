#![cfg(feature = "program")]

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
};

//
// Define the data struct
//
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Dummy {
  pub amount: u32,
  pub toggle: bool,
}

//
// Implement Sealed trait
//
impl Sealed for Dummy {}

//
// Implement IsInitialized trait
//
impl IsInitialized for Dummy {
  fn is_initialized(&self) -> bool {
    true
  }
}

//
// Implement Pack trait
//
impl Pack for Dummy {
  // Fixed length
  const LEN: usize = 5;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 5];
    let (amount, toggle) = array_refs![src, 4, 1];
    let amount = u32::from_le_bytes(*amount);
    let toggle = match toggle {
      [0] => false,
      [1] => true,
      _ => return Err(ProgramError::InvalidAccountData),
    };
    Ok(Dummy { amount, toggle })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 5];
    let (dst_amount, dst_toggle) = mut_array_refs![dst, 4, 1];
    let &Dummy { amount, toggle } = self;
    *dst_amount = amount.to_le_bytes();
    *dst_toggle = match toggle {
      true => [1],
      _ => [0],
    };
  }
}
