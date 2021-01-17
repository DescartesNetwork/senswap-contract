use crate::helper::umath::UMath;

pub struct Oracle {}

impl Oracle {
  fn beta(
    bid_reserve: u64,
    new_bid_reserve: u64,
    bid_lpt: u64,
    ask_reserve: u64,
    ask_lpt: u64,
  ) -> Option<UMath> {
    let one = UMath::new(1, 0)?;
    let two = UMath::new(2, 0)?;
    let four = UMath::new(4, 0)?;

    let _bid_reserve = UMath::new(bid_reserve, 0)?;
    let _new_bid_reserve = UMath::new(new_bid_reserve, 0)?;
    let _bid_lpt = UMath::new(bid_lpt, 0)?;
    let _ask_reserve = UMath::new(ask_reserve, 0)?;
    let _ask_lpt = UMath::new(ask_lpt, 0)?;

    let alpha = _bid_reserve.div(&_new_bid_reserve)?;
    let reversed_alpha = one.div(&alpha)?;
    let lambda = _bid_lpt.div(&_ask_lpt)?;
    let b = reversed_alpha.sub(&alpha)?.mul(&lambda)?;
    let sqrt_delta = b.sqr()?.add(&four)?.sqrt()?;
    let beta = sqrt_delta.sub(&b)?.div(&two)?;

    Some(beta)
  }

  pub fn new_ask_reserve_without_fee(
    bid_reserve: u64,
    new_bid_reserve: u64,
    bid_lpt: u64,
    ask_reserve: u64,
    ask_lpt: u64,
  ) -> Option<u64> {
    let _ask_reserve = UMath::new(ask_reserve, 0)?;
    let beta = Self::beta(bid_reserve, new_bid_reserve, bid_lpt, ask_reserve, ask_lpt)?;
    let new_ask_reserve = beta.mul(&_ask_reserve)?;

    Some(new_ask_reserve.i)
  }
}
