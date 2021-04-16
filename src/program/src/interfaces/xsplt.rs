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
    target_acc: &'a AccountInfo<'a>,
    mint_acc: &'a AccountInfo<'a>,
    owner: &'a AccountInfo<'a>,
    sysvar_rent_acc: &'a AccountInfo<'a>,
    splt_program: &'a AccountInfo<'a>,
    seed: &[u8],
  ) -> ProgramResult {
    let ix = ISPLT::initialize_account(
      *target_acc.key,
      *mint_acc.key,
      *owner.key,
      *sysvar_rent_acc.key,
      *splt_program.key,
    )?;
    invoke_signed(
      &ix,
      &[
        target_acc.clone(),
        mint_acc.clone(),
        owner.clone(),
        sysvar_rent_acc.clone(),
        splt_program.clone(),
      ],
      &[&[&seed]],
    )?;
    Ok(())
  }
  ///
  /// Initialize account
  ///
  pub fn transfer<'a>(
    amount: u64,
    src_acc: &'a AccountInfo<'a>,
    dst_acc: &'a AccountInfo<'a>,
    owner: &'a AccountInfo<'a>,
    splt_program: &'a AccountInfo<'a>,
    seed: &[u8],
  ) -> ProgramResult {
    let ix = ISPLT::transfer(
      amount,
      *src_acc.key,
      *dst_acc.key,
      *owner.key,
      *splt_program.key,
    )?;
    invoke_signed(
      &ix,
      &[
        src_acc.clone(),
        dst_acc.clone(),
        owner.clone(),
        splt_program.clone(),
      ],
      &[&[&seed]],
    )?;
    Ok(())
  }
  ///
  /// Initialize account
  ///
  pub fn close_account<'a>(
    src_acc: &'a AccountInfo<'a>,
    dst_acc: &'a AccountInfo<'a>,
    owner: &'a AccountInfo<'a>,
    splt_program: &'a AccountInfo<'a>,
    seed: &[u8],
  ) -> ProgramResult {
    let ix = ISPLT::close_account(*src_acc.key, *dst_acc.key, *owner.key, *splt_program.key)?;
    invoke_signed(
      &ix,
      &[
        src_acc.clone(),
        dst_acc.clone(),
        owner.clone(),
        splt_program.clone(),
      ],
      &[&[&seed]],
    )?;
    Ok(())
  }
}
