use crate::error::AppError;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq)]
pub enum AppInstruction {
  InitializePool {
    reserve_s: u64,
    reserve_a: u64,
    reserve_b: u64,
  },
  InitializeLPT,
  AddLiquidity {
    delta_s: u64,
    delta_a: u64,
    delta_b: u64,
  },
  RemoveLiquidity {
    lpt: u64,
  },
  Swap {
    amount: u64,
  },
  Transfer {
    lpt: u64,
  },
  FreezePool,
  ThawPool,
  Earn {
    amount: u64,
  },
  CloseLPT,
  TransferPoolOwnership,
  PreInitializePool,
}
impl AppInstruction {
  pub fn unpack(instruction: &[u8]) -> Result<Self, ProgramError> {
    let (&tag, rest) = instruction
      .split_first()
      .ok_or(AppError::InvalidInstruction)?;
    Ok(match tag {
      0 => {
        let reserve_s = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let reserve_a = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let reserve_b = rest
          .get(16..24)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::InitializePool {
          reserve_s,
          reserve_a,
          reserve_b,
        }
      }
      1 => Self::InitializeLPT,
      2 => {
        let delta_s = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let delta_a = rest
          .get(8..16)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        let delta_b = rest
          .get(16..24)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::AddLiquidity {
          delta_s,
          delta_a,
          delta_b,
        }
      }
      3 => {
        let lpt = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
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
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Transfer { lpt }
      }
      6 => Self::FreezePool,
      7 => Self::ThawPool,
      8 => {
        let amount = rest
          .get(..8)
          .and_then(|slice| slice.try_into().ok())
          .map(u64::from_le_bytes)
          .ok_or(AppError::InvalidInstruction)?;
        Self::Earn { amount }
      }
      9 => Self::CloseLPT,
      10 => Self::TransferPoolOwnership,
      11 => Self::PreInitializePool,
      _ => return Err(AppError::InvalidInstruction.into()),
    })
  }
}
