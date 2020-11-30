#![cfg(feature = "program")]

use crate::error::AppError;
use crate::instruction::AppInstruction;
use crate::schema::{pool::Pool, token::Token};
use solana_sdk::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program_pack::Pack,
  pubkey::Pubkey,
};

pub struct Processor {}

impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = AppInstruction::unpack(instruction_data)?;
    match instruction {
      AppInstruction::PoolConstructor {} => {
        info!("Calling PoolConstructor function");
        let accounts_iter = &mut accounts.iter();
        let pool_acc = next_account_info(accounts_iter)?;
        let token_acc = next_account_info(accounts_iter)?;
        if pool_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        if !pool_acc.is_signer {
          return Err(AppError::InvalidOwner.into());
        }
        let _ = Token::unpack(&token_acc.data.borrow())?;
        let mut pool_data = Pool::unpack_unchecked(&pool_acc.data.borrow())?;
        if pool_data.is_initialized() {
          return Err(AppError::ConstructorOnce.into());
        }
        pool_data.token = *token.key;
        pool_data.reserve = 0;
        pool_data.sen = 0;
        pool_data.fee_numerator = 250;
        pool_data.fee_denominator = 100000;
        token_data.initialized = true;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
        Ok(())
      }
    }
  }
}
