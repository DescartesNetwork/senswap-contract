use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

pub const MAX_SIGNERS: usize = 11;

///
/// Network struct
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DAO {
  pub signers: [Pubkey; MAX_SIGNERS],
  pub is_initialized: bool,
}

///
/// DAO implementation
///
impl DAO {
  // Get the number of current signers
  pub fn num_signers(&self) -> u64 {
    let mut counter: u64 = 0;
    for signer in self.signers.iter() {
      if *signer != Pubkey::new(&[0u8; 32]) {
        counter = counter + 1;
      }
    }
    counter
  }
}

///
/// Sealed trait
///
impl Sealed for DAO {}

///
/// IsInitialized trait
///
impl IsInitialized for DAO {
  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}

///
/// Pack trait
///
impl Pack for DAO {
  // Fixed length
  const LEN: usize = 32 * MAX_SIGNERS + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 353];
    let (signers_flat, is_initialized) = array_refs![src, 32 * MAX_SIGNERS, 1];
    let mut dao = DAO {
      signers: [Pubkey::new_from_array([0u8; 32]); MAX_SIGNERS],
      is_initialized: match is_initialized {
        [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidAccountData),
      },
    };
    for (src, dst) in signers_flat.chunks(32).zip(dao.signers.iter_mut()) {
      *dst = Pubkey::new(src);
    }
    Ok(dao)
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 353];
    let (dst_signers_flat, dst_is_initialized) = mut_array_refs![dst, 32 * MAX_SIGNERS, 1];
    let &DAO {
      ref signers,
      is_initialized,
    } = self;
    for (i, src) in signers.iter().enumerate() {
      let dst_array = array_mut_ref![dst_signers_flat, 32 * i, 32];
      dst_array.copy_from_slice(src.as_ref());
    }
    *dst_is_initialized = [is_initialized as u8];
  }
}
