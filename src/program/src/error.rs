#![cfg(feature = "program")]

use num_derive::FromPrimitive as DriveFromPrimitive;
use num_traits::FromPrimitive;
use solana_sdk::{
  decode_error::DecodeError,
  info,
  program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the App program.
#[derive(Clone, Debug, Eq, Error, DriveFromPrimitive, PartialEq)]
pub enum AppError {
  /// Incorrect program id
  #[error("Incorrect program id")]
  IncorrectProgramId,
  /// Operation overflowed
  #[error("Operation overflowed")]
  Overflow,
}

impl From<AppError> for ProgramError {
  fn from(e: AppError) -> Self {
    ProgramError::Custom(e as u32)
  }
}

impl<T> DecodeError<T> for AppError {
  fn type_of() -> &'static str {
    "AppError"
  }
}

impl PrintProgramError for AppError {
  fn print<E>(&self)
  where
    E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
  {
    match self {
      AppError::IncorrectProgramId => info!("Error: Incorrect program id"),
      AppError::Overflow => info!("Error: Operation overflowed"),
    }
  }
}
