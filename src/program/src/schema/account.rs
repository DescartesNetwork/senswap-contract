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
pub struct Account {
  pub owner: Pubkey,
  pub token: Pubkey,
  pub amount: u64,
  pub initialized: bool,
}

//
// Implement Sealed trait
//
impl Sealed for Account {}

//
// Implement IsInitialized trait
//
impl IsInitialized for Account {
  fn is_initialized(&self) -> bool {
    self.initialized
  }
}

//
// Implement Pack trait
//
impl Pack for Account {
  // Fixed length
  const LEN: usize = 32 + 32 + 8 + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 73];
    let (owner, token, amount, initialized) = array_refs![src, 32, 32, 8, 1];
    Ok(Account {
      owner: Pubkey::new_from_array(*owner),
      token: Pubkey::new_from_array(*token),
      amount: u64::from_le_bytes(*amount),
      initialized: match initialized {
        [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidAccountData),
      },
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 73];
    let (dst_owner, dst_token, dst_amount, dst_initialized) = mut_array_refs![dst, 32, 32, 8, 1];
    let &Account {
      ref owner,
      ref token,
      amount,
      initialized,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    dst_token.copy_from_slice(token.as_ref());
    *dst_amount = amount.to_le_bytes();
    *dst_initialized = [initialized as u8];
  }
}
