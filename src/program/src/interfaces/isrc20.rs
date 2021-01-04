use arrayref::mut_array_refs;
use solana_program::{
  instruction::{AccountMeta, Instruction},
  program_error::ProgramError,
  pubkey::Pubkey,
};
use std::mem::size_of;

pub struct ISRC20 {}

impl ISRC20 {
  pub fn token_constructor(
    program_id: Pubkey,
    deployer: Pubkey,
    token_acc: Pubkey,
    dst_acc: Pubkey,
    symbol: [char; 4],
    total_supply: u64,
    decimals: u8,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // TokenConstructor - Code 0
    data.push(0);
    let mut dst_symbol = [0; 16];
    let (first_sym, second_sym, third_sym, forth_sym) =
      mut_array_refs![&mut dst_symbol, 4, 4, 4, 4];
    symbol[0].encode_utf8(first_sym);
    symbol[1].encode_utf8(second_sym);
    symbol[2].encode_utf8(third_sym);
    symbol[3].encode_utf8(forth_sym);
    data.extend_from_slice(&dst_symbol);
    data.extend_from_slice(&total_supply.to_le_bytes());
    data.extend_from_slice(&decimals.to_le_bytes());
    // Build accounts
    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new_readonly(deployer, true));
    accounts.push(AccountMeta::new(token_acc, true));
    accounts.push(AccountMeta::new(dst_acc, true));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }

  pub fn account_constructor(
    program_id: Pubkey,
    caller: Pubkey,
    token_acc: Pubkey,
    target_acc: Pubkey,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // AccountConstructor - Code 1
    data.push(1);
    // Build accounts
    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new_readonly(caller, true));
    accounts.push(AccountMeta::new_readonly(token_acc, false));
    accounts.push(AccountMeta::new(target_acc, true));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }

  pub fn delegation_constructor(
    program_id: Pubkey,
    amount: u64,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // DelegationConstructor - Code 2
    data.push(2);
    data.extend_from_slice(&amount.to_le_bytes());
    // Build accounts
    let accounts = Vec::with_capacity(0);
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }

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
    // Transfer - Code 3
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

  pub fn approve(
    program_id: Pubkey,
    owner: Pubkey,
    token_acc: Pubkey,
    delegation_acc: Pubkey,
    src_acc: Pubkey,
    dlg_acc: Pubkey,
    amount: u64,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // Transfer - Code 4
    data.push(4);
    data.extend_from_slice(&amount.to_le_bytes());
    // Build accounts
    let mut accounts = Vec::with_capacity(5);
    accounts.push(AccountMeta::new_readonly(owner, true));
    accounts.push(AccountMeta::new_readonly(token_acc, false));
    accounts.push(AccountMeta::new(delegation_acc, true));
    accounts.push(AccountMeta::new_readonly(src_acc, false));
    accounts.push(AccountMeta::new_readonly(dlg_acc, false));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }

  pub fn transfer_from(
    program_id: Pubkey,
    dlg_acc: Pubkey,
    token_acc: Pubkey,
    delegation_acc: Pubkey,
    src_acc: Pubkey,
    dst_acc: Pubkey,
    amount: u64,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // TransferFrom - Code 5
    data.push(5);
    data.extend_from_slice(&amount.to_le_bytes());
    // Build accounts
    let mut accounts = Vec::with_capacity(5);
    accounts.push(AccountMeta::new_readonly(dlg_acc, true));
    accounts.push(AccountMeta::new_readonly(token_acc, false));
    accounts.push(AccountMeta::new(delegation_acc, false));
    accounts.push(AccountMeta::new(src_acc, false));
    accounts.push(AccountMeta::new(dst_acc, false));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }

  pub fn increase_approval(
    program_id: Pubkey,
    owner: Pubkey,
    token_acc: Pubkey,
    delegation_acc: Pubkey,
    amount: u64,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // IncreaseApproval - Code 6
    data.push(6);
    data.extend_from_slice(&amount.to_le_bytes());
    // Build accounts
    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new_readonly(owner, true));
    accounts.push(AccountMeta::new_readonly(token_acc, false));
    accounts.push(AccountMeta::new(delegation_acc, false));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }

  pub fn decrease_approval(
    program_id: Pubkey,
    owner: Pubkey,
    token_acc: Pubkey,
    delegation_acc: Pubkey,
    amount: u64,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // DecreaseApproval - Code 7
    data.push(7);
    data.extend_from_slice(&amount.to_le_bytes());
    // Build accounts
    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new_readonly(owner, true));
    accounts.push(AccountMeta::new_readonly(token_acc, false));
    accounts.push(AccountMeta::new(delegation_acc, false));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }

  pub fn revoke(
    program_id: Pubkey,
    owner: Pubkey,
    token_acc: Pubkey,
    delegation_acc: Pubkey,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // Revoke - Code 8
    data.push(8);
    // Build accounts
    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new_readonly(owner, true));
    accounts.push(AccountMeta::new_readonly(token_acc, false));
    accounts.push(AccountMeta::new(delegation_acc, false));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }

  pub fn account_destruction(
    program_id: Pubkey,
    owner: Pubkey,
    token_acc: Pubkey,
    target_acc: Pubkey,
  ) -> Result<Instruction, ProgramError> {
    // Build data
    let mut data = Vec::with_capacity(size_of::<Self>());
    // AccountDestruction - Code 9
    data.push(9);
    // Build accounts
    let mut accounts = Vec::with_capacity(3);
    accounts.push(AccountMeta::new_readonly(owner, true));
    accounts.push(AccountMeta::new_readonly(token_acc, false));
    accounts.push(AccountMeta::new(target_acc, false));
    // Return
    Ok(Instruction {
      program_id,
      accounts,
      data,
    })
  }
}
