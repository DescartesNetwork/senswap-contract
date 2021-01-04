use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
  program_error::ProgramError,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

//
// Define the data struct
//
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pool {
  pub token: Pubkey,
  pub treasury: Pubkey,
  pub reserve: u64,
  pub sen: u64,
  pub fee_numerator: u64,
  pub fee_denominator: u64,
  pub initialized: bool,
}

//
// Implement Sealed trait
//
impl Sealed for Pool {}

//
// Implement IsInitialized trait
//
impl IsInitialized for Pool {
  fn is_initialized(&self) -> bool {
    self.initialized
  }
}

//
// Implement Pack trait
//
impl Pack for Pool {
  // Fixed length
  const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 1;
  // Unpack data from [u8] to the data struct
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 97];
    let (token, treasury, reserve, sen, fee_numerator, fee_denominator, initialized) =
      array_refs![src, 32, 32, 8, 8, 8, 8, 1];
    Ok(Pool {
      token: Pubkey::new_from_array(*token),
      treasury: Pubkey::new_from_array(*treasury),
      reserve: u64::from_le_bytes(*reserve),
      sen: u64::from_le_bytes(*sen),
      fee_numerator: u64::from_le_bytes(*fee_numerator),
      fee_denominator: u64::from_le_bytes(*fee_denominator),
      initialized: match initialized {
        [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidAccountData),
      },
    })
  }
  // Pack data from the data struct to [u8]
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 97];
    let (
      dst_token,
      dst_treasury,
      dst_reserve,
      dst_sen,
      dst_fee_numerator,
      dst_fee_denominator,
      dst_initialized,
    ) = mut_array_refs![dst, 32, 32, 8, 8, 8, 8, 1];
    let &Pool {
      token,
      treasury,
      reserve,
      sen,
      fee_numerator,
      fee_denominator,
      initialized,
    } = self;
    dst_token.copy_from_slice(token.as_ref());
    dst_treasury.copy_from_slice(treasury.as_ref());
    *dst_reserve = reserve.to_le_bytes();
    *dst_sen = sen.to_le_bytes();
    *dst_fee_numerator = fee_numerator.to_le_bytes();
    *dst_fee_denominator = fee_denominator.to_le_bytes();
    *dst_initialized = [initialized as u8];
  }
}
