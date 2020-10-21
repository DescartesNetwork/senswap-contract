#![cfg(feature = "program")]

use crate::error::AppError;
use crate::instruction::AppInstruction;
use byteorder::{ByteOrder, LittleEndian};
use solana_sdk::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  pubkey::Pubkey,
};
use std::mem;

pub struct Processor {}

impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    info!("Hello Rust program entrypoint");
    let instruction = AppInstruction::unpack(instruction_data)?;
    match instruction {
      AppInstruction::SayHello { amount } => {
        info!("Calling SayHello function");
        let accounts_iter = &mut accounts.iter();
        let account = next_account_info(accounts_iter)?;
        if account.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        if account.try_data_len()? < mem::size_of::<u32>() {
          return Err(AppError::Overflow.into());
        }
        let mut data = account.try_borrow_mut_data()?;
        let mut num_greets = LittleEndian::read_u32(&data);
        num_greets += amount;
        LittleEndian::write_u32(&mut data[0..], num_greets);
        Ok(())
      }
    }
  }
}

#[cfg(not(target_arch = "bpf"))]
solana_sdk::program_stubs!();

// Sanity tests
#[cfg(test)]
mod test {
  use super::*;
  use solana_sdk::clock::Epoch;

  #[test]
  fn test_sanity() {
    let program_id = Pubkey::default();
    let key = Pubkey::default();
    let mut lamports = 0;
    let mut data = vec![0; mem::size_of::<u64>()];
    LittleEndian::write_u64(&mut data, 0);
    let owner = Pubkey::default();
    let account = AccountInfo::new(
      &key,
      false,
      true,
      &mut lamports,
      &mut data,
      &owner,
      false,
      Epoch::default(),
    );
    let instruction_data: Vec<u8> = Vec::new();

    let accounts = vec![account];

    assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 0);
    process_instruction(&program_id, &accounts, &instruction_data).unwrap();
    assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 1);
    process_instruction(&program_id, &accounts, &instruction_data).unwrap();
    assert_eq!(LittleEndian::read_u64(&accounts[0].data.borrow()), 2);
  }
}
