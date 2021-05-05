use crate::helper::math::Roots;

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
    let paid_amount_without_fee = ask_reserve.checked_sub(new_ask_reserve_without_fee)?;

    let fee = (paid_amount_without_fee as u128)
      .checked_mul(FEE as u128)?
      .checked_div(DECIMALS as u128)? as u64;
    let mut earning: u64 = 0;
    if !is_exempted {
      earning = (paid_amount_without_fee as u128)
        .checked_mul(EARNING as u128)?
        .checked_div(DECIMALS as u128)? as u64;
    }

    let paid_amount = paid_amount_without_fee
      .checked_sub(fee)?
      .checked_sub(earning)?;
    let new_ask_reserve = new_ask_reserve_without_fee.checked_add(fee)?;
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
    let cbrt_of_delta_plus_reserve = (delta as u128)
      .checked_add(reserve_s as u128)?
      .checked_mul(TRIPPLE_PRECISION)?
      .cbrt(); // Single precision
    let cbrt_of_reserve = (reserve_s as u128).checked_mul(TRIPPLE_PRECISION)?.cbrt(); // Single precision
    let z = cbrt_of_delta_plus_reserve
      .pow(2)
      .checked_mul(cbrt_of_reserve)?
      .checked_div(TRIPPLE_PRECISION)?
      .checked_sub(reserve_s as u128)?;
    // Compute x
    let x = z
      .checked_add(reserve_s as u128)?
      .checked_mul(reserve_s as u128)?
      .sqrt()
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
    let rs = reserve_s;
    let ra = reserve_a;
    let rb = reserve_b;
    let (s1, _a1, _b1) = Self::_rake(delta_s, rs, ra, rb)?;
    let rs = rs.checked_add(delta_s)?;
    let (_a2, _b2, s2) = Self::_rake(delta_a, ra, rb, rs)?;
    let ra = ra.checked_add(delta_a)?;
    let (_b3, s3, _a3) = Self::_rake(delta_b, ra, rs, ra)?;
    let rb = rb.checked_add(delta_b)?;
    let s = s1.checked_add(s2)?.checked_add(s3)?;

    Some((s, rs, ra, rb))
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
