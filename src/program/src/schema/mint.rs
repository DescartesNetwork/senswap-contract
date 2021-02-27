use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
  program_error::ProgramError,
  program_option::COption,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

///
/// Mint struct
///
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mint {
  /// Optional authority used to mint new tokens. The mint authority may only be provided during
  /// mint creation. If no mint authority is present then the mint has a fixed supply and no
  /// further tokens may be minted.
  pub mint_authority: COption<Pubkey>,
  /// Total supply of tokens.
  pub supply: u64,
  /// Number of base 10 digits to the right of the decimal place.
  pub decimals: u8,
  /// Is `true` if this structure has been initialized
  pub is_initialized: bool,
  /// Optional authority to freeze token accounts.
  pub freeze_authority: COption<Pubkey>,
}

///
/// Sealed trait
///
impl Sealed for Mint {}

///
/// IsInitialized trait
impl IsInitialized for Mint {
  fn is_initialized(&self) -> bool {
    self.is_initialized
  }
}

///
/// Pack trait
///
impl Pack for Mint {
  const LEN: usize = 82;
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 82];
    let (mint_authority, supply, decimals, is_initialized, freeze_authority) =
      array_refs![src, 36, 8, 1, 1, 36];
    let mint_authority = unpack_coption_key(mint_authority)?;
    let supply = u64::from_le_bytes(*supply);
    let decimals = decimals[0];
    let is_initialized = match is_initialized {
      [0] => false,
      [1] => true,
      _ => return Err(ProgramError::InvalidAccountData),
    };
    let freeze_authority = unpack_coption_key(freeze_authority)?;
    Ok(Mint {
      mint_authority,
      supply,
      decimals,
      is_initialized,
      freeze_authority,
    })
  }
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 82];
    let (mint_authority_dst, supply_dst, decimals_dst, is_initialized_dst, freeze_authority_dst) =
      mut_array_refs![dst, 36, 8, 1, 1, 36];
    let &Mint {
      ref mint_authority,
      supply,
      decimals,
      is_initialized,
      ref freeze_authority,
    } = self;
    pack_coption_key(mint_authority, mint_authority_dst);
    *supply_dst = supply.to_le_bytes();
    decimals_dst[0] = decimals;
    is_initialized_dst[0] = is_initialized as u8;
    pack_coption_key(freeze_authority, freeze_authority_dst);
  }
}

///
/// Utility
///
fn pack_coption_key(src: &COption<Pubkey>, dst: &mut [u8; 36]) {
  let (tag, body) = mut_array_refs![dst, 4, 32];
  match src {
    COption::Some(key) => {
      *tag = [1, 0, 0, 0];
      body.copy_from_slice(key.as_ref());
    }
    COption::None => {
      *tag = [0; 4];
    }
  }
}
fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>, ProgramError> {
  let (tag, body) = array_refs![src, 4, 32];
  match *tag {
    [0, 0, 0, 0] => Ok(COption::None),
    [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
    _ => Err(ProgramError::InvalidAccountData),
  }
}
