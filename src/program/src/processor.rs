use crate::error::AppError;
use crate::helper::oracle::Oracle;
use crate::instruction::AppInstruction;
use crate::interfaces::isplt::ISPLT;
use crate::schema::{
  account::Account,
  lpt::LPT,
  network::Network,
  pool::{Pool, PoolState},
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program::{invoke, invoke_signed},
  program_pack::{IsInitialized, Pack},
  pubkey::Pubkey,
};

///
/// consensus = 666666667/1000000000 = 2/3
/// fee = 2500000/1000000000 = 0.25%
///
const CONSENSUS: u64 = 666666667;
const FEE: u64 = 2500000;
const DECIMALS: u64 = 1000000000;

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
        let network_acc = next_account_info(accounts_iter)?;
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
        let mut network_data = Network::unpack_unchecked(&network_acc.data.borrow())?;
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

        // Intialize Network
        let mut pool_state = PoolState::Initialized;
        if !network_data.is_initialized() {
          if !network_acc.is_signer {
            return Err(AppError::InvalidOwner.into());
          }
          pool_state = PoolState::Approved;
          network_data.volume = lpt;
          network_data.is_initialized = true;
          Network::pack(network_data, &mut network_acc.data.borrow_mut())?;
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
        pool_data.network = *network_acc.key;
        pool_data.mint = *mint_acc.key;
        pool_data.treasury = *treasury_acc.key;
        pool_data.reserve = reserve;
        pool_data.lpt = lpt;
        pool_data.fee = FEE;
        pool_data.state = pool_state;
        pool_data.voted = lpt;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
        // Add lpt data
        lpt_data.owner = *caller.key;
        lpt_data.pool = *pool_acc.key;
        lpt_data.lpt = lpt;
        lpt_data.voting = *pool_acc.key;
        lpt_data.is_initialized = true;
        LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

        Ok(())
      }

      AppInstruction::InitializeLPT {} => {
        info!("Calling InitializeLPTfunction");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;

        if pool_acc.owner != program_id || lpt_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        if !caller.is_signer || !lpt_acc.is_signer {
          return Err(AppError::InvalidOwner.into());
        }
        let mut lpt_data = LPT::unpack_unchecked(&lpt_acc.data.borrow())?;
        if lpt_data.is_initialized() {
          return Err(AppError::ConstructorOnce.into());
        }

        lpt_data.owner = *caller.key;
        lpt_data.pool = *pool_acc.key;
        lpt_data.lpt = 0;
        lpt_data.voting = *pool_acc.key;
        lpt_data.is_initialized = true;
        LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

        Ok(())
      }

      AppInstruction::AddLiquidity { reserve } => {
        info!("Calling AddLiquidity function");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let network_acc = next_account_info(accounts_iter)?;
        let prev_pool_acc = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let treasury_acc = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;
        let src_acc = next_account_info(accounts_iter)?;
        let splt_program = next_account_info(accounts_iter)?;
        if network_acc.owner != program_id
          || prev_pool_acc.owner != program_id
          || pool_acc.owner != program_id
          || lpt_acc.owner != program_id
        {
          return Err(AppError::IncorrectProgramId.into());
        }
        if !caller.is_signer {
          return Err(AppError::InvalidOwner.into());
        }
        let mut network_data = Network::unpack(&network_acc.data.borrow())?;
        let mut prev_pool_data = Pool::unpack(&prev_pool_acc.data.borrow())?;
        let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
        let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
        if pool_data.treasury != *treasury_acc.key || lpt_data.pool != *pool_acc.key {
          return Err(AppError::UnmatchedPool.into());
        }
        if lpt_data.voting != *prev_pool_acc.key
          || prev_pool_data.network != *network_acc.key
          || pool_data.network != *network_acc.key
        {
          return Err(AppError::IncorrectNetworkId.into());
        }
        if reserve == 0 {
          return Err(AppError::ZeroValue.into());
        }

        // Compute corresponding paid-back lpt
        let paid_lpt = pool_data
          .lpt
          .checked_mul(reserve as u128)
          .ok_or(AppError::Overflow)?
          .checked_div(pool_data.reserve as u128)
          .ok_or(AppError::Overflow)?;

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

        // Update pool data
        pool_data.reserve = pool_data
          .reserve
          .checked_add(reserve)
          .ok_or(AppError::Overflow)?;
        pool_data.lpt = pool_data
          .lpt
          .checked_add(paid_lpt)
          .ok_or(AppError::Overflow)?;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
        // Update previous pool data
        prev_pool_data.voted = pool_data
          .voted
          .checked_add(paid_lpt)
          .ok_or(AppError::Overflow)?;
        Pool::pack(prev_pool_data, &mut prev_pool_acc.data.borrow_mut())?;
        // If the lpt's pool is approved
        // It would effect to the vote, then check it
        if pool_data.is_approved() {
          // Update network data
          network_data.volume = network_data
            .volume
            .checked_add(paid_lpt)
            .ok_or(AppError::Overflow)?;
          Network::pack(network_data, &mut network_acc.data.borrow_mut())?;
          // Checked approve
          Self::checked_approve(&network_acc, &prev_pool_acc)?;
        }
        // Update lpt data
        lpt_data.lpt = lpt_data
          .lpt
          .checked_add(paid_lpt)
          .ok_or(AppError::Overflow)?;
        LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

        Ok(())
      }

      AppInstruction::RemoveLiquidity { lpt } => {
        info!("Calling RemoveLiquidity function");
        let accounts_iter = &mut accounts.iter();
        let caller = next_account_info(accounts_iter)?;
        let network_acc = next_account_info(accounts_iter)?;
        let prev_pool_acc = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let treasury_acc = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;
        let dst_acc = next_account_info(accounts_iter)?;
        let treasurer_acc = next_account_info(accounts_iter)?;
        let splt_program = next_account_info(accounts_iter)?;
        if network_acc.owner != program_id
          || prev_pool_acc.owner != program_id
          || pool_acc.owner != program_id
          || lpt_acc.owner != program_id
        {
          return Err(AppError::IncorrectProgramId.into());
        }
        let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
        let treasurer_key = Pubkey::create_program_address(&seed, program_id)?;
        if !caller.is_signer || treasurer_key != *treasurer_acc.key {
          return Err(AppError::InvalidOwner.into());
        }
        let mut network_data = Network::unpack(&network_acc.data.borrow())?;
        let mut prev_pool_data = Pool::unpack(&prev_pool_acc.data.borrow())?;
        let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
        let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
        if pool_data.treasury != *treasury_acc.key || lpt_data.pool != *pool_acc.key {
          return Err(AppError::UnmatchedPool.into());
        }
        if lpt_data.voting != *prev_pool_acc.key
          || pool_data.network != *network_acc.key
          || prev_pool_data.network != *network_acc.key
        {
          return Err(AppError::IncorrectNetworkId.into());
        }
        if lpt == 0 {
          return Err(AppError::ZeroValue.into());
        }
        if lpt_data.lpt < lpt {
          return Err(AppError::InsufficientFunds.into());
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
        // Update previous pool data
        prev_pool_data.voted = prev_pool_data
          .voted
          .checked_sub(lpt)
          .ok_or(AppError::Overflow)?;
        Pool::pack(prev_pool_data, &mut prev_pool_acc.data.borrow_mut())?;
        // Update pool data
        pool_data.reserve = pool_data
          .reserve
          .checked_sub(paid_reserve)
          .ok_or(AppError::Overflow)?;
        pool_data.lpt = pool_data.lpt.checked_sub(lpt).ok_or(AppError::Overflow)?;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
        // If the lpt's pool is approved
        // It would effect to the vote, then check it
        if pool_data.is_approved() {
          // Update network data
          network_data.volume = network_data
            .volume
            .checked_sub(lpt)
            .ok_or(AppError::Overflow)?;
          Network::pack(network_data, &mut network_acc.data.borrow_mut())?;
        }

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

        // Terminate pool if LPT down to 0
        if pool_data.lpt == 0 {
          // Close treasury
          let ix_close_account = ISPLT::close_account(
            *treasury_acc.key,
            *caller.key,
            *treasurer_acc.key,
            *splt_program.key,
          )?;
          invoke_signed(
            &ix_close_account,
            &[
              treasury_acc.clone(),
              caller.clone(),
              treasurer_acc.clone(),
              splt_program.clone(),
            ],
            &[&seed],
          )?;
          // Close pool
          let dst_lamports = caller.lamports();
          **caller.lamports.borrow_mut() = dst_lamports
            .checked_add(pool_acc.lamports())
            .ok_or(AppError::Overflow)?;
          **pool_acc.lamports.borrow_mut() = 0;
        }

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
        if !bid_pool_data.is_approved()
          || !ask_pool_data.is_approved()
          || bid_pool_data.network != ask_pool_data.network
        {
          return Err(AppError::IncorrectNetworkId.into());
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
        let new_ask_reserve_without_fee = Oracle::curve(
          new_bid_reserve,
          bid_pool_data.reserve,
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
        let paid_amount_with_fee = ((DECIMALS - ask_pool_data.fee) as u128)
          .checked_mul(paid_amount_without_fee as u128)
          .ok_or(AppError::Overflow)?
          .checked_div(DECIMALS as u128)
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

      AppInstruction::Vote {} => {
        let accounts_iter = &mut accounts.iter();
        let owner = next_account_info(accounts_iter)?;
        let network_acc = next_account_info(accounts_iter)?;
        let prev_pool_acc = next_account_info(accounts_iter)?;
        let pool_acc = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;

        if network_acc.owner != program_id
          || pool_acc.owner != program_id
          || lpt_acc.owner != program_id
        {
          return Err(AppError::IncorrectProgramId.into());
        }
        let _ = Network::unpack(&network_acc.data.borrow())?;
        let mut prev_pool_data = Pool::unpack(&prev_pool_acc.data.borrow())?;
        let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
        let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
        if !owner.is_signer || *owner.key != lpt_data.owner {
          return Err(AppError::InvalidOwner.into());
        }
        if pool_data.network != *network_acc.key || !pool_data.is_approved() {
          return Err(AppError::IncorrectNetworkId.into());
        }
        if lpt_data.voting != *prev_pool_acc.key {
          return Err(AppError::UnmatchedPool.into());
        }
        if lpt_data.lpt == 0 {
          return Err(AppError::ZeroValue.into());
        }
        if *pool_acc.key == *prev_pool_acc.key {
          return Ok(());
        }

        // Update LPT data
        lpt_data.voting = *pool_acc.key;
        LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;
        // Update previous pool data
        prev_pool_data.voted = prev_pool_data
          .voted
          .checked_sub(lpt_data.lpt)
          .ok_or(AppError::Overflow)?;
        Pool::pack(prev_pool_data, &mut prev_pool_acc.data.borrow_mut())?;
        // Update pool data
        pool_data.voted = pool_data
          .voted
          .checked_add(lpt_data.lpt)
          .ok_or(AppError::Overflow)?;
        Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
        // Checked approve
        Self::checked_approve(&network_acc, &pool_acc)?;

        Ok(())
      }

      AppInstruction::CloseLPT {} => {
        info!("Calling CloseLPT function");
        let accounts_iter = &mut accounts.iter();
        let owner = next_account_info(accounts_iter)?;
        let lpt_acc = next_account_info(accounts_iter)?;
        let dst_acc = next_account_info(accounts_iter)?;
        if lpt_acc.owner != program_id {
          return Err(AppError::IncorrectProgramId.into());
        }
        let lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
        if !owner.is_signer || *owner.key != lpt_data.owner {
          return Err(AppError::InvalidOwner.into());
        }
        if lpt_data.lpt != 0 {
          return Err(AppError::ZeroValue.into());
        }

        let lpt_lamports = lpt_acc.lamports();
        **dst_acc.lamports.borrow_mut() = lpt_lamports
          .checked_add(dst_acc.lamports())
          .ok_or(AppError::Overflow)?;
        **lpt_acc.lamports.borrow_mut() = 0;

        Ok(())
      }
    }
  }

  pub fn checked_approve(network_acc: &AccountInfo, pool_acc: &AccountInfo) -> ProgramResult {
    let mut network_data = Network::unpack(&network_acc.data.borrow())?;
    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    if pool_data.network != *network_acc.key {
      return Err(AppError::IncorrectNetworkId.into());
    }
    if pool_data.state == PoolState::Approved {
      return Ok(());
    }
    if pool_data
      .voted
      .checked_mul(DECIMALS as u128)
      .ok_or(AppError::Overflow)?
      .checked_div(network_data.volume)
      .ok_or(AppError::Overflow)?
      >= CONSENSUS as u128
    {
      // Update pool data
      pool_data.state = PoolState::Approved;
      Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
      // Update network data
      network_data.volume = network_data
        .volume
        .checked_add(pool_data.lpt)
        .ok_or(AppError::Overflow)?;
      Network::pack(network_data, &mut network_acc.data.borrow_mut())?;
    }
    Ok(())
  }
}
