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
pub struct Sen {
  pub owner: Pubkey,
  pub pool: Pubkey,
  pub sen: u64,
  pub initialized: bool,
}

//
// Implement Sealed trait
//
impl Sealed for Sen {}

//
// Implement IsInitialized trait
//
impl IsInitialized for Sen {
  fn is_initialized(&self) -> bool {
    self.initialized
  }
}

//
// Implement Pack trait
//
impl Pack for Sen {
  // Fixed length
  const LEN: usize = 32 + 32 + 8 + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 73];
    let (owner, pool, sen, initialized) = array_refs![src, 32, 32, 8, 1];
    Ok(Sen {
      owner: Pubkey::new_from_array(*owner),
      pool: Pubkey::new_from_array(*pool),
      sen: u64::from_le_bytes(*sen),
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
    let (dst_owner, dst_pool, dst_sen, dst_initialized) = mut_array_refs![dst, 32, 32, 8, 1];
    let &Sen {
      ref owner,
      ref pool,
      sen,
      initialized,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    dst_pool.copy_from_slice(pool.as_ref());
    *dst_sen = sen.to_le_bytes();
    *dst_initialized = [initialized as u8];
  }
}
