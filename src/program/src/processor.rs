use crate::error::AppError;
use crate::helper::oracle::Oracle;
use crate::instruction::AppInstruction;
use crate::interfaces::isplt::ISPLT;
use crate::schema::{account::Account, lpt::LPT, pool::Pool};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program::{invoke, invoke_signed},
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
};

///
/// fee = 250/100000 = 0.25%
///
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
      AppInstruction::InitializePool { reserve, lpt } => {
        info!("Calling InitializePool function");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let treasury_acc = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;
        let src_acc = next_account_info(accounts_iter)?;
        let mint_acc = next_account_info(accounts_iter)?;
        let treasurer_acc = next_account_info(accounts_iter)?;
        let splt_program = next_account_info(accounts_iter)?;
        let sysvar_rent_acc = next_account_info(accounts_iter)?;
        if pool_acc.owner != program_id || lpt_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
        let treasurer_key = Pubkey::create_program_address(&seed, program_id)?;
        if !caller.is_signer
          || !pool_acc.is_signer
          || !treasury_acc.is_signer
          || !lpt_acc.is_signer
          || treasurer_key != *treasurer_acc.key
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
        let src_data = Account::unpack(&src_acc.data.borrow())?;
        if src_data.mint != *mint_acc.key {
          return Err(AppError::IncorrectTokenId.into());
        }
        if reserve == 0 || lpt == 0 {
          return Err(AppError::ZeroValue.into());
        }

        // Account Constructor
        let ix_initialize_account = ISPLT::initialize_account(
          *treasury_acc.key,
          *mint_acc.key,
          *treasurer_acc.key,
          *sysvar_rent_acc.key,
          *splt_program.key,
        )?;
        invoke_signed(
          &ix_initialize_account,
          &[
            treasury_acc.clone(),
            mint_acc.clone(),
            treasurer_acc.clone(),
            sysvar_rent_acc.clone(),
            splt_program.clone(),
          ],
          &[&seed],
        )?;

        // Deposit token
        let ix_transfer = ISPLT::transfer(
          reserve,
          *src_acc.key,
          *treasury_acc.key,
          *caller.key,
          *splt_program.key,
        )?;
        invoke(
          &ix_transfer,
          &[
            src_acc.clone(),
            treasury_acc.clone(),
            caller.clone(),
            splt_program.clone(),
          ],
        )?;

        // Add pool data
        pool_data.mint = *mint_acc.key;
        pool_data.treasury = *treasury_acc.key;
        pool_data.reserve = reserve;
        pool_data.lpt = lpt;
        pool_data.fee_numerator = FEE_NUMERATOR;
        pool_data.fee_denominator = FEE_DENOMINATOR;
        pool_data.is_initialized = true;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
        // Add lpt data
        lpt_data.owner = *caller.key;
        lpt_data.pool = *pool_acc.key;
        lpt_data.lpt = lpt;
        lpt_data.is_initialized = true;
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
        let splt_program = next_account_info(accounts_iter)?;
        if pool_acc.owner != program_id || lpt_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        if !caller.is_signer {
          return Err(AppError::InvalidOwner.into());
        }
        let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
        let mut lpt_data = LPT::unpack_unchecked(&lpt_acc.data.borrow())?;
        if pool_data.treasury != *treasury_acc.key {
          return Err(AppError::UnmatchedPool.into());
        }
        if lpt_data.is_initialized() {
          if lpt_data.pool != *pool_acc.key {
            return Err(AppError::UnmatchedPool.into());
          }
        } else {
          if !lpt_acc.is_signer {
            return Err(AppError::InvalidOwner.into());
          }
        }
        if reserve == 0 {
          return Err(AppError::ZeroValue.into());
        }

        // Deposit token
        let ix_transfer = ISPLT::transfer(
          reserve,
          *src_acc.key,
          *treasury_acc.key,
          *caller.key,
          *splt_program.key,
        )?;
        invoke(
          &ix_transfer,
          &[
            src_acc.clone(),
            treasury_acc.clone(),
            caller.clone(),
            splt_program.clone(),
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

        // Update or Initialize lpt data
        if lpt_data.is_initialized() {
          lpt_data.lpt = lpt_data
            .lpt
            .checked_add(paid_lpt)
            .ok_or(AppError::Overflow)?;
          LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;
        } else {
          lpt_data.owner = *caller.key;
          lpt_data.pool = *pool_acc.key;
          lpt_data.lpt = paid_lpt;
          lpt_data.is_initialized = true;
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
        let treasurer_acc = next_account_info(accounts_iter)?;
        let splt_program = next_account_info(accounts_iter)?;
        if pool_acc.owner != program_id || lpt_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
        let treasurer_key = Pubkey::create_program_address(&seed, program_id)?;
        if !caller.is_signer || treasurer_key != *treasurer_acc.key {
          return Err(AppError::InvalidOwner.into());
        }
        let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
        let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
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

        // Update lpt data
        lpt_data.lpt = lpt_data.lpt.checked_sub(lpt).ok_or(AppError::Overflow)?;
        LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;
        // Update pool
        pool_data.reserve = pool_data
          .reserve
          .checked_sub(paid_reserve)
          .ok_or(AppError::Overflow)?;
        pool_data.lpt = pool_data.lpt.checked_sub(lpt).ok_or(AppError::Overflow)?;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

        // Withdraw token
        let ix_transfer = ISPLT::transfer(
          paid_reserve,
          *treasury_acc.key,
          *dst_acc.key,
          *treasurer_acc.key,
          *splt_program.key,
        )?;
        invoke_signed(
          &ix_transfer,
          &[
            treasury_acc.clone(),
            dst_acc.clone(),
            treasurer_acc.clone(),
            splt_program.clone(),
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
        let ask_pool_acc = next_account_info(accounts_iter)?;
        let ask_treasury_acc = next_account_info(accounts_iter)?;
        let dst_acc = next_account_info(accounts_iter)?;
        let ask_treasurer_acc = next_account_info(accounts_iter)?;
        let splt_program = next_account_info(accounts_iter)?;
        if bid_pool_acc.owner != program_id || ask_pool_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        let seed: &[&[_]] = &[&ask_pool_acc.key.to_bytes()[..]];
        let ask_treasurer_key = Pubkey::create_program_address(&seed, program_id)?;
        if !caller.is_signer || ask_treasurer_key != *ask_treasurer_acc.key {
          return Err(AppError::InvalidOwner.into());
        }
        let mut bid_pool_data = Pool::unpack(&bid_pool_acc.data.borrow())?;
        let mut ask_pool_data = Pool::unpack(&ask_pool_acc.data.borrow())?;
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
        let new_ask_reserve_without_fee = Oracle::new_ask_reserve_without_fee(
          bid_pool_data.reserve,
          new_bid_reserve,
          bid_pool_data.lpt,
          ask_pool_data.reserve,
          ask_pool_data.lpt,
        )
        .ok_or(AppError::Overflow)?;
        let paid_amount_without_fee = ask_pool_data
          .reserve
          .checked_sub(new_ask_reserve_without_fee)
          .ok_or(AppError::Overflow)?;
        // Apply fee
        let paid_amount_with_fee = (ask_pool_data.fee_denominator as u128)
          .checked_sub(ask_pool_data.fee_numerator as u128)
          .ok_or(AppError::Overflow)?
          .checked_mul(paid_amount_without_fee as u128)
          .ok_or(AppError::Overflow)?
          .checked_div(ask_pool_data.fee_denominator as u128)
          .ok_or(AppError::Overflow)? as u64;
        let new_ask_reserve_with_fee = ask_pool_data
          .reserve
          .checked_sub(paid_amount_with_fee)
          .ok_or(AppError::Overflow)?;

        // Transfer bid
        let ix_transfer = ISPLT::transfer(
          amount,
          *src_acc.key,
          *bid_treasury_acc.key,
          *caller.key,
          *splt_program.key,
        )?;
        invoke(
          &ix_transfer,
          &[
            src_acc.clone(),
            bid_treasury_acc.clone(),
            caller.clone(),
            splt_program.clone(),
          ],
        )?;
        bid_pool_data.reserve = new_bid_reserve;
        Pool::pack(bid_pool_data, &mut bid_pool_acc.data.borrow_mut())?;

        // Transfer ask
        ask_pool_data.reserve = new_ask_reserve_with_fee;
        Pool::pack(ask_pool_data, &mut ask_pool_acc.data.borrow_mut())?;
        let ix_transfer = ISPLT::transfer(
          paid_amount_with_fee,
          *ask_treasury_acc.key,
          *dst_acc.key,
          *ask_treasurer_acc.key,
          *splt_program.key,
        )?;
        invoke_signed(
          &ix_transfer,
          &[
            ask_treasury_acc.clone(),
            dst_acc.clone(),
            ask_treasurer_acc.clone(),
            splt_program.clone(),
          ],
          &[&seed],
        )?;

        Ok(())
      }
    }
  }
}
