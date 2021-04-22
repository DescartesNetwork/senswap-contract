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

  pub fn rake(
    delta_s: u64,
    delta_a: u64,
    delta_b: u64,
    reserve_s: u64,
    reserve_a: u64,
    reserve_b: u64,
  ) -> Option<(u64, u64, u64, u64)> {
    let lpt = 0;
    let new_reserve_s = 0;
    let new_reserve_a = 0;
    let new_reserve_b = 0;

    Some((lpt, new_reserve_s, new_reserve_a, new_reserve_b))
  }
}
