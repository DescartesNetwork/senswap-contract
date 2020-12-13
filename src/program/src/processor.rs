#![cfg(feature = "program")]

use crate::error::AppError;
use crate::instruction::AppInstruction;
use crate::interfaces::isrc20::ISRC20;
use crate::schema::{account::Account, pool::Pool, sen::Sen, token::Token};
use solana_sdk::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program::invoke,
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
};

const FEE_NUMERATOR: u64 = 250;
const FEE_DENOMINATOR: u64 = 100000;

pub struct Processor {}

impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = AppInstruction::unpack(instruction_data)?;
    match instruction {
      AppInstruction::PoolConstructor { reserve, sen } => {
        info!("Calling PoolConstructor function");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let treasury_acc = next_account_info(accounts_iter)?;
        let sen_acc = next_account_info(accounts_iter)?;
        let token_acc = next_account_info(accounts_iter)?;
        let token_program_id = next_account_info(accounts_iter)?;
        if pool_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        if !caller.is_signer || !pool_acc.is_signer || !treasury_acc.is_signer || !sen_acc.is_signer
        {
          return Err(AppError::InvalidOwner.into());
        }
        let _ = Token::unpack(&token_acc.data.borrow())?;
        let mut pool_data = Pool::unpack_unchecked(&pool_acc.data.borrow())?;
        let treasury_data = Account::unpack_unchecked(&treasury_acc.data.borrow())?;
        let mut sen_data = Sen::unpack_unchecked(&sen_acc.data.borrow())?;
        if pool_data.is_initialized() || treasury_data.is_initialized() || sen_data.is_initialized()
        {
          return Err(AppError::ConstructorOnce.into());
        }

        let ix = ISRC20::account_constructor(
          *token_program_id.key,
          *pool_acc.key,
          *token_acc.key,
          *treasury_acc.key,
        )?;
        invoke(
          &ix,
          &[
            token_program_id.clone(),
            pool_acc.clone(),
            token_acc.clone(),
            treasury_acc.clone(),
          ],
        )?;

        pool_data.token = *token_acc.key;
        pool_data.reserve = reserve;
        pool_data.sen = sen;
        pool_data.fee_numerator = FEE_NUMERATOR;
        pool_data.fee_denominator = FEE_DENOMINATOR;
        pool_data.initialized = true;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

        sen_data.owner = *caller.key;
        sen_data.pool = *pool_acc.key;
        sen_data.sen = sen;
        sen_data.initialized = true;
        Sen::pack(sen_data, &mut sen_acc.data.borrow_mut())?;

        Ok(())
      }
    }
  }
}
