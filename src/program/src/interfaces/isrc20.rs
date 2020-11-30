#![cfg(feature = "program")]

use solana_sdk::{
  instruction::{AccountMeta, Instruction},
  program_error::ProgramError,
  pubkey::Pubkey,
};
use std::mem::size_of;

pub struct ISRC20 {}

impl ISRC20 {
  pub fn transfer(
    program_id: Pubkey,
    owner: Pubkey,
    token_acc: Pubkey,
    src_acc: Pubkey,
    dst_acc: Pubkey,
    amount: u64,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    data.push(3);
    data.extend_from_slice(&amount.to_le_bytes());
    // Build accounts
    let mut accounts = Vec::with_capacity(4);
    accounts.push(AccountMeta::new_readonly(owner, true));
    accounts.push(AccountMeta::new_readonly(token_acc, false));
    accounts.push(AccountMeta::new(src_acc, false));
    accounts.push(AccountMeta::new(dst_acc, false));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }
}
