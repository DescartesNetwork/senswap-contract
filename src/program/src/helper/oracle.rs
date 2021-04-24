use num_integer::Roots;

const DOUBLE_PRECISION: u128 = 1000000000000; // 10^12
const TRIPPLE_PRECISION: u128 = 1000000000000000000; // 10^18
const FEE: u64 = 2500000; // 0.25%
const EARNING: u64 = 500000; // 0.05%
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
    let mut earning: u64 = 0;
    if !is_exempted {
      earning = (new_ask_reserve_without_fee as u128)
        .checked_mul(EARNING as u128)?
        .checked_div(DECIMALS as u128)? as u64;
    }

    let new_ask_reserve = new_ask_reserve_without_fee
      .checked_sub(fee)?
      .checked_sub(earning)?;
    if new_ask_reserve == 0 {
      return None;
    }
    let paid_amount = new_ask_reserve.checked_sub(ask_reserve)?;

    Some((new_ask_reserve, paid_amount, earning))
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
    // Compute z
    let cbrt_of_delta = (delta as u128).checked_mul(TRIPPLE_PRECISION)?.cbrt(); // Single precision
    let cbrt_of_reserve = (reserve_s as u128).checked_mul(TRIPPLE_PRECISION)?.cbrt(); // Single precision
    let z = cbrt_of_delta
      .pow(2)
      .checked_mul(cbrt_of_reserve)?
      .checked_div(TRIPPLE_PRECISION)?
      .checked_sub(reserve_s as u128)?;
    // Compute x
    let sqrt_of_delta_plus_reserve = (delta as u128)
      .checked_add(reserve_s as u128)?
      .checked_mul(DOUBLE_PRECISION)?
      .sqrt(); // Single precision
    let sqrt_of_reserve = (reserve_s as u128).checked_mul(DOUBLE_PRECISION)?.sqrt(); // Single precision
    let x = sqrt_of_delta_plus_reserve
      .checked_mul(sqrt_of_reserve)?
      .checked_div(DOUBLE_PRECISION)?
      .checked_sub(reserve_s as u128)?;
    // Compute y
    let y = z.checked_sub(x)?;
    // Compute s, a, b
    let s = (delta as u128).checked_sub(z)? as u64;
    let a = (reserve_a as u128)
      .checked_mul(x)?
      .checked_div((reserve_s as u128).checked_add(x)?)? as u64;
    let b = (reserve_b as u128)
      .checked_mul(y)?
      .checked_div((reserve_s as u128).checked_add(z)?)? as u64;
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
    let _a = a1.checked_add(a2)?.checked_add(a3)?;
    let _b = b1.checked_add(b2)?.checked_add(b3)?;
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
