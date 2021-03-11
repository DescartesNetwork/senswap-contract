use crate::helper::math::Math;
use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;

const PRECISION: u128 = 1000000000000000000; // 10**18
const DOUBLE_PRECISION: u128 = 1000000000000000000000000000000000000; // 10**36

pub struct Oracle {}

impl Oracle {
  ///
  /// alpha = bid_reserve / new_bid_reserve
  ///
  fn alpha(new_bid_reserve: u128, bid_reserve: u128) -> Option<BigUint> {
    let dpcs = BigUint::from(DOUBLE_PRECISION);
    let br = BigUint::from(bid_reserve) * dpcs;
    let nbr = BigUint::from(new_bid_reserve);
    let alpha = br / nbr;
    Some(alpha)
  }

  ///
  /// 1/alpha = new_bid_reserve / bid_reserve
  ///
  fn reversed_alpha(new_bid_reserve: u128, bid_reserve: u128) -> Option<BigUint> {
    let dpcs = BigUint::from(DOUBLE_PRECISION);
    let br = BigUint::from(bid_reserve);
    let nbr = BigUint::from(new_bid_reserve) * dpcs;
    let reversed_alpha = nbr / br;
    Some(reversed_alpha)
  }

  ///
  /// lambda = ask_lpt / bid_lpt
  ///
  fn lambda(bid_lpt: u128, ask_lpt: u128) -> Option<BigUint> {
    let pcs = BigUint::from(PRECISION);
    let bl = BigUint::from(bid_lpt);
    let al = BigUint::from(ask_lpt) * pcs;
    let lambda = al / bl;
    Some(lambda)
  }

  ///
  /// Beta curve: beta^2 + (1/alpha - alpha)/lambda * beta - 1
  /// Let's b = (1/alpha - alpha)/p
  /// Then delta = b^2 + 4 in the quadratic solution
  /// And beta = (-b + √(delta))/2
  ///
  fn beta(
    new_bid_reserve: u128,
    bid_reserve: u128,
    bid_lpt: u128,
    ask_lpt: u128,
  ) -> Option<BigUint> {
    let two = BigUint::from(2u128);
    let four = BigUint::from(4u128);
    let dpcs = BigUint::from(DOUBLE_PRECISION);

    let alpha = Self::alpha(new_bid_reserve, bid_reserve)?; // Double precision
    let reversed_alpha = Self::reversed_alpha(new_bid_reserve, bid_reserve)?; // Double precision
    let lambda = Self::lambda(bid_lpt, ask_lpt)?; // Single precision

    let b = (reversed_alpha - alpha) / lambda; // Single precision
    let delta = b.pow(2) + four * dpcs; // Double precision
    let beta = (Math::sqrt(delta) - b) / two; // Single precision

    Some(beta)
  }

  fn curve_u128(
    new_bid_reserve: u128,
    bid_reserve: u128,
    bid_lpt: u128,
    ask_reserve: u128,
    ask_lpt: u128,
  ) -> Option<u128> {
    let pcs = BigUint::from(PRECISION);
    let beta = Self::beta(new_bid_reserve, bid_reserve, bid_lpt, ask_lpt)?; // Single precision
    let ar = BigUint::from(ask_reserve);
    let nar = ar * beta; // Single precision
    let new_ask_reserve = (nar / pcs).to_u128()?;

    Some(new_ask_reserve)
  }

  fn curve_u64(
    new_bid_reserve: u64,
    bid_reserve: u64,
    bid_lpt: u64,
    ask_reserve: u64,
    ask_lpt: u64,
  ) -> Option<u64> {
    let new_ask_reserve = Self::curve_u128(
      new_bid_reserve as u128,
      bid_reserve as u128,
      bid_lpt as u128,
      ask_reserve as u128,
      ask_lpt as u128,
    )?;
    if new_ask_reserve != new_ask_reserve as u64 as u128 {
      return None;
    }
    Some(new_ask_reserve as u64)
  }

  pub fn curve(
    new_bid_reserve: u64,
    bid_reserve: u64,
    bid_lpt: u64,
    ask_reserve: u64,
    ask_lpt: u64,
  ) -> Option<u64> {
    let new_ask_reserve =
      Self::curve_u64(new_bid_reserve, bid_reserve, bid_lpt, ask_reserve, ask_lpt)?;
    Some(new_ask_reserve)
  }
}
