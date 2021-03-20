use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
  info,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

///
/// LPT struct
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LPT {
  pub owner: Pubkey,
  pub pool: Pubkey,
  pub lpt: u128,
  pub is_initialized: bool,
}

///
/// Sealed trait
///
impl Sealed for LPT {}

///
/// IsInitialized trait
///
impl IsInitialized for LPT {
  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}

///
/// Pack trait
///
impl Pack for LPT {
  // Fixed length
  const LEN: usize = 32 + 32 + 16 + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    info!("Read LPT data");
    let src = array_ref![src, 0, 81];
    let (owner, pool, lpt, is_initialized) = array_refs![src, 32, 32, 16, 1];
    Ok(LPT {
      owner: Pubkey::new_from_array(*owner),
      pool: Pubkey::new_from_array(*pool),
      lpt: u128::from_le_bytes(*lpt),
      is_initialized: match is_initialized {
        [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidAccountData),
      },
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    info!("Write LPT data");
    let dst = array_mut_ref![dst, 0, 81];
    let (dst_owner, dst_pool, dst_lpt, dst_is_initialized) = mut_array_refs![dst, 32, 32, 16, 1];
    let &LPT {
      ref owner,
      ref pool,
      lpt,
      is_initialized,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    dst_pool.copy_from_slice(pool.as_ref());
    *dst_lpt = lpt.to_le_bytes();
    *dst_is_initialized = [is_initialized as u8];
  }
}
