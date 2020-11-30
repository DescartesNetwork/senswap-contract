#![cfg(feature = "program")]

use crate::error::AppError;
use crate::instruction::AppInstruction;
use crate::schema::dummy::Dummy;
use solana_sdk::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program_pack::Pack,
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
    let instruction = AppInstruction::unpack(instruction_data)?;
    match instruction {
      AppInstruction::SayHello { amount, toggle } => {
        info!("Calling SayHello function");
        let accounts_iter = &mut accounts.iter();
        let account = next_account_info(accounts_iter)?;
        if account.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        if account.try_data_len()? < mem::size_of::<u32>() {
          return Err(AppError::Overflow.into());
        }
        let mut data = Dummy::unpack(&account.data.borrow())?;
        data.amount += amount;
        data.toggle = toggle;
        Dummy::pack(data, &mut account.data.borrow_mut())?;
        Ok(())
      }
    }
  }
}
