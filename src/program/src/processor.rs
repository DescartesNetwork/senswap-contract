use crate::error::AppError;
use crate::helper::oracle::Oracle;
use crate::instruction::AppInstruction;
use crate::interfaces::isplt::ISPLT;
use crate::schema::{
  dao::{DAO, MAX_SIGNERS},
  lpt::LPT,
  network::{Network, NetworkState, MAX_MINTS},
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
/// fee = 2500000/1000000000 = 0.25%
/// earn = 500000/1000000000 = 0.05%
///
const FEE: u64 = 2500000;
const EARNING: u64 = 500000;
const DECIMALS: u64 = 1000000000;

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
      AppInstruction::InitializeNetwork {} => {
        info!("Calling InitializeNetwork function");
        Self::intialize_network(program_id, accounts)
      }

      AppInstruction::InitializePool { reserve, lpt } => {
        info!("Calling InitializePool function");
        Self::initialize_pool(reserve, lpt, program_id, accounts)
      }

      AppInstruction::InitializeLPT {} => {
        info!("Calling InitializeLPTfunction");
        Self::initialize_lpt(program_id, accounts)
      }

      AppInstruction::AddLiquidity { reserve } => {
        info!("Calling AddLiquidity function");
        Self::add_liquidity(reserve, program_id, accounts)
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

      AppInstruction::AddSigner {} => {
        info!("Calling AddSigner function");
        Self::add_signer(program_id, accounts)
      }

      AppInstruction::ReplaceSigner {} => {
        info!("Calling ReplaceSigner function");
        Self::replace_signer(program_id, accounts)
      }

      AppInstruction::RemoveSigner {} => {
        info!("Calling RemoveSigner function");
        Self::remove_signer(program_id, accounts)
      }

      AppInstruction::CloseLPT {} => {
        info!("Calling CloseLPT function");
        Self::close_lpt(program_id, accounts)
      }

      AppInstruction::ClosePool {} => {
        info!("Calling ClosePool function");
        Self::close_pool(program_id, accounts)
      }
    }
  }

  ///
  /// Controllers
  ///
  fn intialize_network(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let primary_token_acc = next_account_info(accounts_iter)?;
    let vault_acc = next_account_info(accounts_iter)?;
    let dao_acc = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    let sysvar_rent_acc = next_account_info(accounts_iter)?;
    if network_acc.owner != program_id || dao_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let mut network_data = Network::unpack_unchecked(&network_acc.data.borrow())?;
    let mut dao_data = DAO::unpack_unchecked(&dao_acc.data.borrow())?;
    if network_data.is_initialized() || dao_data.is_initialized() {
      return Err(AppError::ConstructorOnce.into());
    }
    if !owner.is_signer || !network_acc.is_signer || !vault_acc.is_signer || !dao_acc.is_signer {
      return Err(AppError::InvalidOwner.into());
    }

    // Vault Constructor
    let ix_initialize_account = ISPLT::initialize_account(
      *vault_acc.key,
      *primary_token_acc.key,
      *owner.key,
      *sysvar_rent_acc.key,
      *splt_program.key,
    )?;
    invoke(
      &ix_initialize_account,
      &[
        vault_acc.clone(),
        primary_token_acc.clone(),
        owner.clone(),
        sysvar_rent_acc.clone(),
        splt_program.clone(),
      ],
    )?;

    // Update DAO data
    dao_data.signers[0] = *owner.key;
    for j in 1..MAX_SIGNERS {
      let signer = next_account_info(accounts_iter)?;
      dao_data.signers[j] = *signer.key;
    }
    dao_data.is_initialized = true;
    DAO::pack(dao_data, &mut dao_acc.data.borrow_mut())?;
    // Update network data
    network_data.owner = *dao_acc.key;
    network_data.primary_token = *primary_token_acc.key;
    network_data.vault = *vault_acc.key;
    network_data.mints[0] = *primary_token_acc.key;
    for i in 1..MAX_MINTS {
      let mint_acc = next_account_info(accounts_iter)?;
      network_data.mints[i] = *mint_acc.key;
    }
    network_data.state = NetworkState::Initialized;
    Network::pack(network_data, &mut network_acc.data.borrow_mut())?;

    Ok(())
  }

  fn initialize_pool(
    reserve: u64,
    lpt: u128,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let treasury_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let src_acc = next_account_info(accounts_iter)?;
    let mint_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    let sysvar_rent_acc = next_account_info(accounts_iter)?;
    if network_acc.owner != program_id
      || pool_acc.owner != program_id
      || lpt_acc.owner != program_id
    {
      return Err(AppError::IncorrectProgramId.into());
    }

    let mut network_data = Network::unpack(&network_acc.data.borrow())?;
    let mut pool_data = Pool::unpack_unchecked(&pool_acc.data.borrow())?;
    let mut lpt_data = LPT::unpack_unchecked(&lpt_acc.data.borrow())?;
    if pool_data.is_initialized() || lpt_data.is_initialized() {
      return Err(AppError::ConstructorOnce.into());
    }
    let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
    let treasurer_key = Pubkey::create_program_address(&seed, program_id)?;
    if !owner.is_signer
      || !pool_acc.is_signer
      || !lpt_acc.is_signer
      || treasurer_key != *treasurer.key
    {
      return Err(AppError::InvalidOwner.into());
    }
    if !network_data.is_approved(mint_acc.key) {
      return Err(AppError::UnmatchedPool.into());
    }
    if *mint_acc.key != network_data.primary_token && !network_data.is_activated() {
      return Err(AppError::NotInitialized.into());
    }
    if *mint_acc.key == network_data.primary_token && network_data.is_activated() {
      return Err(AppError::ConstructorOnce.into());
    }
    if reserve == 0 || lpt == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Account Constructor
    let ix_initialize_account = ISPLT::initialize_account(
      *treasury_acc.key,
      *mint_acc.key,
      *treasurer.key,
      *sysvar_rent_acc.key,
      *splt_program.key,
    )?;
    invoke_signed(
      &ix_initialize_account,
      &[
        treasury_acc.clone(),
        mint_acc.clone(),
        treasurer.clone(),
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
      *owner.key,
      *splt_program.key,
    )?;
    invoke(
      &ix_transfer,
      &[
        src_acc.clone(),
        treasury_acc.clone(),
        owner.clone(),
        splt_program.clone(),
      ],
    )?;

    // Update network data
    if *mint_acc.key == network_data.primary_token {
      network_data.state = NetworkState::Activated;
      Network::pack(network_data, &mut network_acc.data.borrow_mut())?;
    }
    // Update pool data
    pool_data.owner = *owner.key;
    pool_data.network = *network_acc.key;
    pool_data.mint = *mint_acc.key;
    pool_data.treasury = *treasury_acc.key;
    pool_data.reserve = reserve;
    pool_data.lpt = lpt;
    pool_data.state = PoolState::Initialized;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;
    // Update lpt data
    lpt_data.owner = *owner.key;
    lpt_data.pool = *pool_acc.key;
    lpt_data.lpt = lpt;
    lpt_data.is_initialized = true;
    LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

    Ok(())
  }

  fn initialize_lpt(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    if pool_acc.owner != program_id || lpt_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let mut lpt_data = LPT::unpack_unchecked(&lpt_acc.data.borrow())?;
    if lpt_data.is_initialized() {
      return Err(AppError::ConstructorOnce.into());
    }
    if !owner.is_signer || !lpt_acc.is_signer {
      return Err(AppError::InvalidOwner.into());
    }

    lpt_data.owner = *owner.key;
    lpt_data.pool = *pool_acc.key;
    lpt_data.lpt = 0;
    lpt_data.is_initialized = true;
    LPT::pack(lpt_data, &mut lpt_acc.data.borrow_mut())?;

    Ok(())
  }

  fn add_liquidity(reserve: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let treasury_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let src_acc = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    if pool_acc.owner != program_id || lpt_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
    if !owner.is_signer || pool_data.treasury != *treasury_acc.key || lpt_data.owner != *owner.key {
      return Err(AppError::InvalidOwner.into());
    }
    if lpt_data.pool != *pool_acc.key {
      return Err(AppError::UnmatchedPool.into());
    }
    if pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
    }
    if reserve == 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Deposit token
    let ix_transfer = ISPLT::transfer(
      reserve,
      *src_acc.key,
      *treasury_acc.key,
      *owner.key,
      *splt_program.key,
    )?;
    invoke(
      &ix_transfer,
      &[
        src_acc.clone(),
        treasury_acc.clone(),
        owner.clone(),
        splt_program.clone(),
      ],
    )?;

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

  fn remove_liquidity(lpt: u128, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let treasury_acc = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    if pool_acc.owner != program_id || lpt_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let mut lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
    let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
    let treasurer_key = Pubkey::create_program_address(&seed, program_id)?;
    if !owner.is_signer
      || pool_data.treasury != *treasury_acc.key
      || lpt_data.owner != *owner.key
      || treasurer_key != *treasurer.key
    {
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
      *treasurer.key,
      *splt_program.key,
    )?;
    invoke_signed(
      &ix_transfer,
      &[
        treasury_acc.clone(),
        dst_acc.clone(),
        treasurer.clone(),
        splt_program.clone(),
      ],
      &[&seed],
    )?;

    Ok(())
  }

  fn swap(amount: u64, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
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
    if bid_pool_acc.owner != program_id
      || ask_pool_acc.owner != program_id
      || sen_pool_acc.owner != program_id
    {
      return Err(AppError::IncorrectProgramId.into());
    }

    let network_data = Network::unpack(&network_acc.data.borrow())?;
    let mut bid_pool_data = Pool::unpack(&bid_pool_acc.data.borrow())?;
    let mut ask_pool_data = Pool::unpack(&ask_pool_acc.data.borrow())?;
    let mut sen_pool_data = Pool::unpack(&sen_pool_acc.data.borrow())?;
    let ask_seed: &[&[_]] = &[&ask_pool_acc.key.to_bytes()[..]];
    let ask_treasurer_key = Pubkey::create_program_address(&ask_seed, program_id)?;
    let sen_seed: &[&[_]] = &[&sen_pool_acc.key.to_bytes()[..]];
    let sen_treasurer_key = Pubkey::create_program_address(&sen_seed, program_id)?;
    if !owner.is_signer
      || bid_pool_data.treasury != *bid_treasury_acc.key
      || ask_pool_data.treasury != *ask_treasury_acc.key
      || ask_treasurer_key != *ask_treasurer.key
      || sen_pool_data.treasury != *sen_treasury_acc.key
      || sen_treasurer_key != *sen_treasurer.key
    {
      return Err(AppError::InvalidOwner.into());
    }
    if sen_pool_data.network != *network_acc.key
      || bid_pool_data.network != *network_acc.key
      || ask_pool_data.network != *network_acc.key
    {
      return Err(AppError::IncorrectNetworkId.into());
    }
    if bid_pool_data.is_frozen() || ask_pool_data.is_frozen() {
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
    let ix_transfer = ISPLT::transfer(
      amount,
      *src_acc.key,
      *bid_treasury_acc.key,
      *owner.key,
      *splt_program.key,
    )?;
    invoke(
      &ix_transfer,
      &[
        src_acc.clone(),
        bid_treasury_acc.clone(),
        owner.clone(),
        splt_program.clone(),
      ],
    )?;
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
    // Transfer ask
    Pool::pack(ask_pool_data, &mut ask_pool_acc.data.borrow_mut())?;
    let ix_transfer = ISPLT::transfer(
      paid_amount,
      *ask_treasury_acc.key,
      *dst_acc.key,
      *ask_treasurer.key,
      *splt_program.key,
    )?;
    invoke_signed(
      &ix_transfer,
      &[
        ask_treasury_acc.clone(),
        dst_acc.clone(),
        ask_treasurer.clone(),
        splt_program.clone(),
      ],
      &[&ask_seed],
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
        .checked_sub(new_ask_reserve)
        .ok_or(AppError::Overflow)?;
      sen_pool_data.reserve = new_sen_reserve;
      Pool::pack(sen_pool_data, &mut sen_pool_acc.data.borrow_mut())?;
      // Transfer earning
      let ix_transfer = ISPLT::transfer(
        earning_in_sen,
        *sen_treasury_acc.key,
        *vault_acc.key,
        *sen_treasurer.key,
        *splt_program.key,
      )?;
      invoke_signed(
        &ix_transfer,
        &[
          sen_treasury_acc.clone(),
          vault_acc.clone(),
          sen_treasurer.clone(),
          splt_program.clone(),
        ],
        &[&sen_seed],
      )?;
    }

    Ok(())
  }

  fn transfer(lpt: u128, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let src_lpt_acc = next_account_info(accounts_iter)?;
    let dst_lpt_acc = next_account_info(accounts_iter)?;
    if pool_acc.owner != program_id
      || src_lpt_acc.owner != program_id
      || dst_lpt_acc.owner != program_id
    {
      return Err(AppError::IncorrectProgramId.into());
    }

    let pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let mut src_lpt_data = LPT::unpack(&src_lpt_acc.data.borrow())?;
    let mut dst_lpt_data = LPT::unpack(&dst_lpt_acc.data.borrow())?;
    if !owner.is_signer || src_lpt_data.owner != *owner.key {
      return Err(AppError::InvalidOwner.into());
    }
    if src_lpt_data.pool != *pool_acc.key || dst_lpt_data.pool != *pool_acc.key {
      return Err(AppError::UnmatchedPool.into());
    }
    if pool_data.is_frozen() {
      return Err(AppError::FrozenPool.into());
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

  fn freeze_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let dao_acc = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    if dao_acc.owner != program_id
      || network_acc.owner != program_id
      || pool_acc.owner != program_id
    {
      return Err(AppError::IncorrectProgramId.into());
    }

    let dao_data = DAO::unpack(&dao_acc.data.borrow())?;
    let network_data = Network::unpack(&network_acc.data.borrow())?;
    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let is_owner = Self::multisig(&dao_data, accounts_iter.as_slice());
    if network_data.owner != *dao_acc.key || !is_owner {
      return Err(AppError::InvalidOwner.into());
    }
    if pool_data.network != *network_acc.key {
      return Err(AppError::IncorrectNetworkId.into());
    }

    pool_data.state = PoolState::Frozen;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

    Ok(())
  }

  fn thaw_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let dao_acc = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    if dao_acc.owner != program_id
      || network_acc.owner != program_id
      || pool_acc.owner != program_id
    {
      return Err(AppError::IncorrectProgramId.into());
    }

    let dao_data = DAO::unpack(&dao_acc.data.borrow())?;
    let network_data = Network::unpack(&network_acc.data.borrow())?;
    let mut pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let is_owner = Self::multisig(&dao_data, accounts_iter.as_slice());
    if network_data.owner != *dao_acc.key || is_owner {
      return Err(AppError::InvalidOwner.into());
    }
    if pool_data.network != *network_acc.key {
      return Err(AppError::IncorrectNetworkId.into());
    }

    pool_data.state = PoolState::Initialized;
    Pool::pack(pool_data, &mut pool_acc.data.borrow_mut())?;

    Ok(())
  }

  fn add_signer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let dao_acc = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let new_signer = next_account_info(accounts_iter)?;
    if dao_acc.owner != program_id || network_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let mut dao_data = DAO::unpack(&dao_acc.data.borrow())?;
    let network_data = Network::unpack(&network_acc.data.borrow())?;
    let is_owner = Self::multisig(&dao_data, accounts_iter.as_slice());
    if network_data.owner != *dao_acc.key || is_owner {
      return Err(AppError::InvalidOwner.into());
    }

    for (position, key) in dao_data.signers.iter().enumerate() {
      if *key == Pubkey::new(&[0u8; 32]) {
        dao_data.signers[position] = *new_signer.key;
        break;
      }
    }
    DAO::pack(dao_data, &mut dao_acc.data.borrow_mut())?;

    Ok(())
  }

  fn replace_signer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let dao_acc = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let old_signer = next_account_info(accounts_iter)?;
    let new_signer = next_account_info(accounts_iter)?;
    if dao_acc.owner != program_id || network_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let mut dao_data = DAO::unpack(&dao_acc.data.borrow())?;
    let network_data = Network::unpack(&network_acc.data.borrow())?;
    let is_owner = Self::multisig(&dao_data, accounts_iter.as_slice());
    if network_data.owner != *dao_acc.key || is_owner {
      return Err(AppError::InvalidOwner.into());
    }

    for (position, key) in dao_data.signers.iter().enumerate() {
      if *key == *old_signer.key {
        dao_data.signers[position] = *new_signer.key;
        break;
      }
    }
    DAO::pack(dao_data, &mut dao_acc.data.borrow_mut())?;

    Ok(())
  }

  fn remove_signer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let dao_acc = next_account_info(accounts_iter)?;
    let network_acc = next_account_info(accounts_iter)?;
    let old_signer = next_account_info(accounts_iter)?;
    if dao_acc.owner != program_id || network_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let mut dao_data = DAO::unpack(&dao_acc.data.borrow())?;
    let network_data = Network::unpack(&network_acc.data.borrow())?;
    let is_owner = Self::multisig(&dao_data, accounts_iter.as_slice());
    if network_data.owner != *dao_acc.key || is_owner {
      return Err(AppError::InvalidOwner.into());
    }

    for (position, key) in dao_data.signers.iter().enumerate() {
      if *key == *old_signer.key {
        dao_data.signers[position] = Pubkey::new(&[0u8; 32]);
        break;
      }
    }
    DAO::pack(dao_data, &mut dao_acc.data.borrow_mut())?;

    Ok(())
  }

  fn close_lpt(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let lpt_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    if lpt_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let lpt_data = LPT::unpack(&lpt_acc.data.borrow())?;
    if !owner.is_signer || lpt_data.owner != *owner.key {
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

  fn close_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let pool_acc = next_account_info(accounts_iter)?;
    let treasury_acc = next_account_info(accounts_iter)?;
    let dst_acc = next_account_info(accounts_iter)?;
    let treasurer = next_account_info(accounts_iter)?;
    let splt_program = next_account_info(accounts_iter)?;
    if pool_acc.owner != program_id {
      return Err(AppError::IncorrectProgramId.into());
    }

    let pool_data = Pool::unpack(&pool_acc.data.borrow())?;
    let seed: &[&[_]] = &[&pool_acc.key.to_bytes()[..]];
    let treasurer_key = Pubkey::create_program_address(&seed, program_id)?;
    if !owner.is_signer
      || pool_data.owner != *owner.key
      || pool_data.treasury != *treasury_acc.key
      || treasurer_key != *treasurer.key
    {
      return Err(AppError::InvalidOwner.into());
    }
    if pool_data.lpt != 0 || pool_data.reserve != 0 {
      return Err(AppError::ZeroValue.into());
    }

    // Close treasury
    let ix_close_account = ISPLT::close_account(
      *treasury_acc.key,
      *dst_acc.key,
      *treasurer.key,
      *splt_program.key,
    )?;
    invoke_signed(
      &ix_close_account,
      &[
        treasury_acc.clone(),
        dst_acc.clone(),
        treasurer.clone(),
        splt_program.clone(),
      ],
      &[&seed],
    )?;
    // Close pool
    let dst_lamports = dst_acc.lamports();
    **dst_acc.lamports.borrow_mut() = dst_lamports
      .checked_add(pool_acc.lamports())
      .ok_or(AppError::Overflow)?;
    **pool_acc.lamports.borrow_mut() = 0;

    Ok(())
  }

  ///
  /// Utilities
  ///
  fn multisig(dao_data: &DAO, signers: &[AccountInfo]) -> bool {
    let mut num_signers = 0;
    let mut matched = [false; MAX_SIGNERS];
    for signer in signers {
      for (position, key) in dao_data.signers.iter().enumerate() {
        if *key == Pubkey::new(&[0u8; 32]) {
          continue;
        }
        if signer.is_signer && *key == *signer.key && !matched[position] {
          matched[position] = true;
          num_signers += 1;
        }
      }
    }
    if num_signers == 0 {
      return false;
    }
    if dao_data.num_signers() * 10 / num_signers <= 15 {
      return true;
    }
    false
  }

  fn apply_fee(
    new_ask_reserve: u64,
    ask_reserve: u64,
    exempt: bool,
  ) -> Option<(u64, u64, u64, u64)> {
    let paid_amount_without_fee = ask_reserve.checked_sub(new_ask_reserve)?;
    let fee = (paid_amount_without_fee as u128)
      .checked_mul(FEE as u128)?
      .checked_div(DECIMALS as u128)? as u64;
    let mut earning = (paid_amount_without_fee as u128)
      .checked_mul(EARNING as u128)?
      .checked_div(DECIMALS as u128)? as u64;
    if exempt {
      earning = 0;
    }
    let new_ask_reserve_with_fee = new_ask_reserve.checked_add(fee)?;
    let paid_amount_with_fee = paid_amount_without_fee
      .checked_sub(fee)?
      .checked_sub(earning)?;
    Some((new_ask_reserve_with_fee, paid_amount_with_fee, fee, earning))
  }
}
