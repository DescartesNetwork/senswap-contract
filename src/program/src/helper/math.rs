use num_bigint::BigUint;

pub struct Math {}

impl Math {
  ///
  /// Babylonian method (with a selectively initial guesses)
  /// O(log(log(n))) for complexity
  ///
  pub fn sqrt(m: BigUint) -> BigUint {
    if m < BigUint::from(2u128) {
      return m;
    }
    let bits: u64 = (m.bits() + 1) / 2;
    let mut start = BigUint::from(1u128) << (bits - 1);
    let mut end = BigUint::from(1u128) << (bits + 1);
    while start < end {
      end = (start.clone() + end.clone()) >> 1;
      start = m.clone() / end.clone();
    }
    end
  }

  pub fn cbrt(m: BigUint) -> BigUint {
    if m == BigUint::from(0u128) {
      return BigUint::from(0u128);
    }
    if m < BigUint::from(8u128) {
      return BigUint::from(1u128);
    }
    let bits: u64 = (m.bits() + 1) / 2;
    let mut start = BigUint::from(1u128) << (bits - 1);
    let mut end = BigUint::from(1u128) << (bits + 1);
    while start < end {
      end = (start.clone() + end.clone()) >> 1;
      start = m.clone() / end.clone();
    }

    end
  }
}
