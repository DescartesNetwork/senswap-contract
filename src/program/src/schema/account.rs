use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use num_enum::TryFromPrimitive;
use solana_program::{
  program_error::ProgramError,
  program_option::COption,
  program_pack::{IsInitialized, Pack, Sealed},
  pubkey::Pubkey,
};

///
/// Account struct
///
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Account {
  /// The mint associated with this account
  pub mint: Pubkey,
  /// The owner of this account.
  pub owner: Pubkey,
  /// The amount of tokens this account holds.
  pub amount: u64,
  /// If `delegate` is `Some` then `delegated_amount` represents
  /// the amount authorized by the delegate
  pub delegate: COption<Pubkey>,
  /// The account's state
  pub state: AccountState,
  /// If is_some, this is a native token, and the value logs the rent-exempt reserve. An Account
  /// is required to be rent-exempt, so the value is used by the Processor to ensure that wrapped
  /// SOL accounts do not drop below this threshold.
  pub is_native: COption<u64>,
  /// The amount delegated
  pub delegated_amount: u64,
  /// Optional authority to close the account.
  pub close_authority: COption<Pubkey>,
}

///
/// Account implementation
///
impl Account {
  /// Checks if account is frozen
  pub fn is_frozen(&self) -> bool {
    self.state == AccountState::Frozen
  }
  /// Checks if account is native
  pub fn is_native(&self) -> bool {
    self.is_native.is_some()
  }
}

///
/// Sealed trait
///
impl Sealed for Account {}

impl IsInitialized for Account {
  fn is_initialized(&self) -> bool {
    self.state != AccountState::Uninitialized
  }
}

///
/// Pack trait
///
impl Pack for Account {
  const LEN: usize = 165;
  fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
    let src = array_ref![src, 0, 165];
    let (mint, owner, amount, delegate, state, is_native, delegated_amount, close_authority) =
      array_refs![src, 32, 32, 8, 36, 1, 12, 8, 36];
    Ok(Account {
      mint: Pubkey::new_from_array(*mint),
      owner: Pubkey::new_from_array(*owner),
      amount: u64::from_le_bytes(*amount),
      delegate: unpack_coption_key(delegate)?,
      state: AccountState::try_from_primitive(state[0])
        .or(Err(ProgramError::InvalidAccountData))?,
      is_native: unpack_coption_u64(is_native)?,
      delegated_amount: u64::from_le_bytes(*delegated_amount),
      close_authority: unpack_coption_key(close_authority)?,
    })
  }
  fn pack_into_slice(&self, dst: &mut [u8]) {
    let dst = array_mut_ref![dst, 0, 165];
    let (
      mint_dst,
      owner_dst,
      amount_dst,
      delegate_dst,
      state_dst,
      is_native_dst,
      delegated_amount_dst,
      close_authority_dst,
    ) = mut_array_refs![dst, 32, 32, 8, 36, 1, 12, 8, 36];
    let &Account {
      ref mint,
      ref owner,
      amount,
      ref delegate,
      state,
      ref is_native,
      delegated_amount,
      ref close_authority,
    } = self;
    mint_dst.copy_from_slice(mint.as_ref());
    owner_dst.copy_from_slice(owner.as_ref());
    *amount_dst = amount.to_le_bytes();
    pack_coption_key(delegate, delegate_dst);
    state_dst[0] = state as u8;
    pack_coption_u64(is_native, is_native_dst);
    *delegated_amount_dst = delegated_amount.to_le_bytes();
    pack_coption_key(close_authority, close_authority_dst);
  }
}

///
/// Account state
///
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
pub enum AccountState {
  /// Account is not yet initialized
  Uninitialized,
  /// Account is initialized; the account owner and/or delegate may perform permitted operations
  /// on this account
  Initialized,
  /// Account has been frozen by the mint freeze authority. Neither the account owner nor
  /// the delegate are able to perform operations on this account.
  Frozen,
}

impl Default for AccountState {
  fn default() -> Self {
    AccountState::Uninitialized
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
fn pack_coption_u64(src: &COption<u64>, dst: &mut [u8; 12]) {
  let (tag, body) = mut_array_refs![dst, 4, 8];
  match src {
    COption::Some(amount) => {
      *tag = [1, 0, 0, 0];
      *body = amount.to_le_bytes();
    }
    COption::None => {
      *tag = [0; 4];
    }
  }
}
fn unpack_coption_u64(src: &[u8; 12]) -> Result<COption<u64>, ProgramError> {
  let (tag, body) = array_refs![src, 4, 8];
  match *tag {
    [0, 0, 0, 0] => Ok(COption::None),
    [1, 0, 0, 0] => Ok(COption::Some(u64::from_le_bytes(*body))),
    _ => Err(ProgramError::InvalidAccountData),
  }
}
