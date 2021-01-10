use crate::error::AppError;
use crate::instruction::AppInstruction;
use crate::interfaces::isrc20::ISRC20;
use crate::schema::{account::Account, pool::Pool, lpt::LPT};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program::{invoke, invoke_signed},
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
      AppInstruction::PoolConstructor { reserve, lpt } => {
        info!("Calling PoolConstructor function");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let treasury_acc = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;
        let src_acc = next_account_info(accounts_iter)?;
        let token_acc = next_account_info(accounts_iter)?;
        let token_owner_acc = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        if pool_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
        let token_owner_key = Pubkey::create_program_address(&seed, program_id)?;
        if !caller.is_signer
          || !pool_acc.is_signer
          || !treasury_acc.is_signer
          || !lpt_acc.is_signer
          || token_owner_key != *token_owner_acc.key
        {
          return Err(AppError::InvalidOwner.into());
        }
        let mut pool_data = Pool::unpack_unchecked(&pool_acc.data.borrow())?;
        let treasury_data = Account::unpack_unchecked(&treasury_acc.data.borrow())?;
        let mut lpt_data = LPT::unpack_unchecked(&lpt_acc.data.borrow())?;
        if pool_data.is_initialized() || treasury_data.is_initialized() || lpt_data.is_initialized()
        {
          return Err(AppError::ConstructorOnce.into());
        }
        if reserve == 0 || lpt == 0 {
          return Err(AppError::ZeroValue.into());
        }

        // Account Constructor
        let ix_account_constructor = ISRC20::account_constructor(
          *token_program.key,
          token_owner_key,
          *token_acc.key,
          *treasury_acc.key,
        )?;
        invoke_signed(
          &ix_account_constructor,
          &[
            token_program.clone(),
            token_owner_acc.clone(),
            token_acc.clone(),
            treasury_acc.clone(),
          ],
          &[&seed],
        )?;

        // Deposit token
        let ix_transfer = ISRC20::transfer(
          *token_program.key,
          *caller.key,
          *token_acc.key,
          *src_acc.key,
          *treasury_acc.key,
          reserve,
        )?;
        invoke(
          &ix_transfer,
          &[
            token_program.clone(),
            caller.clone(),
            token_acc.clone(),
            src_acc.clone(),
            treasury_acc.clone(),
          ],
        )?;

        // Add pool data
        pool_data.token = *token_acc.key;
        pool_data.treasury = *treasury_acc.key;
        pool_data.reserve = reserve;
        pool_data.lpt = lpt;
        pool_data.fee_numerator = FEE_NUMERATOR;
        pool_data.fee_denominator = FEE_DENOMINATOR;
        pool_data.initialized = true;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
        // Add lpt data
        lpt_data.owner = *caller.key;
        lpt_data.pool = *pool_acc.key;
        lpt_data.lpt = lpt;
        lpt_data.initialized = true;
        LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

        Ok(())
      }

      AppInstruction::AddLiquidity { reserve } => {
        info!("Calling AddLiquidity function");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let treasury_acc = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;
        let src_acc = next_account_info(accounts_iter)?;
        let token_acc = next_account_info(accounts_iter)?;
        let token_owner_acc = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        if pool_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
        let token_owner_key = Pubkey::create_program_address(&seed, program_id)?;
        if !caller.is_signer || token_owner_key != *token_owner_acc.key {
          return Err(AppError::InvalidOwner.into());
        }
        let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
        let treasury_data = Account::unpack(&treasury_acc.data.borrow())?;
        let mut lpt_data = LPT::unpack_unchecked(&lpt_acc.data.borrow())?;
        if pool_data.token != *token_acc.key || treasury_data.token != *token_acc.key {
          return Err(AppError::IncorrectTokenId.into());
        }
        if pool_data.treasury != *treasury_acc.key {
          return Err(AppError::UnmatchedPool.into());
        }
        if reserve == 0 {
          return Err(AppError::ZeroValue.into());
        }

        // Deposit token
        let ix_transfer = ISRC20::transfer(
          *token_program.key,
          *caller.key,
          *token_acc.key,
          *src_acc.key,
          *treasury_acc.key,
          reserve,
        )?;
        invoke(
          &ix_transfer,
          &[
            token_program.clone(),
            caller.clone(),
            token_acc.clone(),
            src_acc.clone(),
            treasury_acc.clone(),
          ],
        )?;

        // Compute corresponding paid-back lpt
        let paid_lpt = (pool_data.lpt as u128)
          .checked_mul(reserve as u128)
          .ok_or(AppError::Overflow)?
          .checked_div(pool_data.reserve as u128)
          .ok_or(AppError::Overflow)? as u64;

        // Update pool
        pool_data.reserve = pool_data
          .reserve
          .checked_add(reserve)
          .ok_or(AppError::Overflow)?;
        pool_data.lpt = pool_data
          .lpt
          .checked_add(paid_lpt)
          .ok_or(AppError::Overflow)?;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

        // Update or Add lpt data
        if lpt_data.is_initialized() {
          if lpt_data.pool != *pool_acc.key {
            return Err(AppError::UnmatchedPool.into());
          }
          lpt_data.lpt = lpt_data
            .lpt
            .checked_add(paid_lpt)
            .ok_or(AppError::Overflow)?;
          LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;
        } else {
          if !lpt_acc.is_signer {
            return Err(AppError::InvalidOwner.into());
          }
          lpt_data.owner = *caller.key;
          lpt_data.pool = *pool_acc.key;
          lpt_data.lpt = paid_lpt;
          lpt_data.initialized = true;
          LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;
        }

        Ok(())
      }

      AppInstruction::RemoveLiquidity { lpt } => {
        info!("Calling RemoveLiquidity function");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let treasury_acc = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;
        let dst_acc = next_account_info(accounts_iter)?;
        let token_acc = next_account_info(accounts_iter)?;
        let token_owner_acc = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        if pool_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
        let token_owner_key = Pubkey::create_program_address(&seed, program_id)?;
        if !caller.is_signer || token_owner_key != *token_owner_acc.key {
          return Err(AppError::InvalidOwner.into());
        }
        let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
        let treasury_data = Account::unpack(&treasury_acc.data.borrow())?;
        let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
        if pool_data.token != *token_acc.key || treasury_data.token != *token_acc.key {
          return Err(AppError::IncorrectTokenId.into());
        }
        if pool_data.treasury != *treasury_acc.key || lpt_data.pool != *pool_acc.key {
          return Err(AppError::UnmatchedPool.into());
        }
        if lpt == 0 {
          return Err(AppError::ZeroValue.into());
        }

        // Compute corresponding paid-back reserve
        let paid_reserve = (pool_data.reserve as u128)
          .checked_mul(lpt as u128)
          .ok_or(AppError::Overflow)?
          .checked_div(pool_data.lpt as u128)
          .ok_or(AppError::Overflow)? as u64;

        // Update pool
        pool_data.reserve = pool_data
          .reserve
          .checked_sub(paid_reserve)
          .ok_or(AppError::Overflow)?;
        pool_data.lpt = pool_data.lpt.checked_sub(lpt).ok_or(AppError::Overflow)?;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
        // Update lpt data
        lpt_data.lpt = lpt_data.lpt.checked_sub(lpt).ok_or(AppError::Overflow)?;
        LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

        // Withdraw token
        let ix_transfer = ISRC20::transfer(
          *token_program.key,
          *token_owner_acc.key,
          *token_acc.key,
          *treasury_acc.key,
          *dst_acc.key,
          paid_reserve,
        )?;
        invoke_signed(
          &ix_transfer,
          &[
            token_program.clone(),
            token_owner_acc.clone(),
            token_acc.clone(),
            treasury_acc.clone(),
            dst_acc.clone(),
          ],
          &[&seed],
        )?;

        Ok(())
      }

      AppInstruction::Swap { amount } => {
        info!("Calling Swap function");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let bid_pool_acc = next_account_info(accounts_iter)?;
        let bid_treasury_acc = next_account_info(accounts_iter)?;
        let src_acc = next_account_info(accounts_iter)?;
        let bid_token_acc = next_account_info(accounts_iter)?;
        let ask_pool_acc = next_account_info(accounts_iter)?;
        let ask_treasury_acc = next_account_info(accounts_iter)?;
        let dst_acc = next_account_info(accounts_iter)?;
        let ask_token_acc = next_account_info(accounts_iter)?;
        let ask_token_owner_acc = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        if bid_pool_acc.owner != program_id || ask_pool_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        let seed: &[&[_]] = &[&ask_pool_acc.key.to_bytes()[..]];
        let ask_token_owner_key = Pubkey::create_program_address(&seed, program_id)?;
        if !caller.is_signer || ask_token_owner_key != *ask_token_owner_acc.key {
          return Err(AppError::InvalidOwner.into());
        }
        let mut bid_pool_data = Pool::unpack(&bid_pool_acc.data.borrow())?;
        let bid_treasury_data = Account::unpack(&bid_treasury_acc.data.borrow())?;
        let mut ask_pool_data = Pool::unpack(&ask_pool_acc.data.borrow())?;
        let ask_treasury_data = Account::unpack(&ask_treasury_acc.data.borrow())?;
        if bid_pool_data.token != *bid_token_acc.key
          || ask_pool_data.token != *ask_token_acc.key
          || bid_treasury_data.token != *bid_token_acc.key
          || ask_treasury_data.token != *ask_token_acc.key
        {
          return Err(AppError::IncorrectTokenId.into());
        }
        if bid_pool_data.treasury != *bid_treasury_acc.key
          || ask_pool_data.treasury != *ask_treasury_acc.key
        {
          return Err(AppError::UnmatchedPool.into());
        }
        if amount == 0 {
          return Err(AppError::ZeroValue.into());
        }
        if *bid_pool_acc.key == *ask_pool_acc.key {
          return Ok(());
        }

        // Compute new state
        let new_bid_reserve = bid_pool_data
          .reserve
          .checked_add(amount)
          .ok_or(AppError::Overflow)?;
        let new_ask_reserve_no_fee = (bid_pool_data.reserve as u128)
          .checked_mul(ask_pool_data.reserve as u128)
          .ok_or(AppError::Overflow)?
          .checked_div(new_bid_reserve as u128)
          .ok_or(AppError::Overflow)? as u64;
        let paid_amount_no_fee = ask_pool_data
          .reserve
          .checked_sub(new_ask_reserve_no_fee)
          .ok_or(AppError::Overflow)?;
        // Apply fee
        let paid_amount_after_fee = (ask_pool_data.fee_denominator as u128)
          .checked_sub(ask_pool_data.fee_numerator as u128)
          .ok_or(AppError::Overflow)?
          .checked_mul(paid_amount_no_fee as u128)
          .ok_or(AppError::Overflow)?
          .checked_div(ask_pool_data.fee_denominator as u128)
          .ok_or(AppError::Overflow)? as u64;
        let new_ask_reserve_after_fee = ask_pool_data
          .reserve
          .checked_sub(paid_amount_after_fee)
          .ok_or(AppError::Overflow)?;

        // Transfer bid
        let ix_transfer = ISRC20::transfer(
          *token_program.key,
          *caller.key,
          *bid_token_acc.key,
          *src_acc.key,
          *bid_treasury_acc.key,
          amount,
        )?;
        invoke(
          &ix_transfer,
          &[
            token_program.clone(),
            caller.clone(),
            bid_token_acc.clone(),
            src_acc.clone(),
            bid_treasury_acc.clone(),
          ],
        )?;
        bid_pool_data.reserve = new_bid_reserve;
        Pool::pack(bid_pool_data, &mut bid_pool_acc.data.borrow_mut())?;

        // Transfer ask
        ask_pool_data.reserve = new_ask_reserve_after_fee;
        Pool::pack(ask_pool_data, &mut ask_pool_acc.data.borrow_mut())?;
        let ix_transfer = ISRC20::transfer(
          *token_program.key,
          *ask_token_owner_acc.key,
          *ask_token_acc.key,
          *ask_treasury_acc.key,
          *dst_acc.key,
          paid_amount_after_fee,
        )?;
        invoke_signed(
          &ix_transfer,
          &[
            token_program.clone(),
            ask_token_owner_acc.clone(),
            ask_token_acc.clone(),
            ask_treasury_acc.clone(),
            dst_acc.clone(),
          ],
          &[&seed],
        )?;

        Ok(())
      }
    }
  }
}
