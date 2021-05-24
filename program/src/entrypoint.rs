#![cfg(not(feature = "no-entrypoint"))]

use crate::{
  error::{AppError, PrintAppError},
  processor::Processor,
};
use solana_program::{
  account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);

fn process_instruction<'a>(
  program_id: &Pubkey,
  accounts: &'a [AccountInfo<'a>],
  instruction_data: &[u8],
) -> ProgramResult {
  if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
    error.print::<AppError>();
    return Err(error);
  }
  Ok(())
}
