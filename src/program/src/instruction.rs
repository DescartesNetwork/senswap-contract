use crate::error::AppError;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum AppInstruction {
  InitializeNetwork,
  InitializePool { reserve: u64, lpt: u128 },
  InitializeLPT,
  AddLiquidity { reserve: u64 },
  RemoveLiquidity { lpt: u128 },
  Swap { amount: u64 },
  Transfer { lpt: u128 },
  FreezePool,
  ThawPool,
  AddSigner,
  ReplaceSigner,
  RemoveSigner,
  Earn { amount: u64 },
  CloseLPT,
  ClosePool,
}
impl AppInstruction {
  pub fn unpack(instruction: &[u8]) -> Result<Self, ProgramError> {
    let (&tag, rest) = instruction
      .split_first()
      .ok_or(AppError::InvalidInstruction)?;
    Ok(match tag {
      0 => Self::InitializeNetwork,
      1 => {
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
      2 => Self::InitializeLPT,
      3 => {
        let reserve = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::AddLiquidity { reserve }
      }
      4 => {
        let lpt = rest
          .get(..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u128::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::RemoveLiquidity { lpt }
      }
      5 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Swap { amount }
      }
      6 => {
        let lpt = rest
          .get(..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u128::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Transfer { lpt }
      }
      7 => Self::FreezePool,
      8 => Self::ThawPool,
      9 => Self::AddSigner,
      10 => Self::ReplaceSigner,
      11 => Self::RemoveSigner,
      12 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Earn { amount }
      }
      13 => Self::CloseLPT,
      14 => Self::ClosePool,
      _ => return Err(AppError::InvalidInstruction.into()),
    })
  }
}
