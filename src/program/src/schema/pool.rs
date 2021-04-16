use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  info,
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
  pub network: Pubkey,
  pub mint: Pubkey,
  pub treasury: Pubkey,
  pub reserve: u64,
  pub lpt: u128,
  pub state: PoolState,
}

///
/// Pool implementation
///
impl Pool {
  // Is frozen
  pub fn is_frozen(&self) -> bool {
    self.state == PoolState::Frozen
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
  const LEN: usize = 32 + 32 + 32 + 8 + 16 + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    info!("Read pool data");
    let src = array_ref![src, 0, 121];
    let (network, mint, treasury, reserve, lpt, state) = array_refs![src, 32, 32, 32, 8, 16, 1];
    Ok(Pool {
      network: Pubkey::new_from_array(*network),
      mint: Pubkey::new_from_array(*mint),
      treasury: Pubkey::new_from_array(*treasury),
      reserve: u64::from_le_bytes(*reserve),
      lpt: u128::from_le_bytes(*lpt),
      state: PoolState::try_from_primitive(state[0]).or(Err(ProgramError::InvalidAccountData))?,
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    info!("Write pool data");
    let dst = array_mut_ref![dst, 0, 121];
    let (dst_network, dst_mint, dst_treasury, dst_reserve, dst_lpt, dst_state) =
      mut_array_refs![dst, 32, 32, 32, 8, 16, 1];
    let &Pool {
      ref network,
      ref mint,
      ref treasury,
      reserve,
      lpt,
      state,
    } = self;
    dst_network.copy_from_slice(network.as_ref());
    dst_mint.copy_from_slice(mint.as_ref());
    dst_treasury.copy_from_slice(treasury.as_ref());
    *dst_reserve = reserve.to_le_bytes();
    *dst_lpt = lpt.to_le_bytes();
    *dst_state = [state as u8];
  }
}
