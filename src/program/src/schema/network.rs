use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

pub const MAX_MINTS: usize = 32;

///
/// Network struct
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Network {
  pub mints: [Pubkey; MAX_MINTS],
  pub is_initialized: bool,
}

///
/// Network implementation
///
impl Network {
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
    self.is_initialized
  }
}

///
/// Pack trait
///
impl Pack for Network {
  // Fixed length
  const LEN: usize = 32 * MAX_MINTS + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 1025];
    let (mints_flat, is_initialized) = array_refs![src, 32 * MAX_MINTS, 1];
    let mut network = Network {
      mints: [Pubkey::new_from_array([0u8; 32]); MAX_MINTS],
      is_initialized: match is_initialized {
        [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidAccountData),
      },
    };
    for (src, dst) in mints_flat.chunks(32).zip(network.mints.iter_mut()) {
      *dst = Pubkey::new(src);
    }
    Ok(network)
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 1025];
    let (dst_mints_flat, dst_is_initialized) = mut_array_refs![dst, 32 * MAX_MINTS, 1];
    let &Network {
      ref mints,
      is_initialized,
    } = self;
    for (i, src) in mints.iter().enumerate() {
      let dst_array = array_mut_ref![dst_mints_flat, 32 * i, 32];
      dst_array.copy_from_slice(src.as_ref());
    }
    *dst_is_initialized = [is_initialized as u8];
  }
}
