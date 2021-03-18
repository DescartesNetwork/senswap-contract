use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

pub const MAX_MINTS: usize = 11;

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
  pub primary_token: Pubkey, // Must be SEN
  pub vault: Pubkey,         // A SEN account
  pub mints: [Pubkey; MAX_MINTS],
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
  // The mint is legally included in network
  pub fn is_approved(&self, mint: &Pubkey) -> bool {
    if *mint == Pubkey::new(&[0u8; 32]) {
      return false;
    }
    for m in self.mints.iter() {
      if *m == *mint {
        return true;
      }
    }
    false
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
  const LEN: usize = 32 + 32 + 32 + 32 * MAX_MINTS + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 449];
    let (owner, primary_token, vault, mints_flat, state) =
      array_refs![src, 32, 32, 32, 32 * MAX_MINTS, 1];
    let mut network = Network {
      owner: Pubkey::new_from_array(*owner),
      primary_token: Pubkey::new_from_array(*primary_token),
      vault: Pubkey::new_from_array(*vault),
      mints: [Pubkey::new_from_array([0u8; 32]); MAX_MINTS],
      state: NetworkState::try_from_primitive(state[0])
        .or(Err(ProgramError::InvalidAccountData))?,
    };
    for (src, dst) in mints_flat.chunks(32).zip(network.mints.iter_mut()) {
      *dst = Pubkey::new(src);
    }
    Ok(network)
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 449];
    let (dst_owner, dst_primary_token, dst_vault, dst_mints_flat, dst_state) =
      mut_array_refs![dst, 32, 32, 32, 32 * MAX_MINTS, 1];
    let &Network {
      ref owner,
      ref primary_token,
      ref vault,
      ref mints,
      state,
    } = self;
    dst_owner.copy_from_slice(owner.as_ref());
    dst_primary_token.copy_from_slice(primary_token.as_ref());
    dst_vault.copy_from_slice(vault.as_ref());
    for (i, src) in mints.iter().enumerate() {
      let dst_array = array_mut_ref![dst_mints_flat, 32 * i, 32];
      dst_array.copy_from_slice(src.as_ref());
    }
    *dst_state = [state as u8];
  }
}
