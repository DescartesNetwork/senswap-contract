use solana_program::log::sol_log_compute_units;

///
/// Implement square/cubic root for u128
///
pub trait Roots {
  fn sqrt(self) -> Self;
  fn cbrt(self) -> Self;
}

impl Roots for u128 {
  ///
  /// Babylonian method (with a selectively initial guesses)
  /// O(log(log(n))) for complexity
  ///
  fn sqrt(self) -> Self {
    if self < 2 {
      return self;
    }

    let bits = (128 - self.leading_zeros() + 1) / 2;
    let mut start = 1 << (bits - 1);
    let mut end = 1 << (bits + 1);
    while start < end {
      end = (start + end) / 2;
      start = self / end;
    }
    end
  }

  ///
  /// Newton's method
  ///
  fn cbrt(self) -> Self {
    if self < 1 {
      return 0;
    }
    if self < 8 {
      return 1;
    }
    if self < 27 {
      return 2;
    }

    let mut end = self.sqrt();
    loop {
      sol_log_compute_units();
      let next = (self / end.pow(2) + 2 * end) / 3;
      if end != next {
        end = next;
      } else {
        break;
      }
    }
    end
  }
}
