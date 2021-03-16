use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

///
/// Pool struct
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pool {
  pub owner: Pubkey,
  pub mint: Pubkey,
  pub treasury: Pubkey,
  pub reserve: u64,
  pub lpt: u128,
  pub fee: u64,
  pub is_initialized: bool,
}

///
/// Sealed trait
///
impl Sealed for Pool {}

///
/// IsInitialized trait
///
impl IsInitialized for Pool {
  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}

///
/// Pack trait
///
impl Pack for Pool {
  // Fixed length
  const LEN: usize = 32 + 32 + 32 + 8 + 16 + 8 + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 129];
    let (owner, mint, treasury, reserve, lpt, fee, is_initialized) =
      array_refs![src, 32, 32, 32, 8, 16, 8, 1];
    Ok(Pool {
      owner: Pubkey::new_from_array(*owner),
      mint: Pubkey::new_from_array(*mint),
      treasury: Pubkey::new_from_array(*treasury),
      reserve: u64::from_le_bytes(*reserve),
      lpt: u128::from_le_bytes(*lpt),
      fee: u64::from_le_bytes(*fee),
      is_initialized: match is_initialized {
        [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidAccountData),
      },
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 129];
    let (dst_owner, dst_mint, dst_treasury, dst_reserve, dst_lpt, dst_fee, dst_is_initialized) =
      mut_array_refs![dst, 32, 32, 32, 8, 16, 8, 1];
    let &Pool {
      ref owner,
      ref mint,
      ref treasury,
      reserve,
      lpt,
      fee,
      is_initialized,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    dst_mint.copy_from_slice(mint.as_ref());
    dst_treasury.copy_from_slice(treasury.as_ref());
    *dst_reserve = reserve.to_le_bytes();
    *dst_lpt = lpt.to_le_bytes();
    *dst_fee = fee.to_le_bytes();
    *dst_is_initialized = [is_initialized as u8];
  }
}
