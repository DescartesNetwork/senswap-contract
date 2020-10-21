#![cfg(feature = "program")]

use crate::error::AppError;
use solana_sdk::program_error::ProgramError;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum AppInstruction {
  SayHello,
}
impl AppInstruction {
  pub fn unpack(instruction: &[u8]) -> Result<Self, ProgramError> {
    let (&tag, _rest) = instruction
      .split_first()
      .ok_or(AppError::InvalidInstruction)?;
    Ok(match tag {
      0 => Self::SayHello,
      _ => return Err(AppError::InvalidInstruction.into()),
    })
  }
}
