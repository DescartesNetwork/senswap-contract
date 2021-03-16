use crate::error::AppError;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum AppInstruction {
  InitializePool { reserve: u64, lpt: u128 },
  InitializeLPT,
  AddLiquidity { reserve: u64 },
  RemoveLiquidity { lpt: u128 },
  Swap { amount: u64 },
  Transfer { lpt: u128 },
  CloseLPT,
  ClosePool,
}
impl AppInstruction {
  pub fn unpack(instruction: &[u8]) -> Result<Self, ProgramError> {
    let (&tag, rest) = instruction
      .split_first()
      .ok_or(AppError::InvalidInstruction)?;
    Ok(match tag {
      0 => {
        let reserve = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let lpt = rest
          .get(8..24)
          .and_then(|slice| slice.try_into().ok())
          .map(u128::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::InitializePool { reserve, lpt }
      }
      1 => Self::InitializeLPT,
      2 => {
        let reserve = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::AddLiquidity { reserve }
      }
      3 => {
        let lpt = rest
          .get(..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u128::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::RemoveLiquidity { lpt }
      }
      4 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Swap { amount }
      }
      5 => {
        let lpt = rest
          .get(..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u128::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Transfer { lpt }
      }
      6 => Self::CloseLPT,
      7 => Self::ClosePool,
      _ => return Err(AppError::InvalidInstruction.into()),
    })
  }
}
