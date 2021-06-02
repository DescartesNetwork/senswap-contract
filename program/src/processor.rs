use crate::error::AppError;
use crate::helper::{oracle::Oracle, pubutil::Boolean};
use crate::instruction::AppInstruction;
use crate::interfaces::{xsplata::XSPLATA, xsplt::XSPLT};
use crate::schema::{
  mint::Mint,
  pool::{Pool, PoolState},
};
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  msg,
  program_pack::{IsInitialized, Pack},
  pubkey::{Pubkey, PubkeyError},
};

pub struct Processor {}

impl Processor {
  ///
  /// Entrypoint
  ///
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = AppInstruction::unpack(instruction_data)?;
    match instruction {
      AppInstruction::InitializePool {
        reserve_s,
        reserve_a,
        reserve_b,
      } => {
        msg!("Calling InitializePool function");
        Self::initialize_pool(reserve_s, reserve_a, reserve_b, program_id, accounts)
      }

      AppInstruction::AddLiquidity {
        delta_s,
        delta_a,
        delta_b,
      } => {
        msg!("Calling AddLiquidity function");
        Self::add_liquidity(delta_s, delta_a, delta_b, program_id, accounts)
      }

      AppInstruction::RemoveLiquidity { lpt } => {
        msg!("Calling RemoveLiquidity function");
        Self::remove_liquidity(lpt, program_id, accounts)
      }

      AppInstruction::Swap { amount, limit } => {
        msg!("Calling Swap function");
        Self::swap(amount, limit, program_id, accounts)
      }

      AppInstruction::FreezePool {} => {
        msg!("Calling FreezePool function");
        Self::freeze_pool(program_id, accounts)
      }

      AppInstruction::ThawPool {} => {
        msg!("Calling ThawPool function");
        Self::thaw_pool(program_id, accounts)
      }

      AppInstruction::Earn { amount } => {
        msg!("Calling Earn function");
        Self::earn(amount, program_id, accounts)
      }

      AppInstruction::TransferPoolOwnership {} => {
        msg!("Calling TransferPoolOwnership function");
        Self::transfer_pool_ownership(program_id, accounts)
      }
    }
  }

  ///
  /// Controllers
  ///

  pub fn initialize_pool(
    reserve_s: u64,
    reserve_a: u64,
    reserve_b: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let mint_lpt_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;
    let proof_acc = next_account_info(accounts_iter)?; // program_id xor treasurer xor pool_id

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
    let system_program = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    let sysvar_rent_acc = next_account_info(accounts_iter)?;
    let splata_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc])?;
    Self::is_signer(&[payer, pool_acc, vault_acc])?;

    let mut pool_data = Pool::unpack_unchecked(&pool_acc.data.borrow())?;
    let mint_lpt_data = Mint::unpack_unchecked(&mint_lpt_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(pool_acc, treasurer, program_id)?[..]]];
    if pool_data.is_initialized() || mint_lpt_data.is_initialized() {
      return Err(AppError::ConstructorOnce.into());
    }
    if *proof_acc.key != program_id.xor(&(pool_acc.key.xor(treasurer.key)))
      || *mint_s_acc.key == *mint_a_acc.key
      || *mint_s_acc.key == *mint_b_acc.key
    {
      return Err(AppError::InvalidMint.into());
    }
    if reserve_s == 0 || reserve_a == 0 || reserve_b == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Initialize treasury S
    XSPLATA::initialize_account(
      payer,
      treasury_s_acc,
      treasurer,
      mint_s_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      seed,
    )?;
    // Deposit token S
    XSPLT::transfer(
      reserve_s,
      src_s_acc,
      treasury_s_acc,
      payer,
      splt_program,
      &[],
    )?;

    // Initialize treasury A
    XSPLATA::initialize_account(
      payer,
      treasury_a_acc,
      treasurer,
      mint_a_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      seed,
    )?;
    // Deposit token A
    XSPLT::transfer(
      reserve_a,
      src_a_acc,
      treasury_a_acc,
      payer,
      splt_program,
      &[],
    )?;

    // Initialize treasury B
    XSPLATA::initialize_account(
      payer,
      treasury_b_acc,
      treasurer,
      mint_b_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      seed,
    )?;
    // Deposit token B
    XSPLT::transfer(
      reserve_b,
      src_b_acc,
      treasury_b_acc,
      payer,
      splt_program,
      &[],
    )?;

    // Initialize mint
    let mint_s_data = Mint::unpack_unchecked(&mint_s_acc.data.borrow())?;
    XSPLT::initialize_mint(
      mint_s_data.decimals,
      mint_lpt_acc,
      treasurer,
      proof_acc,
      sysvar_rent_acc,
      splt_program,
      seed,
    )?;
    // Initialize lpt account
    XSPLATA::initialize_account(
      payer,
      lpt_acc,
      payer,
      mint_lpt_acc,
      system_program,
      splt_program,
      sysvar_rent_acc,
      splata_program,
      seed,
    )?;
    // Mint LPT
    XSPLT::mint_to(
      reserve_s,
      mint_lpt_acc,
      lpt_acc,
      treasurer,
      splt_program,
      seed,
    )?;

    // Initialize vault
    XSPLT::initialize_account(
      vault_acc,
      mint_s_acc,
      treasurer,
      sysvar_rent_acc,
      splt_program,
      seed,
    )?;

    // Update pool data
    pool_data.owner = *owner.key;
    pool_data.state = PoolState::Initialized;
    pool_data.mint_lpt = *mint_lpt_acc.key;
    pool_data.vault = *vault_acc.key;
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

    Ok(())
  }

  pub fn add_liquidity(
    delta_s: u64,
    delta_a: u64,
    delta_b: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let mint_lpt_acc = next_account_info(accounts_iter)?;

    let src_s_acc = next_account_info(accounts_iter)?;
    let treasury_s_acc = next_account_info(accounts_iter)?;

    let src_a_acc = next_account_info(accounts_iter)?;
    let treasury_a_acc = next_account_info(accounts_iter)?;

    let src_b_acc = next_account_info(accounts_iter)?;
    let treasury_b_acc = next_account_info(accounts_iter)?;

    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc])?;
    Self::is_signer(&[owner])?;

    let mint_lpt_data = Mint::unpack(&mint_lpt_acc.data.borrow())?;
    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(pool_acc, treasurer, program_id)?[..]]];
    if pool_data.mint_lpt != *mint_lpt_acc.key
      || pool_data.treasury_s != *treasury_s_acc.key
      || pool_data.treasury_a != *treasury_a_acc.key
      || pool_data.treasury_b != *treasury_b_acc.key
    {
      return Err(AppError::InvalidOwner.into());
    }
    if delta_s == 0 && delta_a == 0 && delta_b == 0 {
      return Err(AppError::ZeroValue.into());
    }

    let (lpt, reserve_s, reserve_a, reserve_b) = Oracle::rake(
      delta_s,
      delta_a,
      delta_b,
      pool_data.reserve_s,
      pool_data.reserve_a,
      pool_data.reserve_b,
      mint_lpt_data.supply,
    )
    .ok_or(AppError::Overflow)?;

    // Deposit token
    if delta_s > 0 {
      XSPLT::transfer(delta_s, src_s_acc, treasury_s_acc, owner, splt_program, &[])?;
      pool_data.reserve_s = reserve_s;
    }
    if delta_a > 0 {
      XSPLT::transfer(delta_a, src_a_acc, treasury_a_acc, owner, splt_program, &[])?;
      pool_data.reserve_a = reserve_a;
    }
    if delta_b > 0 {
      XSPLT::transfer(delta_b, src_b_acc, treasury_b_acc, owner, splt_program, &[])?;
      pool_data.reserve_b = reserve_b;
    }
    // Update pool
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
    // Mint LPT
    XSPLT::mint_to(lpt, mint_lpt_acc, lpt_acc, treasurer, splt_program, seed)?;

    Ok(())
  }

  pub fn remove_liquidity(
    lpt: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let mint_lpt_acc = next_account_info(accounts_iter)?;

    let dst_s_acc = next_account_info(accounts_iter)?;
    let treasury_s_acc = next_account_info(accounts_iter)?;

    let dst_a_acc = next_account_info(accounts_iter)?;
    let treasury_a_acc = next_account_info(accounts_iter)?;

    let dst_b_acc = next_account_info(accounts_iter)?;
    let treasury_b_acc = next_account_info(accounts_iter)?;

    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc])?;
    Self::is_signer(&[owner])?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(pool_acc, treasurer, program_id)?[..]]];

    let mint_lpt_data = Mint::unpack(&mint_lpt_acc.data.borrow())?;
    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    if pool_data.mint_lpt != *mint_lpt_acc.key
      || pool_data.treasury_s != *treasury_s_acc.key
      || pool_data.treasury_a != *treasury_a_acc.key
      || pool_data.treasury_b != *treasury_b_acc.key
    {
      return Err(AppError::UnmatchedPool.into());
    }
    if pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
    }
    if lpt == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Compute corresponding paid-back reserve
    let delta_s = (lpt as u128)
      .checked_mul(pool_data.reserve_s as u128)
      .ok_or(AppError::Overflow)?
      .checked_div(mint_lpt_data.supply as u128)
      .ok_or(AppError::Overflow)? as u64;
    let delta_a = (lpt as u128)
      .checked_mul(pool_data.reserve_a as u128)
      .ok_or(AppError::Overflow)?
      .checked_div(mint_lpt_data.supply as u128)
      .ok_or(AppError::Overflow)? as u64;
    let delta_b = (lpt as u128)
      .checked_mul(pool_data.reserve_b as u128)
      .ok_or(AppError::Overflow)?
      .checked_div(mint_lpt_data.supply as u128)
      .ok_or(AppError::Overflow)? as u64;
    // Burn LPT
    XSPLT::burn(lpt, lpt_acc, mint_lpt_acc, owner, splt_program, seed)?;
    // Update pool data
    pool_data.reserve_s = pool_data
      .reserve_s
      .checked_sub(delta_s)
      .ok_or(AppError::Overflow)?;
    pool_data.reserve_a = pool_data
      .reserve_a
      .checked_sub(delta_a)
      .ok_or(AppError::Overflow)?;
    pool_data.reserve_b = pool_data
      .reserve_b
      .checked_sub(delta_b)
      .ok_or(AppError::Overflow)?;
    if pool_data.reserve_s == 0 {
      pool_data.state = PoolState::Frozen;
    }
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
    // Withdraw token
    XSPLT::transfer(
      delta_s,
      treasury_s_acc,
      dst_s_acc,
      treasurer,
      splt_program,
      seed,
    )?;
    XSPLT::transfer(
      delta_a,
      treasury_a_acc,
      dst_a_acc,
      treasurer,
      splt_program,
      seed,
    )?;
    XSPLT::transfer(
      delta_b,
      treasury_b_acc,
      dst_b_acc,
      treasurer,
      splt_program,
      seed,
    )?;

    Ok(())
  }

  pub fn swap(
    amount: u64,
    limit: u64,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;

    let src_acc = next_account_info(accounts_iter)?;
    let treasury_bid_acc = next_account_info(accounts_iter)?;

    let dst_acc = next_account_info(accounts_iter)?;
    let treasury_ask_acc = next_account_info(accounts_iter)?;

    let treasury_sen_acc = next_account_info(accounts_iter)?;

    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc])?;
    Self::is_signer(&[payer])?;

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(pool_acc, treasurer, program_id)?[..]]];
    let (bid_code, bid_reserve) = pool_data
      .get_reserve(treasury_bid_acc.key)
      .ok_or(AppError::UnmatchedPool)?;
    let (ask_code, ask_reserve) = pool_data
      .get_reserve(treasury_ask_acc.key)
      .ok_or(AppError::UnmatchedPool)?;
    let (sen_code, _) = pool_data
      .get_reserve(treasury_sen_acc.key)
      .ok_or(AppError::UnmatchedPool)?;
    if sen_code != 0 {
      return Err(AppError::UnmatchedPool.into());
    }
    if pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
    }
    if amount == 0 {
      return Err(AppError::ZeroValue.into());
    }
    if *treasury_bid_acc.key == *treasury_ask_acc.key {
      return Ok(());
    }

    // Compute new state
    let new_bid_reserve = bid_reserve.checked_add(amount).ok_or(AppError::Overflow)?;
    let (new_ask_reserve, paid_amount, earning) =
      Oracle::curve_in_fee(new_bid_reserve, bid_reserve, ask_reserve, ask_code == 0)
        .ok_or(AppError::Overflow)?;
    if paid_amount < limit {
      return Err(AppError::ExceedLimit.into());
    }

    // Transfer bid
    XSPLT::transfer(amount, src_acc, treasury_bid_acc, payer, splt_program, &[])?;
    // Update bid pool data
    match bid_code {
      0 => pool_data.reserve_s = new_bid_reserve,
      1 => pool_data.reserve_a = new_bid_reserve,
      2 => pool_data.reserve_b = new_bid_reserve,
      _ => return Err(AppError::UnmatchedPool.into()),
    }
    match ask_code {
      0 => pool_data.reserve_s = new_ask_reserve,
      1 => pool_data.reserve_a = new_ask_reserve,
      2 => pool_data.reserve_b = new_ask_reserve,
      _ => return Err(AppError::UnmatchedPool.into()),
    }
    // Transfer ask
    XSPLT::transfer(
      paid_amount,
      treasury_ask_acc,
      dst_acc,
      treasurer,
      splt_program,
      seed,
    )?;

    // Execute earning
    if earning != 0 {
      // Swap earning to SEN
      let new_ask_reserve_with_earning = new_ask_reserve
        .checked_add(earning)
        .ok_or(AppError::Overflow)?;
      let (new_sen_reserve, earning_in_sen, _) = Oracle::curve_in_fee(
        new_ask_reserve_with_earning, // with earning
        new_ask_reserve,              // without earning
        pool_data.reserve_s,
        true,
      )
      .ok_or(AppError::Overflow)?;
      match ask_code {
        1 => pool_data.reserve_a = new_ask_reserve_with_earning,
        2 => pool_data.reserve_b = new_ask_reserve_with_earning,
        _ => return Err(AppError::UnmatchedPool.into()),
      }
      pool_data.reserve_s = new_sen_reserve;
      // Transfer earning
      XSPLT::transfer(
        earning_in_sen,
        treasury_sen_acc,
        vault_acc,
        treasurer,
        splt_program,
        seed,
      )?;
    }

    // Save final data
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn freeze_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_pool_owner(owner, pool_acc)?;

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    pool_data.state = PoolState::Frozen;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn thaw_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_pool_owner(owner, pool_acc)?;

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    pool_data.state = PoolState::Initialized;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

    Ok(())
  }

  pub fn earn(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_pool_owner(owner, pool_acc)?;

    let pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let seed: &[&[&[u8]]] = &[&[&Self::safe_seed(pool_acc, treasurer, program_id)?[..]]];
    if pool_data.vault != *vault_acc.key {
      return Err(AppError::InvalidOwner.into());
    }
    if amount == 0 {
      return Err(AppError::ZeroValue.into());
    }
    // Transfer earning
    XSPLT::transfer(amount, vault_acc, dst_acc, treasurer, splt_program, seed)?;

    Ok(())
  }

  pub fn transfer_pool_ownership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let new_owner = next_account_info(accounts_iter)?;

    Self::is_program(program_id, &[pool_acc])?;
    Self::is_signer(&[owner])?;
    Self::is_pool_owner(owner, pool_acc)?;

    // Update pool data
    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    pool_data.owner = *new_owner.key;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

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
