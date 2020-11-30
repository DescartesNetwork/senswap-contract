#![cfg(feature = "program")]

use crate::error::AppError;
use solana_sdk::program_error::ProgramError;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum AppInstruction {
  SayHello { amount: u32, toggle: bool },
}
impl AppInstruction {
  pub fn unpack(instruction: &[u8]) -> Result<Self, ProgramError> {
    let (&tag, rest) = instruction
      .split_first()
      .ok_or(AppError::InvalidInstruction)?;
    Ok(match tag {
      0 => {
        let amount = rest
          .get(..4)
          .and_then(|slice| slice.try_into().ok())
          .map(u32::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let toggle = match rest.get(4..5).ok_or(AppError::InvalidInstruction)? {
          [0] => false,
          [1] => true,
          _ => return Err(ProgramError::InvalidAccountData),
        };
        Self::SayHello { amount, toggle }
      }
      _ => return Err(AppError::InvalidInstruction.into()),
    })
  }
}
