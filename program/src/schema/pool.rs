use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  msg,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

///
/// Pool state
///
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
pub enum PoolState {
  Uninitialized,
  Initialized,
  Frozen,
}
impl Default for PoolState {
  fn default() -> Self {
    PoolState::Uninitialized
  }
}

///
/// Pool struct
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pool {
  pub owner: Pubkey,
  pub state: PoolState,
  pub mint_lpt: Pubkey,
  pub vault: Pubkey,

  pub mint_s: Pubkey,
  pub treasury_s: Pubkey,
  pub reserve_s: u64,

  pub mint_a: Pubkey,
  pub treasury_a: Pubkey,
  pub reserve_a: u64,

  pub mint_b: Pubkey,
  pub treasury_b: Pubkey,
  pub reserve_b: u64,
}

///
/// Pool implementation
///
impl Pool {
  // Is frozen
  pub fn is_frozen(&self) -> bool {
    self.state == PoolState::Frozen
  }
  // Verify the pair of mint and treasury
  // 0: None, 1: S pool, 2: A pool, 3: B pool
  pub fn get_reserve(&self, treasury: &Pubkey) -> Option<(u8, u64)> {
    if self.treasury_s == *treasury {
      return Some((0, self.reserve_s));
    }
    if self.treasury_a == *treasury {
      return Some((1, self.reserve_a));
    }
    if self.treasury_b == *treasury {
      return Some((2, self.reserve_b));
    }

    None
  }
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
    self.state != PoolState::Uninitialized
  }
}

///
/// Pack trait
///
impl Pack for Pool {
  // Fixed length
  const LEN: usize = 32 + 1 + 32 + 32 + 3 * (32 + 32 + 8);
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    msg!("Read pool data");
    let src = array_ref![src, 0, 313];
    let (
      owner,
      state,
      mint_lpt,
      vault,
      mint_s,
      treasury_s,
      reserve_s,
      mint_a,
      treasury_a,
      reserve_a,
      mint_b,
      treasury_b,
      reserve_b,
    ) = array_refs![src, 32, 1, 32, 32, 32, 32, 8, 32, 32, 8, 32, 32, 8];
    Ok(Pool {
      owner: Pubkey::new_from_array(*owner),
      state: PoolState::try_from_primitive(state[0]).or(Err(ProgramError::InvalidAccountData))?,
      mint_lpt: Pubkey::new_from_array(*mint_lpt),
      vault: Pubkey::new_from_array(*vault),
      mint_s: Pubkey::new_from_array(*mint_s),
      treasury_s: Pubkey::new_from_array(*treasury_s),
      reserve_s: u64::from_le_bytes(*reserve_s),
      mint_a: Pubkey::new_from_array(*mint_a),
      treasury_a: Pubkey::new_from_array(*treasury_a),
      reserve_a: u64::from_le_bytes(*reserve_a),
      mint_b: Pubkey::new_from_array(*mint_b),
      treasury_b: Pubkey::new_from_array(*treasury_b),
      reserve_b: u64::from_le_bytes(*reserve_b),
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    msg!("Write pool data");
    let dst = array_mut_ref![dst, 0, 313];
    let (
      dst_owner,
      dst_state,
      dst_mint_lpt,
      dst_vault,
      dst_mint_s,
      dst_treasury_s,
      dst_reserve_s,
      dst_mint_a,
      dst_treasury_a,
      dst_reserve_a,
      dst_mint_b,
      dst_treasury_b,
      dst_reserve_b,
    ) = mut_array_refs![dst, 32, 1, 32, 32, 32, 32, 8, 32, 32, 8, 32, 32, 8];
    let &Pool {
      ref owner,
      state,
      ref mint_lpt,
      ref vault,
      ref mint_s,
      ref treasury_s,
      reserve_s,
      ref mint_a,
      ref treasury_a,
      reserve_a,
      ref mint_b,
      ref treasury_b,
      reserve_b,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    *dst_state = [state as u8];
    dst_mint_lpt.copy_from_slice(mint_lpt.as_ref());
    dst_vault.copy_from_slice(vault.as_ref());
    dst_mint_s.copy_from_slice(mint_s.as_ref());
    dst_treasury_s.copy_from_slice(treasury_s.as_ref());
    *dst_reserve_s = reserve_s.to_le_bytes();
    dst_mint_a.copy_from_slice(mint_a.as_ref());
    dst_treasury_a.copy_from_slice(treasury_a.as_ref());
    *dst_reserve_a = reserve_a.to_le_bytes();
    dst_mint_b.copy_from_slice(mint_b.as_ref());
    dst_treasury_b.copy_from_slice(treasury_b.as_ref());
    *dst_reserve_b = reserve_b.to_le_bytes();
  }
}
