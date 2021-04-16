use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  info,
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

///
/// Network state
///
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
pub enum NetworkState {
  Uninitialized,
  Initialized,
  Activated,
}
impl Default for NetworkState {
  fn default() -> Self {
    NetworkState::Uninitialized
  }
}

///
/// Network struct
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Network {
  pub owner: Pubkey,         // Must be multisig
  pub primary_token: Pubkey, // SEN token
  pub primary_pool: Pubkey,  // A pool of SEN
  pub vault: Pubkey,         // A SEN account
  pub state: NetworkState,
}

///
/// Network implementation
///
impl Network {
  // Check legal to swap
  pub fn is_activated(&self) -> bool {
    self.state == NetworkState::Activated
  }
}

///
/// Sealed trait
///
impl Sealed for Network {}

///
/// IsInitialized trait
///
impl IsInitialized for Network {
  fn is_initialized(&self) -> bool {
    self.state != NetworkState::Uninitialized
  }
}

///
/// Pack trait
///
impl Pack for Network {
  // Fixed length
  const LEN: usize = 32 + 32 + 32 + 32 + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    info!("Read network data");
    let src = array_ref![src, 0, 129];
    let (owner, primary_token, primary_pool, vault, state) = array_refs![src, 32, 32, 32, 32, 1];
    let mut network = Network {
      owner: Pubkey::new_from_array(*owner),
      primary_token: Pubkey::new_from_array(*primary_token),
      primary_pool: Pubkey::new_from_array(*primary_pool),
      vault: Pubkey::new_from_array(*vault),
      state: NetworkState::try_from_primitive(state[0])
        .or(Err(ProgramError::InvalidAccountData))?,
    };
    Ok(network)
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    info!("Write network data");
    let dst = array_mut_ref![dst, 0, 129];
    let (dst_owner, dst_primary_token, dst_primary_pool, dst_vault, dst_state) =
      mut_array_refs![dst, 32, 32, 32, 32, 1];
    let &Network {
      ref owner,
      ref primary_token,
      ref primary_pool,
      ref vault,
      state,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    dst_primary_token.copy_from_slice(primary_token.as_ref());
    dst_primary_pool.copy_from_slice(primary_pool.as_ref());
    dst_vault.copy_from_slice(vault.as_ref());
    *dst_state = [state as u8];
  }
}
