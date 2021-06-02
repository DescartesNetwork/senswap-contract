use solana_program::{
  instruction::{AccountMeta, Instruction},
  program_error::ProgramError,
  pubkey::Pubkey,
};
use std::mem::size_of;

pub struct ISPLATA {}

impl ISPLATA {
  ///
  /// Initialize account
  ///
  pub fn initialize_account(
    funding_acc: Pubkey,
    target_acc: Pubkey,
    owner: Pubkey,
    mint_acc: Pubkey,
    system_program: Pubkey,
    splt_program: Pubkey,
    sysvar_rent_acc: Pubkey,
    program_id: Pubkey,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let data = Vec::with_capacity(size_of::<Self>());
    // Build accounts
    let mut accounts = Vec::with_capacity(7);
    accounts.push(AccountMeta::new(funding_acc, true));
    accounts.push(AccountMeta::new(target_acc, false));
    accounts.push(AccountMeta::new_readonly(owner, false));
    accounts.push(AccountMeta::new_readonly(mint_acc, false));
    accounts.push(AccountMeta::new_readonly(system_program, false));
    accounts.push(AccountMeta::new_readonly(splt_program, false));
    accounts.push(AccountMeta::new_readonly(sysvar_rent_acc, false));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }
}
