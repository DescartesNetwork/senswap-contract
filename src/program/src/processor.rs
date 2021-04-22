use crate::error::AppError;
use crate::helper::oracle::Oracle;
use crate::instruction::AppInstruction;
use crate::interfaces::xsplt::XSPLT;
use crate::schema::{
  lpt::LPT,
  pool::{Pool, PoolState},
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  info,
  program_pack::{IsInitialized, Pack},
  pubkey::{Pubkey, PubkeyError},
};

pub struct Processor {}

impl Processor {
  ///
  /// Entrypoint
  ///
  pub fn process<'a>(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'a>],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = AppInstruction::unpack(instruction_data)?;
    match instruction {
      AppInstruction::InitializePool {
        reserve_s,
        reserve_a,
        reserve_b,
      } => {
        info!("Calling InitializePool function");
        Self::initialize_pool(reserve_s, reserve_a, reserve_b, program_id, accounts)
      }

      AppInstruction::InitializeLPT {} => {
        info!("Calling InitializeLPTfunction");
        Self::initialize_lpt(program_id, accounts)
      }

      AppInstruction::AddLiquidity {
        delta_s,
        delta_a,
        delta_b,
      } => {
        info!("Calling AddLiquidity function");
        Self::add_liquidity(delta_s, delta_a, delta_b, program_id, accounts)
      }

      AppInstruction::RemoveLiquidity { lpt } => {
        info!("Calling RemoveLiquidity function");
        Self::remove_liquidity(lpt, program_id, accounts)
      }

      AppInstruction::Swap { amount } => {
        info!("Calling Swap function");
        Self::swap(amount, program_id, accounts)
      }

      AppInstruction::Transfer { lpt } => {
        info!("Calling Transfer function");
        Self::transfer(lpt, program_id, accounts)
      }

      AppInstruction::FreezePool {} => {
        info!("Calling FreezePool function");
        Self::freeze_pool(program_id, accounts)
      }

      AppInstruction::ThawPool {} => {
        info!("Calling ThawPool function");
        Self::thaw_pool(program_id, accounts)
      }

      AppInstruction::Earn { amount } => {
        info!("Calling Earn function");
        Self::earn(amount, program_id, accounts)
      }

      AppInstruction::CloseLPT {} => {
        info!("Calling CloseLPT function");
        Self::close_lpt(program_id, accounts)
      }

      AppInstruction::ClosePool {} => {
        info!("Calling ClosePool function");
        Self::close_pool(program_id, accounts)
      }

      AppInstruction::TransferOwnership {} => {
        info!("Calling TransferOwnership function");
        Self::transfer_ownership(program_id, accounts)
      }
    }
  }

  ///
  /// Controllers
  ///

  pub fn initialize_pool<'a>(
    reserve_s: u64,
    reserve_a: u64,
    reserve_b: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo<'a>],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;

    let src_s_acc = next_account_info(accounts_iter)?;
    let mint_s_acc = next_account_info(accounts_iter)?;
    let treasury_s_acc = next_account_info(accounts_iter)?;

    let src_a_acc = next_account_info(accounts_iter)?;
    let mint_a_acc = next_account_info(accounts_iter)?;
    let treasury_a_acc = next_account_info(accounts_iter)?;

    let src_b_acc = next_account_info(accounts_iter)?;
    let mint_b_acc = next_account_info(accounts_iter)?;
    let treasury_b_acc = next_account_info(accounts_iter)?;

    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    let sysvar_rent_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc, lpt_acc])?;
    Self::is_signer(&[
      owner,
      pool_acc,
      lpt_acc,
      treasury_s_acc,
      treasury_a_acc,
      treasury_b_acc,
    ])?;
    Self::safe_seed(pool_acc, treasurer, program_id)?;

    let mut pool_data = Pool::unpack_unchecked(&pool_acc.data.borrow())?;
    let mut lpt_data = LPT::unpack_unchecked(&lpt_acc.data.borrow())?;
    if pool_data.is_initialized() || lpt_data.is_initialized() {
      return Err(AppError::ConstructorOnce.into());
    }
    if reserve_s == 0 || reserve_a == 0 || reserve_b == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Initialize treasury S
    XSPLT::initialize_account(
      treasury_s_acc,
      mint_s_acc,
      treasurer,
      sysvar_rent_acc,
      splt_program,
      &[],
    )?;
    // Deposit token S
    XSPLT::transfer(
      reserve_s,
      src_s_acc,
      treasury_s_acc,
      owner,
      splt_program,
      &[],
    )?;

    // Initialize treasury A
    XSPLT::initialize_account(
      treasury_a_acc,
      mint_a_acc,
      treasurer,
      sysvar_rent_acc,
      splt_program,
      &[],
    )?;
    // Deposit token A
    XSPLT::transfer(
      reserve_a,
      src_a_acc,
      treasury_a_acc,
      owner,
      splt_program,
      &[],
    )?;

    // Initialize treasury B
    XSPLT::initialize_account(
      treasury_b_acc,
      mint_b_acc,
      treasurer,
      sysvar_rent_acc,
      splt_program,
      &[],
    )?;
    // Deposit token B
    XSPLT::transfer(
      reserve_b,
      src_b_acc,
      treasury_b_acc,
      owner,
      splt_program,
      &[],
    )?;

    // Update pool data
    pool_data.owner = *owner.key;
    pool_data.state = PoolState::Initialized;
    pool_data.mint_s = *mint_s_acc.key;
    pool_data.treasury_s = *treasury_s_acc.key;
    pool_data.reserve_s = reserve_s;
    pool_data.mint_a = *mint_a_acc.key;
    pool_data.treasury_a = *treasury_a_acc.key;
    pool_data.reserve_a = reserve_a;
    pool_data.mint_b = *mint_b_acc.key;
    pool_data.treasury_b = *treasury_b_acc.key;
    pool_data.reserve_b = reserve_b;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
    // Update lpt data
    lpt_data.owner = *owner.key;
    lpt_data.pool = *pool_acc.key;
    lpt_data.lpt = reserve_s;
    lpt_data.is_initialized = true;
    LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn initialize_lpt(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc, lpt_acc])?;
    Self::is_signer(&[owner, lpt_acc])?;

    let mut lpt_data = LPT::unpack_unchecked(&lpt_acc.data.borrow())?;
    if lpt_data.is_initialized() {
      return Err(AppError::ConstructorOnce.into());
    }

    lpt_data.owner = *owner.key;
    lpt_data.pool = *pool_acc.key;
    lpt_data.lpt = 0;
    lpt_data.is_initialized = true;
    LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn add_liquidity<'a>(
    delta_s: u64,
    delta_a: u64,
    delta_b: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo<'a>],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;

    let src_s_acc = next_account_info(accounts_iter)?;
    let mint_s_acc = next_account_info(accounts_iter)?;
    let treasury_s_acc = next_account_info(accounts_iter)?;

    let src_a_acc = next_account_info(accounts_iter)?;
    let mint_a_acc = next_account_info(accounts_iter)?;
    let treasury_a_acc = next_account_info(accounts_iter)?;

    let src_b_acc = next_account_info(accounts_iter)?;
    let mint_b_acc = next_account_info(accounts_iter)?;
    let treasury_b_acc = next_account_info(accounts_iter)?;

    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc, lpt_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_lpt_owner(owner, lpt_acc)?;

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
    if pool_data.treasury_s != *treasury_s_acc.key
      || pool_data.treasury_a != *treasury_a_acc.key
      || pool_data.treasury_b != *treasury_b_acc.key
    {
      return Err(AppError::InvalidOwner.into());
    }
    if lpt_data.pool != *pool_acc.key {
      return Err(AppError::UnmatchedPool.into());
    }
    if reserve == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Deposit token
    XSPLT::transfer(reserve, src_acc, treasury_acc, owner, splt_program, &[])?;
    // Compute corresponding paid-back lpt
    let paid_lpt = (pool_data.lpt)
      .checked_mul(reserve as u128)
      .ok_or(AppError::Overflow)?
      .checked_div(pool_data.reserve as u128)
      .ok_or(AppError::Overflow)?;
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
    // Update lpt data
    lpt_data.lpt = lpt_data
      .lpt
      .checked_add(paid_lpt)
      .ok_or(AppError::Overflow)?;
    LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn remove_liquidity<'a>(
    lpt: u128,
    program_id: &Pubkey,
    accounts: &[AccountInfo<'a>],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let treasury_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc, lpt_acc])?;
    Self::is_signer(&[owner])?;

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
    let seed: &[u8] = &Self::safe_seed(pool_acc, treasurer, program_id)?[..];
    if pool_data.treasury != *treasury_acc.key || lpt_data.owner != *owner.key {
      return Err(AppError::InvalidOwner.into());
    }
    if lpt_data.pool != *pool_acc.key {
      return Err(AppError::UnmatchedPool.into());
    }
    if pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
    }
    if lpt == 0 {
      return Err(AppError::ZeroValue.into());
    }
    if lpt_data.lpt < lpt {
      return Err(AppError::InsufficientFunds.into());
    }

    // Compute corresponding paid-back reserve
    let paid_reserve = (pool_data.reserve as u128)
      .checked_mul(lpt)
      .ok_or(AppError::Overflow)?
      .checked_div(pool_data.lpt)
      .ok_or(AppError::Overflow)? as u64;
    // Update lpt data
    lpt_data.lpt = lpt_data.lpt.checked_sub(lpt).ok_or(AppError::Overflow)?;
    LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;
    // Update pool data
    pool_data.reserve = pool_data
      .reserve
      .checked_sub(paid_reserve)
      .ok_or(AppError::Overflow)?;
    pool_data.lpt = pool_data.lpt.checked_sub(lpt).ok_or(AppError::Overflow)?;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
    // Withdraw token
    XSPLT::transfer(
      paid_reserve,
      treasury_acc,
      dst_acc,
      treasurer,
      splt_program,
      seed,
    )?;

    Ok(())
  }

  pub fn swap<'a>(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo<'a>]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;

    let bid_pool_acc = next_account_info(accounts_iter)?;
    let bid_treasury_acc = next_account_info(accounts_iter)?;
    let src_acc = next_account_info(accounts_iter)?;

    let ask_pool_acc = next_account_info(accounts_iter)?;
    let ask_treasury_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    let ask_treasurer = next_account_info(accounts_iter)?;

    let sen_pool_acc = next_account_info(accounts_iter)?;
    let sen_treasury_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;
    let sen_treasurer = next_account_info(accounts_iter)?;

    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(
      program_id,
      &[network_acc, bid_pool_acc, ask_pool_acc, sen_pool_acc],
    )?;
    Self::is_signer(&[owner])?;

    let network_data = Network::unpack(&network_acc.data.borrow())?;
    let mut bid_pool_data = Pool::unpack(&bid_pool_acc.data.borrow())?;
    let mut ask_pool_data = Pool::unpack(&ask_pool_acc.data.borrow())?;
    let mut sen_pool_data = Pool::unpack(&sen_pool_acc.data.borrow())?;
    let ask_seed: &[u8] = &Self::safe_seed(ask_pool_acc, ask_treasurer, program_id)?[..];
    let sen_seed: &[u8] = &Self::safe_seed(sen_pool_acc, sen_treasurer, program_id)?[..];
    if bid_pool_data.treasury != *bid_treasury_acc.key
      || ask_pool_data.treasury != *ask_treasury_acc.key
      || sen_pool_data.treasury != *sen_treasury_acc.key
    {
      return Err(AppError::InvalidOwner.into());
    }
    if bid_pool_data.network != *network_acc.key
      || ask_pool_data.network != *network_acc.key
      || sen_pool_data.network != *network_acc.key
    {
      return Err(AppError::IncorrectNetworkId.into());
    }
    if bid_pool_data.is_frozen() || ask_pool_data.is_frozen() || sen_pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
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

    // Transfer bid
    XSPLT::transfer(amount, src_acc, bid_treasury_acc, owner, splt_program, &[])?;
    // Update bid pool data
    bid_pool_data.reserve = new_bid_reserve;
    Pool::pack(bid_pool_data, &mut bid_pool_acc.data.borrow_mut())?;

    // Apply fee
    let exempt = ask_pool_data.mint == network_data.primary_token;
    let (new_ask_reserve_with_fee, paid_amount, _fee, earning) =
      Self::apply_fee(new_ask_reserve_without_fee, ask_pool_data.reserve, exempt)
        .ok_or(AppError::Overflow)?;

    // Update ask pool data (Including swap ask_token to SEN)
    // new_ask_reserve_without_fee + fee + earning = new_ask_reserve_with_fee + earning
    let new_ask_reserve = new_ask_reserve_with_fee
      .checked_add(earning)
      .ok_or(AppError::Overflow)?;
    ask_pool_data.reserve = new_ask_reserve;
    Pool::pack(ask_pool_data, &mut ask_pool_acc.data.borrow_mut())?;
    // Transfer ask
    XSPLT::transfer(
      paid_amount,
      ask_treasury_acc,
      dst_acc,
      ask_treasurer,
      splt_program,
      ask_seed,
    )?;

    // Execute earning
    if earning != 0 {
      // Swap earning to SEN
      let new_sen_reserve = Oracle::curve(
        new_ask_reserve,          // with earning
        new_ask_reserve_with_fee, // without earning
        ask_pool_data.lpt,
        sen_pool_data.reserve,
        sen_pool_data.lpt,
      )
      .ok_or(AppError::Overflow)?;
      let earning_in_sen = sen_pool_data
        .reserve
        .checked_sub(new_sen_reserve)
        .ok_or(AppError::Overflow)?;
      sen_pool_data.reserve = new_sen_reserve;
      Pool::pack(sen_pool_data, &mut sen_pool_acc.data.borrow_mut())?;
      // Transfer earning
      XSPLT::transfer(
        earning_in_sen,
        sen_treasury_acc,
        vault_acc,
        sen_treasurer,
        splt_program,
        sen_seed,
      )?;
    }

    Ok(())
  }

  pub fn transfer(lpt: u128, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let src_lpt_acc = next_account_info(accounts_iter)?;
    let dst_lpt_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[src_lpt_acc, dst_lpt_acc])?;
    Self::is_signer(&[owner])?;

    let mut src_lpt_data = LPT::unpack(&src_lpt_acc.data.borrow())?;
    let mut dst_lpt_data = LPT::unpack(&dst_lpt_acc.data.borrow())?;
    if src_lpt_data.owner != *owner.key {
      return Err(AppError::InvalidOwner.into());
    }
    if src_lpt_data.pool != dst_lpt_data.pool {
      return Err(AppError::UnmatchedPool.into());
    }
    if lpt == 0 {
      return Err(AppError::ZeroValue.into());
    }
    if src_lpt_data.lpt < lpt {
      return Err(AppError::InsufficientFunds.into());
    }
    if *src_lpt_acc.key == *dst_lpt_acc.key {
      return Ok(());
    }

    // Update lpt data
    src_lpt_data.lpt = src_lpt_data
      .lpt
      .checked_sub(lpt)
      .ok_or(AppError::Overflow)?;
    LPT::pack(src_lpt_data, &mut src_lpt_acc.data.borrow_mut())?;
    dst_lpt_data.lpt = dst_lpt_data
      .lpt
      .checked_add(lpt)
      .ok_or(AppError::Overflow)?;
    LPT::pack(dst_lpt_data, &mut dst_lpt_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn freeze_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[network_acc, pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_owner(owner, network_acc)?;

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    if pool_data.network != *network_acc.key {
      return Err(AppError::IncorrectNetworkId.into());
    }

    pool_data.state = PoolState::Frozen;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn thaw_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[network_acc, pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_owner(owner, network_acc)?;

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    if pool_data.network != *network_acc.key {
      return Err(AppError::IncorrectNetworkId.into());
    }

    pool_data.state = PoolState::Initialized;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn earn<'a>(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo<'a>]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[network_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_owner(owner, network_acc)?;

    let network_data = Network::unpack(&network_acc.data.borrow())?;
    let seed: &[u8] = &Self::safe_seed(vault_acc, treasurer, program_id)?[..];
    if network_data.vault != *vault_acc.key {
      return Err(AppError::InvalidOwner.into());
    }
    if amount == 0 {
      return Err(AppError::ZeroValue.into());
    }
    // Transfer earning
    XSPLT::transfer(amount, vault_acc, dst_acc, treasurer, splt_program, seed)?;

    Ok(())
  }

  pub fn close_lpt(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[lpt_acc])?;
    Self::is_signer(&[owner])?;

    let lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
    if lpt_data.owner != *owner.key {
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

  pub fn close_pool<'a>(program_id: &Pubkey, accounts: &[AccountInfo<'a>]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let treasury_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[network_acc, pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_owner(owner, network_acc)?;

    let pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let seed: &[u8] = &Self::safe_seed(pool_acc, treasurer, program_id)?[..];
    if pool_data.treasury != *treasury_acc.key {
      return Err(AppError::InvalidOwner.into());
    }
    if pool_data.network != *network_acc.key {
      return Err(AppError::IncorrectNetworkId.into());
    }
    if pool_data.lpt != 0 || pool_data.reserve != 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Close treasury
    XSPLT::close_account(treasury_acc, dst_acc, treasurer, splt_program, seed)?;
    // Close pool
    let dst_lamports = dst_acc.lamports();
    **dst_acc.lamports.borrow_mut() = dst_lamports
      .checked_add(pool_acc.lamports())
      .ok_or(AppError::Overflow)?;
    **pool_acc.lamports.borrow_mut() = 0;

    Ok(())
  }

  pub fn transfer_ownership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let new_owner = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[network_acc])?;
    Self::is_signer(&[owner, new_owner])?;
    Self::is_owner(owner, network_acc)?;

    // Update network data
    let mut network_data = Network::unpack(&network_acc.data.borrow())?;
    network_data.owner = *new_owner.key;
    Network::pack(network_data, &mut network_acc.data.borrow_mut())?;

    Ok(())
  }

  ///
  /// Utilities
  ///

  pub fn is_program(program_id: &Pubkey, accounts: &[&AccountInfo]) -> ProgramResult {
    for acc in &mut accounts.iter() {
      if acc.owner != program_id {
        return Err(AppError::IncorrectProgramId.into());
      }
    }
    Ok(())
  }

  pub fn is_signer(accounts: &[&AccountInfo]) -> ProgramResult {
    for acc in &mut accounts.iter() {
      if !acc.is_signer {
        return Err(AppError::InvalidOwner.into());
      }
    }
    Ok(())
  }

  pub fn is_pool_owner(owner: &AccountInfo, pool_acc: &AccountInfo) -> ProgramResult {
    let pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    if pool_data.owner != *owner.key {
      return Err(AppError::InvalidOwner.into());
    }
    Ok(())
  }

  pub fn is_lpt_owner(owner: &AccountInfo, lpt_acc: &AccountInfo) -> ProgramResult {
    let lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
    if lpt_data.owner != *owner.key {
      return Err(AppError::InvalidOwner.into());
    }
    Ok(())
  }

  pub fn safe_seed(
    seed_acc: &AccountInfo,
    expected_acc: &AccountInfo,
    program_id: &Pubkey,
  ) -> Result<[u8; 32], PubkeyError> {
    let seed: [u8; 32] = seed_acc.key.to_bytes();
    let key = Pubkey::create_program_address(&[&seed], program_id)?;
    if key != *expected_acc.key {
      return Err(PubkeyError::InvalidSeeds);
    }
    Ok(seed)
  }
}
