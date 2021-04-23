use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;

const SINGLE_PRECISION: u128 = 1000000000; // 10^9
const DOUBLE_PRECISION: u128 = 1000000000000000000; // 10^18
const TRIPPLE_PRECISION: u128 = 1000000000000000000000000000; // 10^27
const FEE: u64 = 2500000; // 0.25%
const EARN: u64 = 500000; // 0.05%
const DECIMALS: u64 = 1000000000; // 10^9

pub struct Oracle {}

impl Oracle {
  pub fn curve(new_bid_reserve: u64, bid_reserve: u64, ask_reserve: u64) -> Option<u64> {
    if new_bid_reserve == 0 || bid_reserve == 0 || ask_reserve == 0 {
      return None;
    }
    let new_ask_reserve = (bid_reserve as u128)
      .checked_mul(ask_reserve as u128)?
      .checked_div(new_bid_reserve as u128)? as u64;
    if new_ask_reserve == 0 {
      return None;
    }

    Some(new_ask_reserve)
  }

  pub fn curve_in_fee(
    new_bid_reserve: u64,
    bid_reserve: u64,
    ask_reserve: u64,
    is_exempted: bool,
  ) -> Option<(u64, u64, u64)> {
    let new_ask_reserve_without_fee = Self::curve(new_bid_reserve, bid_reserve, ask_reserve)?;
    let fee = (new_ask_reserve_without_fee as u128)
      .checked_mul(FEE as u128)?
      .checked_div(DECIMALS as u128)? as u64;
    let earn: u64 = 0;
    if !is_exempted {
      earn = (new_ask_reserve_without_fee as u128)
        .checked_mul(EARN as u128)?
        .checked_div(DECIMALS as u128)? as u64;
    }

    let new_ask_reserve = new_ask_reserve_without_fee
      .checked_sub(fee)?
      .checked_sub(earn)?;
    if new_ask_reserve == 0 {
      return None;
    }

    Some((new_ask_reserve, fee, earn))
  }

  pub fn _rake(
    delta: u64,
    reserve_s: u64,
    reserve_a: u64,
    reserve_b: u64,
  ) -> Option<(u64, u64, u64)> {
    if reserve_s == 0 || reserve_a == 0 || reserve_b == 0 {
      return None;
    }
    if delta == 0 {
      return Some((0, 0, 0));
    }
    // Precision configs
    let double_precision = BigUint::from(DOUBLE_PRECISION);
    let tripple_precision = BigUint::from(TRIPPLE_PRECISION);
    // Compute z
    let cbrt_of_delta = (BigUint::from(delta) * tripple_precision).cbrt(); // Single precision
    let cbrt_of_reserve = (BigUint::from(reserve_s) * tripple_precision).cbrt(); // Single precision
    let z = (cbrt_of_delta.pow(2) * cbrt_of_reserve / tripple_precision)
      .to_u64()?
      .checked_sub(reserve_s)?;
    // Compute x
    let sqrt_of_delta_plus_reserve =
      ((BigUint::from(delta) + BigUint::from(reserve_s)) * double_precision).sqrt(); // Single precision
    let sqrt_of_reserve = (BigUint::from(reserve_s) * double_precision).sqrt(); // Single precision
    let x = (sqrt_of_delta_plus_reserve * sqrt_of_reserve / double_precision)
      .to_u64()?
      .checked_sub(reserve_s)?;
    // Compute y
    let y = z.checked_sub(x)?;
    // Compute s, a, b
    let s = delta.checked_sub(z)?;
    let a = (reserve_a as u128)
      .checked_mul(x as u128)?
      .checked_div(reserve_s.checked_add(x)? as u128)? as u64;
    let b = (reserve_b as u128)
      .checked_mul(y as u128)?
      .checked_div(reserve_s.checked_add(z)? as u128)? as u64;
    // Return
    Some((s, a, b))
  }

  pub fn rake(
    delta_s: u64,
    delta_a: u64,
    delta_b: u64,
    reserve_s: u64,
    reserve_a: u64,
    reserve_b: u64,
  ) -> Option<(u64, u64, u64, u64)> {
    let (s1, a1, b1) = Self::_rake(delta_s, reserve_s, reserve_a, reserve_b)?;
    let (a2, b2, s2) = Self::_rake(delta_a, reserve_a, reserve_b, reserve_s)?;
    let (b3, s3, a3) = Self::_rake(delta_b, reserve_b, reserve_s, reserve_a)?;
    let s = s1.checked_add(s2)?.checked_add(s3)?;
    let a = a1.checked_add(a2)?.checked_add(a3)?;
    let b = b1.checked_add(b2)?.checked_add(b3)?;
    let new_reserve_s = reserve_s.checked_add(delta_s)?;
    let new_reserve_a = reserve_a.checked_add(delta_a)?;
    let new_reserve_b = reserve_b.checked_add(delta_b)?;

    Some((s, new_reserve_s, new_reserve_a, new_reserve_b))
  }

  pub fn rake_in_fee(
    delta_s: u64,
    delta_a: u64,
    delta_b: u64,
    reserve_s: u64,
    reserve_a: u64,
    reserve_b: u64,
  ) -> Option<(u64, u64, u64, u64)> {
    let (s, new_reserve_s, new_reserve_a, new_reserve_b) =
      Self::rake(delta_s, delta_a, delta_b, reserve_s, reserve_a, reserve_b)?;

    let fee = (s as u128)
      .checked_mul(FEE as u128)?
      .checked_div(DECIMALS as u128)? as u64;
    let s_in_fee = s.checked_sub(fee)?;
    let new_reserve_s_in_fee = new_reserve_s.checked_add(fee)?;

    Some((s_in_fee, new_reserve_s_in_fee, new_reserve_a, new_reserve_b))
  }
}
