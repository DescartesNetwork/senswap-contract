use crate::interfaces::isplt::ISPLT;
use solana_program::{
  account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
};

pub struct XSPLT {}

impl XSPLT {
  ///
  /// Initialize account
  ///
  pub fn initialize_account<'a>(
    target_acc: &'a AccountInfo,
    mint_acc: &'a AccountInfo,
    owner: &'a AccountInfo,
    sysvar_rent_acc: &'a AccountInfo,
    splt_program: &'a AccountInfo,
    seed: &[u8],
  ) -> ProgramResult {
    let ix_initialize_account = ISPLT::initialize_account(
      *target_acc.key,
      *mint_acc.key,
      *owner.key,
      *sysvar_rent_acc.key,
      *splt_program.key,
    )?;
    let account_infos: &'a [AccountInfo] = &[
      target_acc.clone(),
      mint_acc.clone(),
      owner.clone(),
      sysvar_rent_acc.clone(),
      splt_program.clone(),
    ];
    invoke_signed(&ix_initialize_account, account_infos, &[&[&seed]])?;
    Ok(())
  }
}
